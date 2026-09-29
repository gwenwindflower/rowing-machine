use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use assert_cmd::cargo::cargo_bin_cmd;
use jiff::civil::Date;
use serde_json::Value;

type Tables = BTreeMap<&'static str, Vec<Value>>;
const ENTITIES: [&str; 7] = [
    "accounts",
    "users",
    "plans",
    "subscriptions",
    "mrr_movements",
    "invoices",
    "opportunities",
];
const DAYS: i64 = 1460;

fn text<'a>(row: &'a Value, column: &str) -> &'a str {
    row[column].as_str().unwrap()
}

fn number(row: &Value, column: &str) -> i64 {
    row[column].as_i64().unwrap()
}

fn date(row: &Value, column: &str) -> Date {
    text(row, column)[..10].parse().unwrap()
}

fn day(row: &Value, column: &str) -> i64 {
    "2023-01-01"
        .parse::<Date>()
        .unwrap()
        .until(date(row, column))
        .unwrap()
        .get_days()
        .into()
}

fn end(row: &Value) -> i64 {
    if row["ended_at"].is_null() {
        DAYS
    } else {
        day(row, "ended_at")
    }
}

fn read(directory: &Path, entity: &str) -> Vec<Value> {
    std::fs::read_to_string(directory.join(format!("raw_{entity}.jsonl")))
        .unwrap_or_else(|error| panic!("reading {entity}: {error}"))
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn saas_output_supports_revenue_reconciliation_and_retention_queries() {
    let directory = tempfile::tempdir().unwrap();
    cargo_bin_cmd!("rowing-machine")
        .args([
            "--scenario",
            "saas",
            "--years",
            "4",
            "--scale",
            "30",
            "--seed",
            "42",
            "--format",
            "jsonl",
            "--quiet",
            "--output-dir",
        ])
        .arg(directory.path())
        .assert()
        .success();
    let tables: Tables = ENTITIES
        .into_iter()
        .map(|name| (name, read(directory.path(), name)))
        .collect();
    assert_activation(&tables);
    assert_subscriptions(&tables);
    assert_movements(&tables);
    assert_invoices(&tables);
    assert_monthly_metrics(&tables);
    assert_self_serve_signup_retention(&tables);
    assert_paying_customer_retention(&tables);
    assert_relations(&tables);
}

fn assert_relations(tables: &Tables) {
    let keys: BTreeMap<_, BTreeSet<_>> = tables
        .iter()
        .map(|(&entity, rows)| {
            assert!(!rows.is_empty(), "{entity} is empty");
            let ids: BTreeSet<_> = rows.iter().map(|row| text(row, "id")).collect();
            assert!(
                ids.iter().all(|id| !id.trim().is_empty()),
                "empty {entity} primary key"
            );
            assert_eq!(ids.len(), rows.len(), "duplicate {entity} primary key");
            (entity, ids)
        })
        .collect();
    for (entity, field, target) in [
        ("users", "account_id", "accounts"),
        ("subscriptions", "account_id", "accounts"),
        ("subscriptions", "plan_id", "plans"),
        ("mrr_movements", "account_id", "accounts"),
        ("mrr_movements", "subscription_id", "subscriptions"),
        ("invoices", "account_id", "accounts"),
        ("invoices", "subscription_id", "subscriptions"),
    ] {
        for row in &tables[entity] {
            assert!(
                keys[target].contains(text(row, field)),
                "dangling {entity}.{field}"
            );
        }
    }
    for (child, foreign, parent) in [
        ("mrr_movements", "subscription_id", "subscriptions"),
        ("invoices", "subscription_id", "subscriptions"),
    ] {
        let parents: BTreeMap<_, _> = tables[parent]
            .iter()
            .map(|row| (text(row, "id"), row))
            .collect();
        for row in &tables[child] {
            assert_eq!(row["account_id"], parents[text(row, foreign)]["account_id"]);
        }
    }
}

fn assert_activation(tables: &Tables) {
    let accounts: BTreeMap<_, _> = tables["accounts"]
        .iter()
        .map(|row| (text(row, "id"), row))
        .collect();
    let mut activated = 0;
    for user in &tables["users"] {
        assert!(day(user, "created_at") >= day(accounts[text(user, "account_id")], "created_at"));
        if !user["activated_at"].is_null() {
            let delay = day(user, "activated_at") - day(user, "created_at");
            assert!((0..=7).contains(&delay));
            activated += 1;
        }
    }
    assert!(activated > 0 && activated < tables["users"].len());
}

fn assert_subscriptions(tables: &Tables) {
    let plans: BTreeMap<_, _> = tables["plans"]
        .iter()
        .map(|row| (text(row, "id"), row))
        .collect();
    let mut intervals = BTreeSet::new();
    for sub in &tables["subscriptions"] {
        let plan = plans[text(sub, "plan_id")];
        let seats = number(sub, "seats");
        assert!(seats > 0);
        let interval = text(sub, "billing_interval");
        intervals.insert(interval);
        let expected = match interval {
            "monthly" => seats * number(plan, "seat_price_monthly"),
            "annual" => (seats * number(plan, "seat_price_annual") + 6) / 12,
            other => panic!("unknown billing interval {other}"),
        };
        assert_eq!(number(sub, "mrr"), expected);
        assert!(day(sub, "started_at") < end(sub));
        let unpaid = tables["invoices"].iter().any(|invoice| {
            invoice["account_id"] == sub["account_id"] && invoice["paid_at"].is_null()
        });
        let expected_status = if !sub["ended_at"].is_null() {
            "canceled"
        } else if unpaid {
            "past_due"
        } else {
            "active"
        };
        assert_eq!(text(sub, "status"), expected_status);
    }
    assert_eq!(intervals, BTreeSet::from(["annual", "monthly"]));
}

fn active_mrr(subscriptions: &[&Value], at: i64) -> i64 {
    subscriptions
        .iter()
        .filter(|sub| day(sub, "started_at") <= at && at < end(sub))
        .map(|sub| number(sub, "mrr"))
        .sum()
}

fn assert_movements(tables: &Tables) {
    let mut kinds = BTreeSet::new();
    for account in &tables["accounts"] {
        let mut movements: Vec<_> = tables["mrr_movements"]
            .iter()
            .filter(|row| row["account_id"] == account["id"])
            .collect();
        movements.sort_by_key(|row| day(row, "occurred_at"));
        let subscriptions: Vec<_> = tables["subscriptions"]
            .iter()
            .filter(|row| row["account_id"] == account["id"])
            .collect();
        let mut before = 0;
        let mut ever_paid = false;
        for movement in movements {
            let after = number(movement, "mrr_after");
            let kind = if after == 0 {
                "churn"
            } else if before == 0 {
                if ever_paid { "reactivation" } else { "new" }
            } else if after > before {
                "expansion"
            } else {
                "contraction"
            };
            assert_ne!(after, before);
            assert!(after >= 0);
            assert_eq!(text(movement, "movement_type"), kind);
            kinds.insert(kind);
            assert_eq!(before + number(movement, "mrr_delta"), after);
            assert_eq!(
                active_mrr(&subscriptions, day(movement, "occurred_at")),
                after
            );
            before = after;
            ever_paid |= after > 0;
        }
    }
    assert_eq!(
        kinds,
        BTreeSet::from(["new", "expansion", "contraction", "churn", "reactivation"])
    );
}

fn assert_invoices(tables: &Tables) {
    let mut late = 0;
    let mut never_paid = 0;
    for sub in &tables["subscriptions"] {
        let mut invoices: Vec<_> = tables["invoices"]
            .iter()
            .filter(|row| row["subscription_id"] == sub["id"])
            .collect();
        invoices.sort_by_key(|row| day(row, "period_start"));
        assert!(!invoices.is_empty());
        let mut expected_start = day(sub, "started_at");
        for invoice in invoices {
            let start = day(invoice, "period_start");
            let finish = day(invoice, "period_end");
            assert_eq!(start, expected_start);
            assert!(finish > start);
            assert!(day(invoice, "issued_at") <= start);
            let annual = text(sub, "billing_interval") == "annual";
            let cycle = date(invoice, "period_start")
                .checked_add(jiff::Span::new().months(if annual { 12 } else { 1 }))
                .unwrap();
            let cycle_days = i64::from(
                date(invoice, "period_start")
                    .until(cycle)
                    .unwrap()
                    .get_days(),
            );
            assert_eq!(finish, (start + cycle_days).min(end(sub)));
            let full = number(sub, "mrr") * if annual { 12 } else { 1 };
            assert_eq!(
                number(invoice, "amount"),
                (full * (finish - start) + cycle_days / 2) / cycle_days
            );
            if invoice["paid_at"].is_null() {
                if start + 31 < DAYS {
                    never_paid += 1;
                    let account_subs: Vec<_> = tables["subscriptions"]
                        .iter()
                        .filter(|row| row["account_id"] == sub["account_id"])
                        .collect();
                    assert_eq!(
                        active_mrr(&account_subs, start + 31),
                        0,
                        "unpaid invoice remains active past grace period"
                    );
                }
            } else {
                let paid = day(invoice, "paid_at");
                assert!(paid >= day(invoice, "issued_at") && paid < DAYS);
                late += usize::from(paid > day(invoice, "issued_at"));
            }
            expected_start = finish;
        }
        assert_eq!(expected_start, end(sub));
    }
    assert!(late > 0);
    assert!(never_paid > 0);
    assert!(never_paid * 10 < tables["invoices"].len());
}

fn assert_monthly_metrics(tables: &Tables) {
    let subscriptions: Vec<_> = tables["subscriptions"].iter().collect();
    let start: Date = "2023-01-01".parse().unwrap();
    let mut positive_months = 0;
    for month in 0..48 {
        let at = i64::from(
            start
                .until(start.checked_add(jiff::Span::new().months(month)).unwrap())
                .unwrap()
                .get_days(),
        );
        let ledger: i64 = tables["mrr_movements"]
            .iter()
            .filter(|row| day(row, "occurred_at") <= at)
            .map(|row| number(row, "mrr_delta"))
            .sum();
        let mrr = active_mrr(&subscriptions, at);
        assert_eq!(ledger, mrr);
        let arr_from_contracts: i64 = subscriptions
            .iter()
            .filter(|sub| day(sub, "started_at") <= at && at < end(sub))
            .map(|sub| number(sub, "mrr") * 12)
            .sum();
        assert_eq!(arr_from_contracts, ledger * 12);
        positive_months += usize::from(mrr > 0);
    }
    assert!(positive_months > 36);
}

fn assert_self_serve_signup_retention(tables: &Tables) {
    let prospects: BTreeSet<_> = tables["opportunities"]
        .iter()
        .map(|row| text(row, "account_id"))
        .collect();
    let mut retained = [0_i64; 4];
    let mut cohort = 0;
    for account in &tables["accounts"] {
        let signup = day(account, "created_at");
        if signup >= 365 || prospects.contains(text(account, "id")) {
            continue;
        }
        cohort += 1;
        let subscriptions: Vec<_> = tables["subscriptions"]
            .iter()
            .filter(|row| row["account_id"] == account["id"])
            .collect();
        for (index, age) in [30, 90, 180, 365].into_iter().enumerate() {
            retained[index] += i64::from(active_mrr(&subscriptions, signup + age) > 0);
        }
    }
    assert!(cohort >= 40, "cohort too small: {cohort}");
    assert!(
        retained[0] > 0 && retained[0] < cohort,
        "signup cohort must include nonconverters: {retained:?} of {cohort}"
    );
    assert!(
        retained[1] < retained[0],
        "no early signup-cohort decline: {retained:?} of {cohort}"
    );
    assert!(retained[3] > 0, "no long-lived signup-cohort accounts");
    let early_loss = retained[0] - retained[1];
    let later_loss = retained[2] - retained[3];
    assert!(
        early_loss * 185 > later_loss * 60,
        "signup-cohort retention does not flatten: {retained:?} of {cohort}"
    );
}

fn assert_paying_customer_retention(tables: &Tables) {
    let mut first_paid: BTreeMap<&str, i64> = BTreeMap::new();
    for subscription in &tables["subscriptions"] {
        let started = day(subscription, "started_at");
        first_paid
            .entry(text(subscription, "account_id"))
            .and_modify(|first| *first = (*first).min(started))
            .or_insert(started);
    }
    let mut retained = [0_i64; 4];
    let mut cohort = 0;
    for (account, started) in first_paid {
        if started >= 365 {
            continue;
        }
        cohort += 1;
        let subscriptions: Vec<_> = tables["subscriptions"]
            .iter()
            .filter(|row| text(row, "account_id") == account)
            .collect();
        for (index, age) in [0, 60, 150, 335].into_iter().enumerate() {
            retained[index] += i64::from(active_mrr(&subscriptions, started + age) > 0);
        }
    }
    assert!(cohort >= 40, "paying cohort too small: {cohort}");
    assert_eq!(retained[0], cohort);
    assert!(
        retained[1] < retained[0],
        "no early paid-cohort decline: {retained:?}"
    );
    assert!(retained[3] > 0, "no long-lived paying customers");
    let early_loss = retained[0] - retained[1];
    let later_loss = retained[2] - retained[3];
    assert!(
        early_loss * 185 > later_loss * 60,
        "paid-cohort retention does not flatten: {retained:?}"
    );
}
