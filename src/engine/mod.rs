pub mod calendar;
pub mod calibration;
pub mod stream;

#[cfg(test)]
mod tests;

use std::{collections::BTreeMap, path::PathBuf, time::Instant};

use anyhow::{Context, Result, ensure};
use indicatif::ProgressBar;
use jiff::civil::Date;
use rayon::prelude::*;

use crate::{
    output::{OutputSink, UnitEncoder},
    scenario::Scenario,
};

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub days: usize,
    pub target_rows: Option<usize>,
    pub scale: usize,
    pub seed: u64,
    pub start_date: Date,
    pub output_dir: PathBuf,
    pub prefix: String,
    pub quiet: bool,
    pub format: crate::output::Format,
    pub compress: bool,
    pub workers: usize,
}

/// Streams a scenario's ordered stages to its entity writers.
///
/// # Errors
/// Returns an error for unordered units, failed generation, or output failures.
pub fn run(scenario: &mut dyn Scenario, config: &RunConfig) -> Result<BTreeMap<String, u64>> {
    let started = Instant::now();
    ensure!(
        config.workers > 0,
        "--workers 0: use a positive integer greater than zero"
    );
    let pool = if config.workers == 1 {
        None
    } else {
        Some(
            rayon::ThreadPoolBuilder::new()
                .num_threads(config.workers)
                .build()
                .with_context(|| {
                    format!(
                        "--workers {}: could not start worker threads; use fewer workers",
                        config.workers
                    )
                })?,
        )
    };
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
    let encoder = UnitEncoder::new(scenario.entities(), config.format);
    let batch_size = if pool.is_some() {
        config.workers.saturating_mul(4)
    } else {
        1
    };
    for stage_units in units.chunk_by(|left, right| left.stage == right.stage) {
        let mut remaining_units = stage_units.len();
        for batch in stage_units.chunks(batch_size) {
            let generate = |unit: &crate::scenario::WorkUnit| {
                let rows = scenario
                    .generate(config.seed, *unit)
                    .with_context(|| format!("generating {} unit {unit:?}", scenario.name()))?;
                let prepared = encoder.encode(&rows)?;
                Ok::<_, anyhow::Error>((rows, prepared))
            };
            let results: Vec<_> = if let Some(pool) = &pool {
                pool.install(|| batch.par_iter().map(generate).collect())
            } else {
                batch.iter().map(generate).collect()
            };
            for rows in results {
                let (rows, prepared) = rows?;
                sink.write_unit(prepared, remaining_units).with_context(|| format!("--output-dir {}: writing output; check directory permissions and available space", config.output_dir.display()))?;
                remaining_units -= 1;
                scenario.observe(&rows)?;
                progress.inc(1);
            }
        }
        scenario.complete_stage(stage_units[0].stage)?;
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
