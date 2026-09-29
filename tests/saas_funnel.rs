use std::collections::{BTreeMap, BTreeSet};

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

type Tables = BTreeMap<&'static str, Vec<Value>>;

fn text<'a>(row: &'a Value, column: &str) -> &'a str {
    row[column].as_str().unwrap()
}

fn number(row: &Value, column: &str) -> i64 {
    row[column].as_i64().unwrap()
}

#[test]
fn marketing_output_supports_funnel_attribution_and_monthly_paid_cac() {
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
    let tables: Tables = [
        "campaigns",
        "ad_spend",
        "touches",
        "leads",
        "accounts",
        "subscriptions",
    ]
    .into_iter()
    .map(|entity| {
        let contents =
            std::fs::read_to_string(directory.path().join(format!("raw_{entity}.jsonl")))
                .unwrap_or_else(|error| panic!("reading {entity}: {error}"));
        let rows = contents
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        (entity, rows)
    })
    .collect();
    assert_schemas(&tables);
    assert_campaigns(&tables);
    assert_attribution(&tables);
    assert_channel_metrics(&tables);
}

fn assert_schemas(tables: &Tables) {
    for (entity, columns) in [
        (
            "campaigns",
            "id channel name started_at ended_at daily_budget",
        ),
        ("ad_spend", "date campaign_id impressions clicks spend"),
        (
            "touches",
            "id visitor_id campaign_id channel occurred_at landing_page",
        ),
        (
            "leads",
            "id visitor_id name email created_at lead_source first_touch_id last_touch_id status account_id",
        ),
    ] {
        assert!(!tables[entity].is_empty(), "{entity} is empty");
        let expected: BTreeSet<_> = columns.split_whitespace().collect();
        for row in &tables[entity] {
            assert_eq!(
                row.as_object()
                    .unwrap()
                    .keys()
                    .map(String::as_str)
                    .collect::<BTreeSet<_>>(),
                expected
            );
        }
    }
    for entity in ["campaigns", "touches", "leads", "accounts"] {
        let keys: BTreeSet<_> = tables[entity].iter().map(|row| text(row, "id")).collect();
        assert_eq!(keys.len(), tables[entity].len(), "duplicate {entity} key");
        assert!(keys.iter().all(|id| !id.is_empty()));
    }
}

fn assert_campaigns(tables: &Tables) {
    let campaigns: BTreeMap<_, _> = tables["campaigns"]
        .iter()
        .map(|row| (text(row, "id"), row))
        .collect();
    let mut touches = BTreeMap::new();
    for touch in &tables["touches"] {
        if let Some(id) = touch["campaign_id"].as_str() {
            assert_eq!(touch["channel"], campaigns[id]["channel"]);
            *touches
                .entry((&text(touch, "occurred_at")[..10], id))
                .or_insert(0_i64) += 1;
        } else {
            assert!(["content", "referral", "direct"].contains(&text(touch, "channel")));
        }
    }
    let mut days = BTreeSet::new();
    for spend in &tables["ad_spend"] {
        let campaign_id = text(spend, "campaign_id");
        let date = &text(spend, "date")[..10];
        assert!(
            days.insert((date, campaign_id)),
            "duplicate campaign/day spend"
        );
        let campaign = campaigns[campaign_id];
        assert!(date >= &text(campaign, "started_at")[..10]);
        if let Some(end) = campaign["ended_at"].as_str() {
            assert!(date < &end[..10]);
        }
        let budget = number(campaign, "daily_budget");
        assert!(budget > 0);
        assert!((number(spend, "spend") - budget).abs() * 4 <= budget);
        assert!(number(spend, "clicks") >= 0);
        assert!(number(spend, "clicks") <= number(spend, "impressions"));
        assert_eq!(
            touches.remove(&(date, campaign_id)).unwrap_or(0),
            number(spend, "clicks")
        );
    }
    assert!(touches.is_empty(), "paid touches without spend");
    for campaign in &tables["campaigns"] {
        let start: jiff::civil::Date = text(campaign, "started_at")[..10].parse().unwrap();
        let end: jiff::civil::Date = campaign["ended_at"]
            .as_str()
            .map_or("2026-12-31", |s| &s[..10])
            .parse()
            .unwrap();
        let end = end.min("2026-12-31".parse().unwrap());
        let count = days
            .iter()
            .filter(|(_, id)| *id == text(campaign, "id"))
            .count();
        assert_eq!(
            i64::try_from(count).unwrap(),
            i64::from(start.until(end).unwrap().get_days())
        );
    }
}

fn assert_attribution(tables: &Tables) {
    let touches: BTreeMap<_, _> = tables["touches"]
        .iter()
        .map(|row| (text(row, "id"), row))
        .collect();
    let accounts: BTreeMap<_, _> = tables["accounts"]
        .iter()
        .map(|row| (text(row, "id"), row))
        .collect();
    let mut converted = BTreeSet::new();
    let mut multiple = 0;
    let mut attribution_changes = 0;
    for lead in &tables["leads"] {
        let first = touches[text(lead, "first_touch_id")];
        let last = touches[text(lead, "last_touch_id")];
        assert_eq!(first["visitor_id"], lead["visitor_id"]);
        assert_eq!(last["visitor_id"], lead["visitor_id"]);
        assert!(text(first, "occurred_at") <= text(last, "occurred_at"));
        assert!(text(last, "occurred_at") <= text(lead, "created_at"));
        assert_eq!(lead["lead_source"], first["channel"]);
        multiple += usize::from(first["id"] != last["id"]);
        attribution_changes += usize::from(first["channel"] != last["channel"]);
        if let Some(id) = lead["account_id"].as_str() {
            assert!(
                converted.insert(id),
                "multiple leads converted to one account"
            );
            assert!(["converted", "demo_requested"].contains(&text(lead, "status")));
            let account = accounts[id];
            assert_eq!(account["first_touch_id"], first["id"]);
            assert_eq!(account["acquisition_channel"], first["channel"]);
            assert!(text(lead, "created_at") <= text(account, "created_at"));
        } else {
            assert_ne!(text(lead, "status"), "converted");
        }
    }
    assert!(multiple * 10 > tables["leads"].len());
    assert!(attribution_changes * 10 > tables["leads"].len());
    assert_eq!(converted.len(), accounts.len());
    let paid: BTreeSet<_> = tables["subscriptions"]
        .iter()
        .map(|row| text(row, "account_id"))
        .collect();
    assert!(paid.is_subset(&converted));
    assert!(!paid.is_empty() && paid.len() < converted.len());
    assert!(converted.len() < tables["leads"].len());
    assert!(tables["leads"].len() < touches.len());
}

fn assert_channel_metrics(tables: &Tables) {
    let mut visitors_by_channel: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for touch in &tables["touches"] {
        visitors_by_channel
            .entry(text(touch, "channel"))
            .or_default()
            .insert(text(touch, "visitor_id"));
    }
    let mut lead_totals = BTreeMap::new();
    for lead in &tables["leads"] {
        let totals = lead_totals
            .entry(text(lead, "lead_source"))
            .or_insert((0_usize, 0_usize));
        totals.0 += 1;
        totals.1 += usize::from(!lead["account_id"].is_null());
    }
    let rates: Vec<_> = lead_totals
        .iter()
        .map(|(channel, &(leads, converted))| {
            (
                1000 * leads / visitors_by_channel[channel].len(),
                1000 * converted / leads,
            )
        })
        .collect();
    for index in 0..2 {
        let values: Vec<_> = rates
            .iter()
            .map(|&(leads, converted)| if index == 0 { leads } else { converted })
            .collect();
        assert!(
            values.iter().max().unwrap() - values.iter().min().unwrap() > 50,
            "channel funnel rates differ by less than five percentage points: {rates:?}"
        );
    }
    let campaigns: BTreeMap<_, _> = tables["campaigns"]
        .iter()
        .map(|row| (text(row, "id"), text(row, "channel")))
        .collect();
    let mut spend_by_month = BTreeMap::new();
    let mut channel_totals = BTreeMap::new();
    for spend in &tables["ad_spend"] {
        let channel = campaigns[text(spend, "campaign_id")];
        *spend_by_month
            .entry((channel, &text(spend, "date")[..7]))
            .or_insert(0_i64) += number(spend, "spend");
        let totals = channel_totals.entry(channel).or_insert((0_i64, 0_i64));
        totals.0 += number(spend, "spend");
        totals.1 += number(spend, "clicks");
    }
    let cpcs: BTreeSet<_> = channel_totals
        .values()
        .map(|&(spend, clicks)| spend / clicks.max(1))
        .collect();
    assert!(
        cpcs.len() >= 2,
        "paid channels have indistinguishable costs"
    );
    let mut first_paid = BTreeMap::new();
    for subscription in &tables["subscriptions"] {
        let date = text(subscription, "started_at");
        first_paid
            .entry(text(subscription, "account_id"))
            .and_modify(|current: &mut &str| *current = (*current).min(date))
            .or_insert(date);
    }
    let mut paid_by_month = BTreeMap::new();
    for account in &tables["accounts"] {
        if let Some(date) = first_paid.get(text(account, "id")) {
            *paid_by_month
                .entry((text(account, "acquisition_channel"), &date[..7]))
                .or_insert(0_i64) += 1;
        }
    }
    let mut cac_channels = BTreeSet::new();
    for ((channel, month), count) in paid_by_month {
        if let Some(spend) = spend_by_month.get(&(channel, month)) {
            assert!(*spend / count > 0, "nonpositive CAC for {channel}/{month}");
            cac_channels.insert(channel);
        }
    }
    assert!(
        cac_channels.len() >= 2,
        "paid attribution cannot compare channel CAC"
    );
    for channel in ["content", "referral", "direct"] {
        let count = |year: &str| {
            tables["touches"]
                .iter()
                .filter(|row| {
                    text(row, "channel") == channel && text(row, "occurred_at").starts_with(year)
                })
                .count()
        };
        assert!(
            count("2026") > count("2023"),
            "{channel} traffic does not grow"
        );
    }
}
