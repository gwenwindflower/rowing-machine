mod support;

use std::collections::BTreeMap;

use assert_cmd::cargo::cargo_bin_cmd;
use jiff::civil::date;
use rowing_machine::{engine::calendar::precompute, scenario::ecommerce::Ecommerce};
use serde::Deserialize;

#[derive(Deserialize)]
struct ReferenceMetric {
    value: f64,
    absolute_tolerance: f64,
    relative_tolerance: f64,
}

#[test]
fn ecommerce_distribution_matches_the_go_reference() {
    let output = tempfile::tempdir().unwrap();
    cargo_bin_cmd!("rowing-machine")
        .args([
            "--seed",
            "42",
            "--scale",
            "10",
            "--years",
            "4",
            "--quiet",
            "--output-dir",
        ])
        .arg(output.path())
        .assert()
        .success();
    let scenario = Ecommerce::new(42, 10, precompute(date(2023, 1, 1), 4 * 365).unwrap()).unwrap();
    let actual =
        support::stats::summarize(output.path(), "raw", &scenario.customer_personas()).unwrap();
    let reference: BTreeMap<String, ReferenceMetric> =
        serde_json::from_str(include_str!("fixtures/go-reference-stats.json")).unwrap();
    let mut failures = Vec::new();
    for (name, metric) in &reference {
        let value = actual.get(name).copied().unwrap_or_default();
        let difference = (value - metric.value).abs();
        let tolerance = metric.absolute_tolerance + metric.relative_tolerance * metric.value.abs();
        if !value.is_finite() || difference > tolerance {
            failures.push(format!(
                "{name}: expected {} ± {tolerance}, got {value}",
                metric.value
            ));
        }
    }
    for (name, value) in &actual {
        if !reference.contains_key(name) && value.abs() > f64::EPSILON {
            failures.push(format!("unexpected metric {name}: {value}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
