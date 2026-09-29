use anyhow::{Context, Result};
use jiff::civil::Date;

use crate::{engine::stream::Stream, output::Value, scenario::UnitRows, theme::Theme};

const DAY_MICROS: i64 = 86_400_000_000;

struct User {
    index: usize,
    created: usize,
    activated: Option<usize>,
    role: usize,
    departed: Option<usize>,
}
struct Subscription {
    index: usize,
    start: usize,
    end: Option<usize>,
    seats: i64,
    tier: usize,
    annual: bool,
}
impl Subscription {
    fn mrr(&self) -> i64 {
        self.seats * monthly_price(self.tier, self.annual)
    }
}

fn monthly_price(tier: usize, annual: bool) -> i64 {
    if annual {
        [1250, 2500, 5000][tier]
    } else {
        [1500, 3000, 6000][tier]
    }
}

fn churn_probability(tenure: f64, engagement: f64) -> f64 {
    (0.012 * (-tenure / 60.0).exp() + 0.0005) * (1.6 - engagement)
}

fn conversion_probability(activation_share: f64) -> f64 {
    0.15 + activation_share * 0.75
}
struct Movement {
    subscription: usize,
    day: usize,
    delta: i64,
    after: i64,
    kind: &'static str,
}
struct Invoice {
    subscription: usize,
    start: usize,
    end: usize,
    cycle_end: usize,
    paid: Option<usize>,
}
#[derive(Default)]
struct Lifecycle {
    users: Vec<User>,
    subscriptions: Vec<Subscription>,
    movements: Vec<Movement>,
    invoices: Vec<Invoice>,
}

pub(super) fn user_count(
    seed: u64,
    account: usize,
    arrival: usize,
    start: Date,
    days: usize,
) -> Result<usize> {
    Ok(simulate(seed, account, arrival, start, days)?.users.len())
}

fn add_user(state: &mut Lifecycle, rng: &mut Stream, day: usize, days: usize, engagement: f64) {
    let activated = (rng.uniform() < engagement).then(|| day + rng.index(7));
    state.users.push(User {
        index: state.users.len(),
        created: day,
        activated: activated.filter(|&d| d < days),
        role: rng.index(3),
        departed: None,
    });
}

fn cycle_end(start: Date, day: usize, annual: bool) -> Result<usize> {
    let date = start.checked_add(jiff::Span::new().days(i64::try_from(day)?))?;
    let end = date.checked_add(jiff::Span::new().months(if annual { 12 } else { 1 }))
        .with_context(|| format!("--start-date {start} puts billing after {date} beyond the supported calendar; choose an earlier --start-date or fewer --years"))?;
    Ok(usize::try_from(start.until(end)?.get_days())?)
}

fn issue(
    state: &mut Lifecycle,
    rng: &mut Stream,
    subscription: usize,
    day: usize,
    start: Date,
    days: usize,
) -> Result<()> {
    let cycle_end = cycle_end(start, day, state.subscriptions[subscription].annual)?;
    let draw = rng.uniform();
    let paid = if draw < 0.02 {
        None
    } else if draw < 0.15 {
        Some(day + 8 + rng.index(13))
    } else {
        Some(day)
    };
    state.invoices.push(Invoice {
        subscription,
        start: day,
        end: cycle_end.min(days),
        cycle_end,
        paid,
    });
    Ok(())
}

fn close(state: &mut Lifecycle, subscription: usize, day: usize) {
    state.subscriptions[subscription].end = Some(day);
    for invoice in state
        .invoices
        .iter_mut()
        .filter(|i| i.subscription == subscription)
    {
        invoice.end = invoice.end.min(day);
    }
}

fn change_membership(
    state: &mut Lifecycle,
    rng: &mut Stream,
    day: usize,
    days: usize,
    engagement: f64,
    band: usize,
) {
    if state.users.len() < 60 && rng.uniform() < [0.008, 0.018, 0.04][band] {
        add_user(state, rng, day, days, engagement);
    } else {
        let active = state
            .users
            .iter_mut()
            .filter(|u| u.departed.is_none())
            .collect::<Vec<_>>();
        if active.len() > 1 && rng.uniform() < 0.006 {
            let index = rng.index(active.len());
            active
                .into_iter()
                .nth(index)
                .expect("active user exists")
                .departed = Some(day);
        }
    }
}

fn simulate(
    seed: u64,
    account: usize,
    arrival: usize,
    start: Date,
    days: usize,
) -> Result<Lifecycle> {
    let mut state = Lifecycle::default();
    if arrival >= days {
        return Ok(state);
    }
    let mut rng = Stream::derive(seed, "saas.lifecycle", &[account as u64]);
    let band = Stream::derive(seed, "saas.profile", &[account as u64]).index(3);
    let engagement = 0.25 + rng.uniform() * 0.7;
    let tier = if rng.uniform() < 0.7 {
        band
    } else {
        rng.index(3)
    };
    let annual = rng.uniform() < 0.25;
    for _ in 0..[2, 5, 12][band] {
        add_user(&mut state, &mut rng, arrival, days, engagement);
    }
    let mut current = None;
    let mut ever_paid = false;
    let mut churned = None;
    for day in arrival..days {
        let old = current.map_or(0, |index: usize| state.subscriptions[index].mrr());
        let overdue = state
            .invoices
            .iter()
            .any(|i| day > i.start + 30 && i.paid.is_none_or(|paid| paid > day));
        let tenure = f64::from(u32::try_from(day - arrival)?);
        let hazard = churn_probability(tenure, engagement);
        let mut desired = current.is_some();
        if current.is_some() && (overdue || rng.uniform() < hazard) {
            desired = false;
            churned = Some(day);
        } else if current.is_none() && !ever_paid && day == arrival + 14 {
            let activated = f64::from(u32::try_from(
                state
                    .users
                    .iter()
                    .filter(|u| u.activated.is_some_and(|a| a <= day))
                    .count(),
            )?) / f64::from(u32::try_from(state.users.len())?);
            desired = rng.uniform() < conversion_probability(activated);
        } else if current.is_none() && churned.is_some_and(|d| day > d + 45) && !overdue {
            desired = rng.uniform() < 0.0015;
        }
        if desired && current.is_some() {
            change_membership(&mut state, &mut rng, day, days, engagement, band);
        }
        let seats = i64::try_from(state.users.iter().filter(|u| u.departed.is_none()).count())?;
        let after = if desired {
            seats * monthly_price(tier, annual)
        } else {
            0
        };
        if after != old {
            let ended = current.take();
            if let Some(index) = ended {
                close(&mut state, index, day);
            }
            let subscription = if desired {
                let index = state.subscriptions.len();
                state.subscriptions.push(Subscription {
                    index,
                    start: day,
                    end: None,
                    seats,
                    tier,
                    annual,
                });
                issue(&mut state, &mut rng, index, day, start, days)?;
                current = Some(index);
                index
            } else {
                ended.expect("positive MRR has a subscription")
            };
            let kind = if after == 0 {
                "churn"
            } else if old == 0 {
                if ever_paid { "reactivation" } else { "new" }
            } else if after > old {
                "expansion"
            } else {
                "contraction"
            };
            state.movements.push(Movement {
                subscription,
                day,
                delta: after - old,
                after,
                kind,
            });
            ever_paid |= desired;
        } else if let Some(index) = current
            && state.invoices.last().is_some_and(|i| i.end == day)
        {
            issue(&mut state, &mut rng, index, day, start, days)?;
        }
    }
    Ok(state)
}

fn slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

fn status(sub: &Subscription, state: &Lifecycle, days: usize) -> &'static str {
    if sub.end.is_some() {
        "canceled"
    } else if state
        .invoices
        .iter()
        .any(|i| i.paid.is_none_or(|paid| paid >= days))
    {
        "past_due"
    } else {
        "active"
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate(
    seed: u64,
    account: usize,
    arrival: usize,
    start: Date,
    days: usize,
    theme: &Theme,
    user_name_offset: usize,
    account_name: &str,
) -> Result<UnitRows> {
    let state = simulate(seed, account, arrival, start, days)?;
    let account_id = Value::Uuid(Stream::derive(seed, "saas.account", &[account as u64]).uuid());
    let base = start
        .at(0, 0, 0, 0)
        .to_zoned(jiff::tz::TimeZone::UTC)?
        .timestamp()
        .as_microsecond();
    let timestamp = |day: usize| {
        Value::Timestamp(base + i64::try_from(day).expect("calendar day fits i64") * DAY_MICROS)
    };
    let id = |kind: &str, index: usize| {
        Value::Uuid(Stream::derive(seed, kind, &[account as u64, index as u64]).uuid())
    };
    let mut rows = Vec::new();
    for user in &state.users {
        let name = theme.name(seed, "person", user_name_offset + user.index);
        let email = format!(
            "{}-{account}-{}@{}.example",
            slug(&name),
            user.index,
            slug(account_name)
        );
        rows.push((
            "users",
            vec![
                id("saas.user", user.index),
                account_id.clone(),
                Value::Text(name),
                Value::Text(email),
                Value::Text(theme.label("roles", user.role).into()),
                timestamp(user.created),
                user.activated.map_or(Value::Null, timestamp),
            ],
        ));
    }
    for sub in &state.subscriptions {
        rows.push((
            "subscriptions",
            vec![
                id("saas.subscription", sub.index),
                account_id.clone(),
                Value::Uuid(Stream::derive(seed, "saas.plan", &[sub.tier as u64]).uuid()),
                Value::Text(if sub.annual { "annual" } else { "monthly" }.into()),
                Value::Integer(sub.seats),
                Value::Cents(sub.mrr()),
                timestamp(sub.start),
                sub.end.map_or(Value::Null, timestamp),
                Value::Text(status(sub, &state, days).into()),
            ],
        ));
    }
    for (index, movement) in state.movements.iter().enumerate() {
        rows.push((
            "mrr_movements",
            vec![
                id("saas.movement", index),
                account_id.clone(),
                id("saas.subscription", movement.subscription),
                Value::Text(movement.kind.into()),
                timestamp(movement.day),
                Value::Cents(movement.delta),
                Value::Cents(movement.after),
            ],
        ));
    }
    for (index, invoice) in state.invoices.iter().enumerate() {
        let sub = &state.subscriptions[invoice.subscription];
        let full = sub.mrr() * if sub.annual { 12 } else { 1 };
        let duration = i64::try_from(invoice.end - invoice.start)?;
        let cycle = i64::try_from(invoice.cycle_end - invoice.start)?;
        let amount = (full * duration + cycle / 2) / cycle;
        rows.push((
            "invoices",
            vec![
                id("saas.invoice", index),
                account_id.clone(),
                id("saas.subscription", invoice.subscription),
                timestamp(invoice.start),
                timestamp(invoice.start),
                timestamp(invoice.end),
                Value::Cents(amount),
                invoice
                    .paid
                    .filter(|&d| d < days)
                    .map_or(Value::Null, timestamp),
            ],
        ));
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engagement_and_tenure_reduce_churn_and_activation_increases_conversion() {
        for tenure in [0.0, 30.0, 180.0, 365.0] {
            assert!(churn_probability(tenure, 0.25) > churn_probability(tenure, 0.95));
            assert!(churn_probability(tenure, 0.5) > churn_probability(tenure + 30.0, 0.5));
        }
        for share in [0.0, 0.25, 0.5, 0.75] {
            assert!(conversion_probability(share) < conversion_probability(share + 0.25));
        }
        let start = "2024-01-01".parse().unwrap();
        let mut trials = [0_u32; 2];
        let mut converted = [0_u32; 2];
        for account in 0..2000 {
            let state = simulate(42, account, 0, start, 16).unwrap();
            let activated = state.users.iter().filter(|u| u.activated.is_some()).count();
            let cohort = usize::from(activated * 2 >= state.users.len());
            trials[cohort] += 1;
            converted[cohort] += u32::from(!state.subscriptions.is_empty());
        }
        assert!(
            f64::from(converted[1]) / f64::from(trials[1])
                > f64::from(converted[0]) / f64::from(trials[0]) + 0.2
        );
    }

    #[test]
    fn larger_employee_bands_start_with_more_users_choose_higher_tiers_and_grow_faster() {
        let start = "2024-01-01".parse().unwrap();
        let mut accounts = [0_u32; 3];
        let mut initial = [0_u32; 3];
        let mut growth = [0_u32; 3];
        let mut paid = [0_u32; 3];
        let mut tiers = [0_u32; 3];
        for account in 0..1500 {
            let state = simulate(42, account, 0, start, 365).unwrap();
            let band = Stream::derive(42, "saas.profile", &[account as u64]).index(3);
            accounts[band] += 1;
            initial[band] +=
                u32::try_from(state.users.iter().filter(|u| u.created == 0).count()).unwrap();
            growth[band] +=
                u32::try_from(state.users.iter().filter(|u| u.created > 0).count()).unwrap();
            if let Some(sub) = state.subscriptions.first() {
                paid[band] += 1;
                tiers[band] += u32::try_from(sub.tier).unwrap();
            }
        }
        for band in 0..2 {
            assert!(
                f64::from(initial[band]) / f64::from(accounts[band])
                    < f64::from(initial[band + 1]) / f64::from(accounts[band + 1])
            );
            assert!(
                f64::from(growth[band]) / f64::from(accounts[band])
                    < f64::from(growth[band + 1]) / f64::from(accounts[band + 1])
            );
            assert!(
                f64::from(tiers[band]) / f64::from(paid[band])
                    < f64::from(tiers[band + 1]) / f64::from(paid[band + 1])
            );
        }
    }

    #[test]
    fn seats_follow_active_membership_and_movements_follow_revenue() {
        let start = "2024-01-01".parse().unwrap();
        let mut kinds = std::collections::BTreeSet::new();
        for account in 0..300 {
            let state = simulate(71, account, 5, start, 500).unwrap();
            for sub in &state.subscriptions {
                let seats = state
                    .users
                    .iter()
                    .filter(|u| u.created <= sub.start && u.departed.is_none_or(|d| d > sub.start))
                    .count();
                assert_eq!(sub.seats, i64::try_from(seats).unwrap());
            }
            let mut previous = 0;
            let mut paid = false;
            for movement in &state.movements {
                let expected = if movement.after == 0 {
                    "churn"
                } else if previous == 0 {
                    if paid { "reactivation" } else { "new" }
                } else if movement.after > previous {
                    "expansion"
                } else {
                    "contraction"
                };
                assert_eq!(movement.kind, expected);
                assert_eq!(movement.after - previous, movement.delta);
                kinds.insert(movement.kind);
                paid |= movement.after > 0;
                previous = movement.after;
            }
            for user in &state.users {
                assert!(user.created >= 5);
                if let Some(activated) = user.activated {
                    assert!((user.created..user.created + 7).contains(&activated));
                }
            }
        }
        assert_eq!(
            kinds,
            std::collections::BTreeSet::from([
                "new",
                "expansion",
                "contraction",
                "churn",
                "reactivation"
            ])
        );
    }

    #[test]
    fn invoices_include_late_payments_and_unpaid_accounts_churn_after_grace() {
        let start = "2024-01-01".parse().unwrap();
        let mut late = 0;
        let mut unpaid = 0;
        for account in 0..300 {
            let state = simulate(42, account, 0, start, 400).unwrap();
            for invoice in &state.invoices {
                assert!(invoice.start < invoice.end);
                if invoice.paid.is_some_and(|paid| paid > invoice.start) {
                    late += 1;
                }
                if invoice.paid.is_none() && invoice.start + 31 < 400 {
                    unpaid += 1;
                    let deadline = invoice.start + 31;
                    assert!(state.subscriptions.iter().all(|s| s.start > deadline || s.end.is_some_and(|end| end <= deadline)));
                }
            }
        }
        assert!(late > 0);
        assert!(unpaid > 0);
    }

    #[test]
    fn billing_cycles_follow_calendar_months_and_years() {
        let start = "2024-01-31".parse().unwrap();
        assert_eq!(cycle_end(start, 0, false).unwrap(), 29);
        assert_eq!(cycle_end(start, 0, true).unwrap(), 366);
    }

    #[test]
    fn active_status_reflects_unpaid_invoices_from_ended_intervals() {
        let start = "2024-01-01".parse().unwrap();
        let theme = Theme::load("plain").unwrap();
        let mut checked = 0;
        for account in 0..500 {
            let state = simulate(42, account, 0, start, 70).unwrap();
            if state.subscriptions.last().is_some_and(|s| s.end.is_none())
                && state
                    .invoices
                    .iter()
                    .any(|i| i.paid.is_none_or(|p| p >= 70))
            {
                let rows = generate(42, account, 0, start, 70, &theme, 0, "Example Org").unwrap();
                let sub = rows
                    .iter()
                    .rev()
                    .find(|(entity, _)| *entity == "subscriptions")
                    .unwrap();
                assert_eq!(
                    sub.1[8],
                    Value::Text("past_due".into()),
                    "account {account}"
                );
                checked += 1;
            }
        }
        assert!(checked > 0);
    }

    #[test]
    fn revenue_ledger_matches_active_subscription_every_day() {
        let start = "2024-01-01".parse().unwrap();
        for account in 0..80 {
            let state = simulate(42, account, 0, start, 400).unwrap();
            let mut mrr = 0;
            for day in 0..400 {
                for movement in state.movements.iter().filter(|m| m.day == day) {
                    mrr += movement.delta;
                    assert_eq!(mrr, movement.after);
                }
                let active: i64 = state
                    .subscriptions
                    .iter()
                    .filter(|s| s.start <= day && s.end.is_none_or(|end| day < end))
                    .map(Subscription::mrr)
                    .sum();
                assert_eq!(active, mrr);
            }
            for sub in &state.subscriptions {
                let invoices: Vec<_> = state
                    .invoices
                    .iter()
                    .filter(|i| i.subscription == sub.index)
                    .collect();
                assert_eq!(invoices.first().unwrap().start, sub.start);
                assert_eq!(invoices.last().unwrap().end, sub.end.unwrap_or(400));
                for pair in invoices.windows(2) {
                    assert_eq!(pair[0].end, pair[1].start);
                }
            }
        }
    }
}
