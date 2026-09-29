use anyhow::Result;
use jiff::civil::Date;

use super::{DAY_MICROS, Lifecycle};
use crate::{engine::stream::Stream, output::Value, scenario::UnitRows, theme::Theme};

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

fn choose_feature(rng: &mut Stream, tier: usize, role: usize) -> usize {
    let weights: [usize; 16] = std::array::from_fn(|feature| {
        (if feature % 3 == tier { 5 } else { 1 }) * (if (feature / 3) % 3 == role { 5 } else { 1 })
    });
    let mut draw = rng.index(weights.iter().sum());
    for (feature, weight) in weights.into_iter().enumerate() {
        if draw < weight {
            return feature;
        }
        draw -= weight;
    }
    unreachable!("feature weights cover every draw")
}

pub(super) fn generate(
    seed: u64,
    account: usize,
    start: Date,
    days: usize,
    state: &Lifecycle,
    theme: &Theme,
) -> Result<UnitRows> {
    let account_id = Value::Uuid(Stream::derive(seed, "saas.account", &[account as u64]).uuid());
    let mut rows = Vec::new();
    let features: Vec<_> = (0..16)
        .map(|index| theme.name(seed, "feature", index))
        .collect();
    for session in sessions(seed, account, start, days, state)? {
        let indices = [
            account as u64,
            session.user as u64,
            session.day as u64,
            session.ordinal as u64,
        ];
        let session_id = Value::Uuid(Stream::derive(seed, "saas.session", &indices).uuid());
        let user_id = Value::Uuid(
            Stream::derive(seed, "saas.user", &[account as u64, session.user as u64]).uuid(),
        );
        rows.push((
            "sessions",
            vec![
                session_id.clone(),
                user_id.clone(),
                account_id.clone(),
                Value::Timestamp(session.start),
                Value::Timestamp(session.end),
                Value::Text(session.device.into()),
            ],
        ));
        let mut rng = Stream::derive(seed, "saas.usage.events", &indices);
        let count = if session.activation {
            1
        } else {
            4 + rng.index(9)
        };
        for event in 0..count {
            let feature = if session.activation {
                0
            } else {
                choose_feature(&mut rng, state.tier, state.users[session.user].role)
            };
            let action = if session.activation {
                "activated"
            } else {
                "used"
            };
            let timestamp = session.start
                + (session.end - session.start) * i64::try_from(event)? / i64::try_from(count)?;
            rows.push((
                "events",
                vec![
                    Value::Uuid(
                        Stream::derive(
                            seed,
                            "saas.event",
                            &[indices[0], indices[1], indices[2], indices[3], event as u64],
                        )
                        .uuid(),
                    ),
                    session_id.clone(),
                    user_id.clone(),
                    account_id.clone(),
                    Value::Timestamp(timestamp),
                    Value::Text(format!("{}:{action}", features[feature])),
                    Value::Text(features[feature].clone()),
                ],
            ));
        }
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::saas::lifecycle::simulate;

    #[test]
    fn emitted_sessions_settle_after_onboarding_and_scale_with_active_user_days() {
        use super::super::{Subscription, User};
        let start = "2024-03-04".parse().unwrap();
        let state = Lifecycle {
            engagement: 0.6,
            users: (0..60)
                .map(|index| User {
                    index,
                    created: 0,
                    activated: None,
                    role: 0,
                    departed: None,
                })
                .collect(),
            subscriptions: vec![Subscription {
                index: 0,
                start: 0,
                end: None,
                seats: 60,
                tier: 0,
                annual: false,
            }],
            ..Lifecycle::default()
        };
        let sessions = sessions(42, 0, start, 168, &state).unwrap();
        let count = |from, to| {
            sessions
                .iter()
                .filter(|s| (from..to).contains(&s.day))
                .count()
        };
        let onboarding = count(0, 28);
        let settled = count(84, 112);
        let later = count(140, 168);
        assert!(onboarding * 10 > settled * 13);
        assert!(later * 10 > settled * 8 && later * 10 < settled * 12);
        let half_users = sessions.iter().filter(|s| s.user < 30).count();
        assert!(half_users * 10 > sessions.len() * 4 && half_users * 10 < sessions.len() * 6);
    }

    #[test]
    fn feature_choices_vary_with_tier_and_role() {
        let mut counts = [[[0_u32; 16]; 3]; 3];
        for (tier, roles) in counts.iter_mut().enumerate() {
            for (role, features) in roles.iter_mut().enumerate() {
                let mut rng = Stream::derive(42, "test.feature", &[]);
                for _ in 0..20_000 {
                    features[choose_feature(&mut rng, tier, role)] += 1;
                }
            }
        }
        for (feature, _) in counts[0][0].iter().enumerate() {
            let favored_tier = feature % 3;
            let favored_role = (feature / 3) % 3;
            assert!(
                counts[favored_tier][favored_role][feature]
                    > counts[(favored_tier + 1) % 3][favored_role][feature] * 2
            );
            assert!(
                counts[favored_tier][favored_role][feature]
                    > counts[favored_tier][(favored_role + 1) % 3][feature] * 2
            );
        }
    }

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
        assert!(rate(90, ordinary, Some(0)).abs() < f64::EPSILON);
    }
}
