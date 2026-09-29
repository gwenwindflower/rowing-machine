use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::scenario::Scenario;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct Calibration {
    pub days: usize,
    pub rows: BTreeMap<String, u64>,
}

/// Counts a scenario's output while preserving observation and stage completion.
///
/// # Errors
/// Returns an error for unordered work units or failed scenario generation.
pub fn sample(scenario: &mut dyn Scenario, seed: u64) -> Result<BTreeMap<String, u64>> {
    let units = scenario.units();
    ensure!(
        units.windows(2).all(|pair| pair[0] < pair[1]),
        "scenario {} must declare unique work units in stage and index order",
        scenario.name()
    );
    let mut counts: BTreeMap<String, u64> = scenario
        .entities()
        .into_iter()
        .map(|entity| (entity.name.to_owned(), 0))
        .collect();
    let mut stage = None;
    for unit in units {
        if let Some(previous) = stage
            && previous != unit.stage
        {
            scenario.complete_stage(previous)?;
        }
        stage = Some(unit.stage);
        let rows = scenario
            .generate(seed, unit)
            .with_context(|| format!("sampling {} unit {unit:?}", scenario.name()))?;
        for (entity, _) in &rows {
            *counts.entry((*entity).to_owned()).or_default() += 1;
        }
        scenario.observe(&rows)?;
    }
    if let Some(stage) = stage {
        scenario.complete_stage(stage)?;
    }
    Ok(counts)
}

/// Samples durations to reach a target within 5%, or the closest whole day.
///
/// The factory must produce deterministic scenarios whose calibration entity's
/// cumulative row count never decreases as duration increases. Every sample
/// starts with fresh scenario state. Durations are bounded by `max_days`.
///
/// # Errors
/// Returns an error for zero targets, invalid bounds, failed samples, or targets
/// beyond the supported calendar.
pub fn calibrate<S: Scenario>(
    mut factory: impl FnMut(usize) -> Result<S>,
    seed: u64,
    entity: &str,
    target: u64,
    max_days: usize,
) -> Result<Calibration> {
    ensure!(
        target > 0,
        "--target-rows {target} must be positive; use at least 1"
    );
    ensure!(
        max_days > 0,
        "--target-rows {target} has no supported calendar days; use an earlier --start-date"
    );
    let mut days = 30.min(max_days);
    let mut lower_days = 0;
    let mut lower_count = 0;
    let mut upper: Option<(usize, u64)> = None;
    let mut best: Option<Calibration> = None;
    loop {
        let mut scenario = factory(days).with_context(|| {
            format!("--target-rows {target}: preparing {days}-day calibration sample")
        })?;
        let rows = sample(&mut scenario, seed).with_context(|| {
            format!("--target-rows {target}: counting {days}-day calibration sample")
        })?;
        let count = rows.get(entity).copied().unwrap_or(0);
        let candidate = Calibration { days, rows };
        if best.as_ref().is_none_or(|previous| {
            (count.abs_diff(target), days)
                < (
                    previous
                        .rows
                        .get(entity)
                        .copied()
                        .unwrap_or(0)
                        .abs_diff(target),
                    previous.days,
                )
        }) {
            best = Some(candidate);
        }
        if count.abs_diff(target) <= target / 20 {
            break;
        }
        if count < target {
            ensure!(
                days < max_days,
                "--target-rows {target} exceeds the {count} {entity} rows attainable within the supported calendar; use a smaller target or an earlier --start-date"
            );
            lower_days = days;
            lower_count = count;
        } else {
            upper = Some((days, count));
        }
        if let Some((upper_days, upper_count)) = upper {
            let width = upper_days - lower_days;
            if width <= 1 {
                break;
            }
            let offset = (width as u128 * u128::from(target - lower_count))
                / u128::from(upper_count - lower_count);
            let margin = (width / 10).max(1);
            let offset = usize::try_from(offset).unwrap_or(width);
            days = lower_days + offset.clamp(margin, width - margin);
        } else {
            let estimate = if count == 0 {
                days.saturating_mul(4)
            } else {
                usize::try_from(days as u128 * u128::from(target) / u128::from(count))
                    .unwrap_or(max_days)
            };
            days = estimate
                .clamp(days + 1, days.saturating_mul(4))
                .min(max_days);
        }
    }
    best.context("calibration did not sample a duration")
}
