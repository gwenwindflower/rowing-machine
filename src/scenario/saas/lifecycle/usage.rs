use anyhow::Result;
use jiff::civil::Date;

use super::{DAY_MICROS, Lifecycle};
use crate::{engine::stream::Stream, output::Value, scenario::UnitRows};

struct Session {
    user: usize,
    day: usize,
    ordinal: usize,
    start: i64,
    end: i64,
    activation: bool,
    device: &'static str,
}

fn region_offset(seed: u64, account: usize) -> i8 {
    let mut profile = Stream::derive(seed, "saas.profile", &[account as u64]);
    profile.index(3);
    profile.index(6);
    [-5, 1, 9, -3][profile.index(4)]
}

fn frequency(
    engagement: f64,
    personal: f64,
    age: u32,
    date: Date,
    until_churn: Option<usize>,
) -> f64 {
    let weekday = if date.weekday().to_monday_zero_offset() < 5 {
        1.0
    } else {
        0.08
    };
    let holiday =
        if (date.month() == 12 && date.day() >= 24) || (date.month() == 1 && date.day() == 1) {
            0.2
        } else {
            1.0
        };
    let fade = until_churn.map_or(1.0, |days| {
        f64::from(u32::try_from(days.min(28)).expect("four weeks fit u32")) / 28.0
    });
    (0.4 + engagement * 1.2)
        * personal
        * (1.0 + (-f64::from(age) / 14.0).exp())
        * weekday
        * holiday
        * fade
}

fn sessions(
    seed: u64,
    account: usize,
    start: Date,
    days: usize,
    state: &Lifecycle,
) -> Result<Vec<Session>> {
    let base = start
        .at(0, 0, 0, 0)
        .to_zoned(jiff::tz::TimeZone::UTC)?
        .timestamp()
        .as_microsecond();
    let offset = i64::from(region_offset(seed, account));
    let arrival = state.users.first().map_or(days, |user| user.created);
    let mut sessions = Vec::new();
    for user in &state.users {
        let personal = 0.7
            + Stream::derive(
                seed,
                "saas.usage.personal",
                &[account as u64, user.index as u64],
            )
            .uniform()
                * 0.6;
        for day in user.created..user.departed.unwrap_or(days).min(days) {
            let midnight = base + i64::try_from(day)? * DAY_MICROS;
            if user.activated == Some(day) {
                sessions.push(Session {
                    user: user.index,
                    day,
                    ordinal: 0,
                    start: midnight,
                    end: midnight + 60_000_000,
                    activation: true,
                    device: "desktop",
                });
            }
            if day >= arrival + 14
                && !state
                    .subscriptions
                    .iter()
                    .any(|sub| sub.start <= day && sub.end.is_none_or(|end| day < end))
            {
                continue;
            }
            let until_churn = state
                .movements
                .iter()
                .find(|movement| movement.kind == "churn" && movement.day >= day)
                .map(|movement| movement.day - day);
            let date = start.checked_add(jiff::Span::new().days(i64::try_from(day)?))?;
            let rate = frequency(
                state.engagement,
                personal,
                u32::try_from(day - user.created)?,
                date,
                until_churn,
            );
            let mut rng = Stream::derive(
                seed,
                "saas.usage.sessions",
                &[account as u64, user.index as u64, day as u64],
            );
            let count = (0..4)
                .take_while(|&slot| rng.uniform() < (rate - f64::from(slot)).clamp(0.0, 1.0))
                .count();
            for ordinal in 1..=count {
                let minute = i64::try_from(rng.index(8 * 60))?;
                let started = midnight + ((9 - offset) * 60 + minute) * 60_000_000;
                let duration = i64::try_from(5 + rng.index(51))? * 60_000_000;
                let ended = (started + duration).min(base + i64::try_from(days)? * DAY_MICROS - 1);
                sessions.push(Session {
                    user: user.index,
                    day,
                    ordinal,
                    start: started,
                    end: ended,
                    activation: false,
                    device: if rng.uniform() < 0.85 {
                        "desktop"
                    } else {
                        "mobile"
                    },
                });
            }
        }
    }
    Ok(sessions)
}

pub(super) fn generate(
    seed: u64,
    account: usize,
    start: Date,
    days: usize,
    state: &Lifecycle,
) -> Result<UnitRows> {
    let account_id = Value::Uuid(Stream::derive(seed, "saas.account", &[account as u64]).uuid());
    let mut rows = Vec::new();
    for session in sessions(seed, account, start, days, state)? {
        let indices = [
            account as u64,
            session.user as u64,
            session.day as u64,
            session.ordinal as u64,
        ];
        rows.push((
            "sessions",
            vec![
                Value::Uuid(Stream::derive(seed, "saas.session", &indices).uuid()),
                Value::Uuid(
                    Stream::derive(seed, "saas.user", &[account as u64, session.user as u64])
                        .uuid(),
                ),
                account_id.clone(),
                Value::Timestamp(session.start),
                Value::Timestamp(session.end),
                Value::Text(session.device.into()),
            ],
        ));
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::saas::lifecycle::simulate;

    #[test]
    fn sessions_follow_membership_workdays_and_regional_business_hours() {
        let start = "2024-01-01".parse().unwrap();
        let mut weekdays = 0;
        let mut weekends = 0;
        for account in 0..30 {
            let state = simulate(42, account, 0, start, 120).unwrap();
            let sessions = sessions(42, account, start, 120, &state).unwrap();
            assert!(!sessions.is_empty());
            for session in sessions {
                let user = &state.users[session.user];
                assert!(session.day >= user.created);
                assert!(user.departed.is_none_or(|end| session.day < end));
                assert!(session.start < session.end);
                if session.activation {
                    assert_eq!(Some(session.day), user.activated);
                    continue;
                }
                let local = jiff::Timestamp::from_microsecond(session.start)
                    .unwrap()
                    .to_zoned(jiff::tz::TimeZone::fixed(
                        jiff::tz::Offset::from_hours(region_offset(42, account)).unwrap(),
                    ));
                assert!((9..17).contains(&local.hour()));
                if local.weekday().to_monday_zero_offset() < 5 {
                    weekdays += 1;
                } else {
                    weekends += 1;
                }
            }
        }
        assert!(weekdays > weekends * 8);
    }

    #[test]
    fn session_rate_decays_after_onboarding_dips_at_holidays_and_fades_before_churn() {
        let ordinary = "2024-06-03".parse().unwrap();
        let holiday = "2024-12-30".parse().unwrap();
        let rate = |age, date, until| frequency(0.6, 0.7, age, date, until);
        assert!(rate(0, ordinary, None) > rate(14, ordinary, None));
        assert!(rate(14, ordinary, None) > rate(90, ordinary, None));
        assert!(rate(90, holiday, None) < rate(90, ordinary, None) * 0.3);
        assert!(rate(90, ordinary, Some(7)) < rate(90, ordinary, Some(21)));
        assert_eq!(rate(90, ordinary, Some(0)), 0.0);
    }
}
