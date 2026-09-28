#[path = "../tests/support/stats.rs"]
mod stats;

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Serialize;

#[derive(Serialize)]
struct ReferenceMetric {
    value: f64,
    absolute_tolerance: f64,
    relative_tolerance: f64,
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [directory, personas, destination] = args.as_slice() else {
        bail!("usage: capture_stats OUTPUT_DIRECTORY PERSONAS_CSV FIXTURE_JSON");
    };
    let mut persona_map = BTreeMap::new();
    for row in csv::Reader::from_path(personas)?.records() {
        let row = row?;
        persona_map.insert(row[0].to_owned(), row[1].to_owned());
    }
    let statistics = stats::summarize(Path::new(directory), "raw", &persona_map)?;
    let metrics: BTreeMap<_, _> = statistics
        .into_iter()
        .map(|(name, value)| {
            let (absolute_tolerance, relative_tolerance) = tolerance(&name, value);
            (
                name,
                ReferenceMetric {
                    value,
                    absolute_tolerance,
                    relative_tolerance,
                },
            )
        })
        .collect();
    let mut json = serde_json::to_string_pretty(&metrics)?;
    json.push('\n');
    std::fs::write(destination, json).with_context(|| format!("writing {destination}"))
}

fn tolerance(name: &str, value: f64) -> (f64, f64) {
    if name.starts_with("rows.products.")
        || name.starts_with("rows.supplies.")
        || name.starts_with("rows.stores.")
    {
        (0.0, 0.0)
    } else if name.starts_with("persona_order_share.") {
        ((0.001 + value * 0.25).min(0.05), 0.0)
    } else if name == "sparrow_rate" {
        (0.05, 0.0)
    } else if name.starts_with("mean_order_total.") {
        (0.0, 0.25)
    } else if name == "mean_items_per_order" {
        (0.0, 0.10)
    } else if name.starts_with("guild_rank.") {
        (2.0, 0.05)
    } else if name.starts_with("rows.customers.") {
        (5.0, 0.10)
    } else {
        (20.0, 0.20)
    }
}
