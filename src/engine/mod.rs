pub mod calendar;
pub mod stream;

use std::{collections::BTreeMap, path::PathBuf, time::Instant};

use anyhow::{Context, Result, ensure};
use indicatif::ProgressBar;
use jiff::civil::Date;

use crate::{output::OutputSink, scenario::Scenario};

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub days: usize,
    pub scale: usize,
    pub seed: u64,
    pub start_date: Date,
    pub output_dir: PathBuf,
    pub prefix: String,
    pub quiet: bool,
    pub format: crate::output::Format,
    pub compress: bool,
}

/// Streams a scenario's ordered stages to its entity writers.
///
/// # Errors
/// Returns an error for unordered units, failed generation, or output failures.
pub fn run(scenario: &mut dyn Scenario, config: &RunConfig) -> Result<BTreeMap<String, u64>> {
    let started = Instant::now();
    let units = scenario.units();
    ensure!(
        units.windows(2).all(|pair| pair[0] < pair[1]),
        "scenario {} must declare unique work units in stage and index order",
        scenario.name()
    );
    let mut sink = OutputSink::with_options(
        &config.output_dir,
        &config.prefix,
        scenario.entities(),
        config.format,
        config.compress,
    )
    .with_context(|| {
        format!(
            "--output-dir {}: could not prepare output; choose a writable directory",
            config.output_dir.display()
        )
    })?;
    let progress = if config.quiet {
        ProgressBar::hidden()
    } else {
        println!("Seed: {}", config.seed);
        ProgressBar::new(units.len() as u64)
    };
    let mut stage = None;
    let mut remaining_units = BTreeMap::<u32, usize>::new();
    for unit in &units {
        *remaining_units.entry(unit.stage).or_default() += 1;
    }
    for unit in units {
        if let Some(previous) = stage
            && previous != unit.stage
        {
            scenario.complete_stage(previous)?;
        }
        stage = Some(unit.stage);
        let rows = scenario
            .generate(config.seed, unit)
            .with_context(|| format!("generating {} unit {unit:?}", scenario.name()))?;
        let mut unit_counts = BTreeMap::<&str, usize>::new();
        for (entity, _) in &rows {
            *unit_counts.entry(entity).or_default() += 1;
        }
        for (entity, count) in unit_counts {
            sink.estimate_rows(entity, count.saturating_mul(remaining_units[&unit.stage]));
        }
        *remaining_units.entry(unit.stage).or_default() -= 1;
        for (entity, row) in &rows {
            sink.write(entity, row).with_context(|| format!("--output-dir {}: writing {entity}; check directory permissions and available space", config.output_dir.display()))?;
        }
        scenario.observe(&rows)?;
        progress.inc(1);
    }
    if let Some(stage) = stage {
        scenario.complete_stage(stage)?;
    }
    let counts = sink.finish().context("flushing generated entity files")?;
    progress.finish_and_clear();
    if !config.quiet {
        for (entity, count) in &counts {
            println!("{entity}: {count}");
        }
        println!("Elapsed: {:.2}s", started.elapsed().as_secs_f64());
    }
    Ok(counts)
}
