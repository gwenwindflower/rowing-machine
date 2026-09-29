use std::collections::{BTreeMap, BTreeSet};

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

fn generate(scale: &str) -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    cargo_bin_cmd!("rowing-machine")
        .args([
            "--scenario",
            "saas",
            "--years",
            "1",
            "--start-date",
            "2023-01-01",
            "--scale",
            scale,
            "--seed",
            "42",
            "--quiet",
            "--format",
            "jsonl",
            "--output-dir",
        ])
        .arg(directory.path())
        .assert()
        .success();
    directory
}

fn rows(directory: &std::path::Path, entity: &str) -> Vec<Value> {
    std::fs::read_to_string(directory.join(format!("raw_{entity}.jsonl")))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn seconds(value: &Value) -> i64 {
    format!("{}Z", value.as_str().unwrap())
        .parse::<jiff::Timestamp>()
        .unwrap()
        .as_second()
}

#[test]
fn usage_preserves_relationships_time_bounds_and_activation_markers() {
    let directory = generate("4");
    let users = rows(directory.path(), "users");
    let users: BTreeMap<_, _> = users
        .iter()
        .map(|row| (row["id"].as_str().unwrap(), row))
        .collect();
    let accounts = rows(directory.path(), "accounts");
    let accounts: BTreeSet<_> = accounts
        .iter()
        .map(|row| row["id"].as_str().unwrap())
        .collect();
    let sessions = rows(directory.path(), "sessions");
    let run_end = seconds(&Value::String("2024-01-01T00:00:00".into()));
    assert!(!sessions.is_empty());
    let session_index: BTreeMap<_, _> = sessions
        .iter()
        .map(|row| (row["id"].as_str().unwrap(), row))
        .collect();
    assert_eq!(session_index.len(), sessions.len());
    for session in &sessions {
        let user = users[session["user_id"].as_str().unwrap()];
        assert_eq!(session["account_id"], user["account_id"]);
        assert!(accounts.contains(session["account_id"].as_str().unwrap()));
        assert!(seconds(&session["started_at"]) >= seconds(&user["created_at"]));
        assert!(seconds(&session["ended_at"]) >= seconds(&session["started_at"]));
        assert!(seconds(&session["ended_at"]) < run_end);
        assert!(!session["device"].as_str().unwrap().is_empty());
    }
    let theme = rowing_machine::theme::Theme::load("plain").unwrap();
    let features: BTreeSet<_> = (0..16)
        .map(|index| theme.name(42, "feature", index))
        .collect();
    let mut event_ids = BTreeSet::new();
    let mut populated = BTreeSet::new();
    let mut activated = BTreeMap::<&str, usize>::new();
    let events = rows(directory.path(), "events");
    for event in &events {
        assert!(event_ids.insert(event["id"].as_str().unwrap()));
        let session_id = event["session_id"].as_str().unwrap();
        let session = session_index[session_id];
        populated.insert(session_id);
        assert_eq!(event["user_id"], session["user_id"]);
        assert_eq!(event["account_id"], session["account_id"]);
        assert!(
            (seconds(&session["started_at"])..=seconds(&session["ended_at"]))
                .contains(&seconds(&event["occurred_at"]))
        );
        let feature = event["feature"].as_str().unwrap();
        assert!(features.contains(feature));
        let name = event["event_name"].as_str().unwrap();
        if name == format!("{feature}:activated") {
            let user_id = event["user_id"].as_str().unwrap();
            assert_eq!(event["occurred_at"], users[user_id]["activated_at"]);
            *activated.entry(user_id).or_default() += 1;
        } else {
            assert_eq!(name, format!("{feature}:used"));
        }
    }
    assert_eq!(populated.len(), sessions.len());
    assert!(!activated.is_empty());
    for (id, user) in users {
        assert_eq!(
            activated.get(id).copied().unwrap_or(0),
            usize::from(!user["activated_at"].is_null())
        );
    }
    for entry in std::fs::read_dir(directory.path()).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() != "raw_events.jsonl" {
            let count = std::fs::read_to_string(entry.path())
                .unwrap()
                .lines()
                .count();
            assert!(
                events.len() > count,
                "events must outnumber {:?}",
                entry.file_name()
            );
        }
    }
}

#[test]
fn emitted_features_reflect_account_tiers_and_user_roles() {
    let directory = generate("4");
    let theme = rowing_machine::theme::Theme::load("plain").unwrap();
    let features: BTreeMap<_, _> = (0..16)
        .map(|index| (theme.name(42, "feature", index), index))
        .collect();
    let plans = rows(directory.path(), "plans");
    let plans: BTreeMap<_, _> = plans
        .iter()
        .map(|plan| {
            let tier = ["starter", "growth", "enterprise"]
                .iter()
                .position(|tier| plan["tier"] == *tier)
                .unwrap();
            (plan["id"].as_str().unwrap(), tier)
        })
        .collect();
    let subscriptions = rows(directory.path(), "subscriptions");
    let tiers: BTreeMap<_, _> = subscriptions
        .iter()
        .map(|subscription| {
            (
                subscription["account_id"].as_str().unwrap(),
                plans[subscription["plan_id"].as_str().unwrap()],
            )
        })
        .collect();
    let users = rows(directory.path(), "users");
    let roles: BTreeMap<_, _> = users
        .iter()
        .map(|user| {
            let role = ["admin", "editor", "viewer"]
                .iter()
                .position(|role| user["role"] == *role)
                .unwrap();
            (user["id"].as_str().unwrap(), role)
        })
        .collect();
    let mut tier_adoption = [[0_usize; 3]; 3];
    let mut role_adoption = [[0_usize; 3]; 3];
    for event in rows(directory.path(), "events") {
        if event["event_name"]
            .as_str()
            .unwrap()
            .ends_with(":activated")
        {
            continue;
        }
        let feature = features[event["feature"].as_str().unwrap()];
        let role = roles[event["user_id"].as_str().unwrap()];
        if let Some(&tier) = tiers.get(event["account_id"].as_str().unwrap()) {
            tier_adoption[tier][feature % 3] += 1;
        }
        role_adoption[role][(feature / 3) % 3] += 1;
    }
    for (dimension, counts) in [("tier", tier_adoption), ("role", role_adoption)] {
        for (category, counts) in counts.iter().enumerate() {
            let total: usize = counts.iter().sum();
            assert!(
                total > 100,
                "fixture needs events for {dimension} {category}"
            );
            assert!(
                counts[category] * 2 > total,
                "{dimension} {category} must favor its feature group: {counts:?}"
            );
        }
    }
}

#[test]
fn sessions_favor_weekdays_and_regional_business_hours() {
    let directory = generate("4");
    let accounts = rows(directory.path(), "accounts");
    let offsets: BTreeMap<_, _> = accounts
        .iter()
        .map(|account| {
            let offset = match account["region"].as_str().unwrap() {
                "North America" => -5,
                "Europe" => 1,
                "Asia Pacific" => 9,
                "Latin America" => -3,
                region => panic!("unknown region {region}"),
            };
            (account["id"].as_str().unwrap(), offset)
        })
        .collect();
    let sessions = rows(directory.path(), "sessions");
    let mut weekdays = 0;
    let mut business_hours = 0;
    let mut holidays = 0;
    let mut december = 0;
    for session in &sessions {
        let local = seconds(&session["started_at"])
            + offsets[session["account_id"].as_str().unwrap()] * 3600;
        weekdays += usize::from((local.div_euclid(86400) + 3).rem_euclid(7) < 5);
        business_hours += usize::from((9..17).contains(&(local.rem_euclid(86400) / 3600)));
        let date = jiff::Timestamp::from_second(local).unwrap().to_string();
        holidays += usize::from(&date[5..10] >= "12-24" || &date[5..10] == "01-01");
        december += usize::from(&date[5..10] >= "12-01" && &date[5..10] < "12-24");
    }
    assert!(weekdays * 100 > sessions.len() * 80);
    assert!(business_hours * 100 > sessions.len() * 80);
    assert!(holidays * 23 < december * 9, "holiday daily usage must dip");
}

#[test]
fn event_volume_grows_with_scale() {
    let counts: Vec<_> = ["4", "8"]
        .into_iter()
        .map(|scale| {
            let directory = generate(scale);
            rows(directory.path(), "events").len()
        })
        .collect();
    assert!(counts[1] * 10 > counts[0] * 13, "{counts:?}");
    assert!(counts[1] * 10 < counts[0] * 30, "{counts:?}");
}

#[test]
fn engagement_falls_in_the_weeks_before_churn() {
    let directory = generate("8");
    let movements = rows(directory.path(), "mrr_movements");
    let churns: Vec<_> = movements
        .iter()
        .filter(|movement| movement["movement_type"] == "churn")
        .filter_map(|movement| {
            let account = movement["account_id"].as_str().unwrap();
            let churn = seconds(&movement["occurred_at"]);
            let paid_start = movements
                .iter()
                .filter(|row| row["account_id"] == account)
                .filter(|row| {
                    row["movement_type"] == "new" || row["movement_type"] == "reactivation"
                })
                .map(|row| seconds(&row["occurred_at"]))
                .filter(|&at| at < churn)
                .max()
                .unwrap();
            (churn - paid_start >= 56 * 86400).then_some((account, churn))
        })
        .collect();
    assert!(
        !churns.is_empty(),
        "fixture needs accounts that churn after 56 continuous paid days"
    );
    let sessions = rows(directory.path(), "sessions");
    let mut earlier = 0;
    let mut preceding = 0;
    for (account, churn) in churns {
        for session in sessions
            .iter()
            .filter(|session| session["account_id"] == account)
        {
            let days_before = (churn - seconds(&session["started_at"])).div_euclid(86400);
            earlier += usize::from((28..56).contains(&days_before));
            preceding += usize::from((0..28).contains(&days_before));
        }
    }
    assert!(earlier > 0);
    assert!(
        preceding * 10 < earlier * 8,
        "sessions before churn: {preceding}; prior window: {earlier}"
    );
}
