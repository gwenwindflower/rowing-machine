use std::collections::{BTreeMap, BTreeSet};

use assert_cmd::cargo::cargo_bin_cmd;
use jiff::civil::Date;
use serde_json::Value;

type Tables = BTreeMap<&'static str, Vec<Value>>;

fn text<'a>(row: &'a Value, column: &str) -> &'a str {
    row[column].as_str().unwrap()
}

fn number(row: &Value, column: &str) -> i64 {
    row[column].as_i64().unwrap()
}

fn date(row: &Value, column: &str) -> Date {
    text(row, column)[..10].parse().unwrap()
}

fn days(start: Date, end: Date) -> i64 {
    i64::from(start.until(end).unwrap().get_days())
}

#[test]
fn sales_output_supports_pipeline_rep_productivity_and_blended_payback() {
    let directory = tempfile::tempdir().unwrap();
    cargo_bin_cmd!("rowing-machine")
        .args([
            "--scenario",
            "saas",
            "--years",
            "4",
            "--scale",
            "100",
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
    let tables: Tables = [
        "accounts",
        "leads",
        "subscriptions",
        "ad_spend",
        "sales_reps",
        "opportunities",
        "opportunity_stages",
        "sales_activities",
    ]
    .into_iter()
    .map(|entity| {
        let contents =
            std::fs::read_to_string(directory.path().join(format!("raw_{entity}.jsonl")))
                .unwrap_or_else(|error| panic!("reading {entity}: {error}"));
        (
            entity,
            contents
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect(),
        )
    })
    .collect();
    assert_schemas(&tables);
    assert_pipeline(&tables);
    assert_band_routes(&tables);
    assert_rep_capacity(&tables);
    assert_payback(&tables);
}

fn assert_schemas(tables: &Tables) {
    for (entity, columns) in [
        (
            "sales_reps",
            "id name segment hired_at departed_at annual_cost",
        ),
        (
            "opportunities",
            "id account_id lead_id owner_id created_at stage amount closed_at outcome",
        ),
        ("opportunity_stages", "opportunity_id stage entered_at"),
        (
            "sales_activities",
            "id rep_id opportunity_id activity_type occurred_at",
        ),
    ] {
        assert!(!tables[entity].is_empty(), "{entity} is empty");
        let expected: BTreeSet<_> = columns.split_whitespace().collect();
        let mut keys = BTreeSet::new();
        for row in &tables[entity] {
            assert_eq!(
                row.as_object()
                    .unwrap()
                    .keys()
                    .map(String::as_str)
                    .collect::<BTreeSet<_>>(),
                expected
            );
            let key = if entity == "opportunity_stages" {
                format!("{}:{}", text(row, "opportunity_id"), text(row, "stage"))
            } else {
                text(row, "id").to_owned()
            };
            assert!(keys.insert(key), "duplicate {entity} primary key");
        }
    }
}

fn assert_pipeline(tables: &Tables) {
    let index = |entity| {
        tables[entity]
            .iter()
            .map(|row| (text(row, "id"), row))
            .collect::<BTreeMap<_, _>>()
    };
    let accounts = index("accounts");
    let leads = index("leads");
    let reps = index("sales_reps");
    let opportunities = index("opportunities");
    let mut stages: BTreeMap<&str, Vec<&Value>> = BTreeMap::new();
    for stage in &tables["opportunity_stages"] {
        assert!(opportunities.contains_key(text(stage, "opportunity_id")));
        stages
            .entry(text(stage, "opportunity_id"))
            .or_default()
            .push(stage);
    }
    let first_paid = first_paid_subscriptions(&tables["subscriptions"]);
    let mut band_metrics: BTreeMap<&str, (i64, i64, i64)> = BTreeMap::new();
    let mut won = 0;
    let mut closed = 0;
    let mut quarter_end = 0;
    let mut open = 0;
    for opportunity in &tables["opportunities"] {
        let account_id = text(opportunity, "account_id");
        let account = accounts[account_id];
        let lead = leads[text(opportunity, "lead_id")];
        let rep = reps[text(opportunity, "owner_id")];
        assert_eq!(lead["account_id"], opportunity["account_id"]);
        assert_eq!(account["employee_band"], rep["segment"]);
        let created = date(opportunity, "created_at");
        assert!(created >= date(account, "created_at"));
        assert!(created >= date(lead, "created_at"));
        assert!(created >= date(rep, "hired_at"));
        if !rep["departed_at"].is_null() {
            assert!(created < date(rep, "departed_at"));
        }
        assert_stage_history(opportunity, stages.remove(text(opportunity, "id")).unwrap());
        if opportunity["closed_at"].is_null() {
            open += 1;
            assert!(opportunity["outcome"].is_null());
            assert!(!first_paid.contains_key(account_id));
        } else {
            closed += 1;
            let close = date(opportunity, "closed_at");
            quarter_end +=
                i64::from(close.month() % 3 == 0 && close.day() > close.days_in_month() - 14);
            let metrics = band_metrics
                .entry(text(account, "employee_band"))
                .or_default();
            metrics.0 += 1;
            metrics.1 += days(created, close);
            metrics.2 += number(opportunity, "amount");
            match text(opportunity, "outcome") {
                "won" => {
                    won += 1;
                    assert_eq!(text(opportunity, "stage"), "closed_won");
                    let subscription = first_paid[account_id];
                    assert_eq!(opportunity["closed_at"], subscription["started_at"]);
                    assert_eq!(
                        number(opportunity, "amount"),
                        12 * number(subscription, "mrr")
                    );
                    assert_eq!(text(lead, "status"), "converted");
                }
                "lost" => {
                    assert_eq!(text(opportunity, "stage"), "closed_lost");
                    assert!(!first_paid.contains_key(account_id));
                }
                outcome => panic!("unknown outcome {outcome}"),
            }
        }
    }
    assert!(stages.is_empty());
    assert!(open > 0);
    assert!(
        won * 5 > closed && won * 5 < closed * 3,
        "win rate {won}/{closed}"
    );
    assert!(
        quarter_end * 4 > closed,
        "quarter-end closes {quarter_end}/{closed}"
    );
    for pair in [["small", "medium"], ["medium", "large"]] {
        let left = band_metrics[pair[0]];
        let right = band_metrics[pair[1]];
        assert!(
            left.1 * right.0 < right.1 * left.0,
            "cycle lengths: {band_metrics:?}"
        );
        assert!(
            left.2 * right.0 < right.2 * left.0,
            "deal amounts: {band_metrics:?}"
        );
    }
    assert_activities(&tables["sales_activities"], &opportunities, &reps);
}

fn assert_stage_history(opportunity: &Value, mut history: Vec<&Value>) {
    history.sort_by_key(|row| text(row, "entered_at"));
    let expected = [
        "discovery",
        "demo",
        "proposal",
        "negotiation",
        text(opportunity, "stage"),
    ];
    assert!(history.len() <= expected.len());
    for (stage, expected) in history.iter().zip(expected) {
        assert_eq!(text(stage, "stage"), expected);
    }
    assert_eq!(
        text(history[0], "entered_at"),
        text(opportunity, "created_at")
    );
    assert_eq!(history.last().unwrap()["stage"], opportunity["stage"]);
    for pair in history.windows(2) {
        assert!(text(pair[0], "entered_at") < text(pair[1], "entered_at"));
    }
    if opportunity["closed_at"].is_null() {
        assert!(history.len() <= 4);
    } else {
        assert_eq!(history.len(), 5);
        assert_eq!(history[4]["entered_at"], opportunity["closed_at"]);
    }
}

fn first_paid_subscriptions(subscriptions: &[Value]) -> BTreeMap<&str, &Value> {
    let mut first_paid: BTreeMap<&str, &Value> = BTreeMap::new();
    for subscription in subscriptions {
        first_paid
            .entry(text(subscription, "account_id"))
            .and_modify(|first| {
                if text(subscription, "started_at") < text(first, "started_at") {
                    *first = subscription;
                }
            })
            .or_insert(subscription);
    }
    first_paid
}

fn assert_activities(
    activities: &[Value],
    opportunities: &BTreeMap<&str, &Value>,
    reps: &BTreeMap<&str, &Value>,
) {
    for activity in activities {
        let opportunity = opportunities[text(activity, "opportunity_id")];
        let rep = reps[text(activity, "rep_id")];
        let occurred = date(activity, "occurred_at");
        assert!(occurred >= date(opportunity, "created_at"));
        if !opportunity["closed_at"].is_null() {
            assert!(occurred <= date(opportunity, "closed_at"));
        }
        assert!(occurred >= date(rep, "hired_at"));
        if !rep["departed_at"].is_null() {
            assert!(occurred < date(rep, "departed_at"));
        }
        assert!(!text(activity, "activity_type").is_empty());
    }
}

fn assert_rep_capacity(tables: &Tables) {
    let end: Date = "2026-12-31".parse().unwrap();
    for rep in &tables["sales_reps"] {
        let capacity = match text(rep, "segment") {
            "small" => 20,
            "medium" => 30,
            "large" => 40,
            other => panic!("unknown segment {other}"),
        };
        let opportunities: Vec<_> = tables["opportunities"]
            .iter()
            .filter(|row| row["owner_id"] == rep["id"])
            .collect();
        for opportunity in &opportunities {
            let created = date(opportunity, "created_at");
            let concurrent = opportunities
                .iter()
                .filter(|row| {
                    date(row, "created_at") <= created
                        && (row["closed_at"].is_null() || date(row, "closed_at") > created)
                })
                .count();
            assert!(
                concurrent < capacity,
                "rep {} exceeds {capacity} opportunities at {created}",
                text(rep, "id")
            );
            let tenure = usize::try_from(days(date(rep, "hired_at"), created)).unwrap();
            let ramp_capacity = capacity * (tenure + 30).min(120) / 120;
            assert!(
                concurrent < ramp_capacity,
                "rep {} exceeds ramp capacity {ramp_capacity} at tenure {tenure}",
                text(rep, "id")
            );
            assert!(created < end);
        }
    }
}

fn assert_band_routes(tables: &Tables) {
    let sales_accounts: BTreeSet<_> = tables["opportunities"]
        .iter()
        .map(|row| text(row, "account_id"))
        .collect();
    assert_eq!(sales_accounts.len(), tables["opportunities"].len());
    for band in ["small", "large"] {
        let accounts: Vec<_> = tables["accounts"]
            .iter()
            .filter(|row| text(row, "employee_band") == band)
            .collect();
        let demos = accounts
            .iter()
            .filter(|row| sales_accounts.contains(text(row, "id")))
            .count();
        if band == "small" {
            assert!(
                demos * 2 < accounts.len(),
                "small accounts mostly request demos"
            );
        } else {
            assert!(
                demos * 2 > accounts.len(),
                "large accounts mostly self-serve"
            );
        }
    }
}

fn assert_payback(tables: &Tables) {
    let start: Date = "2023-01-01".parse().unwrap();
    let end: Date = "2026-12-31".parse().unwrap();
    let mut acquisition_cost: i64 = tables["ad_spend"]
        .iter()
        .map(|row| number(row, "spend"))
        .sum();
    for rep in &tables["sales_reps"] {
        let hired = date(rep, "hired_at").max(start);
        let departed = if rep["departed_at"].is_null() {
            end
        } else {
            date(rep, "departed_at").min(end)
        };
        assert!(number(rep, "annual_cost") > 0);
        acquisition_cost += number(rep, "annual_cost") * days(hired, departed) / 365;
    }
    let first_paid = first_paid_subscriptions(&tables["subscriptions"]);
    let new_mrr: i64 = first_paid.values().map(|row| number(row, "mrr")).sum();
    assert!(new_mrr > 0);
    assert!(
        (6 * new_mrr..=36 * new_mrr).contains(&acquisition_cost),
        "payback cost={acquisition_cost}, new MRR={new_mrr}"
    );
}
