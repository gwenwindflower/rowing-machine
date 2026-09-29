use super::{calibrate, sample};
use crate::{
    output::EntitySchema,
    scenario::{Scenario, UnitRows, WorkUnit},
};
use anyhow::Result;

struct GrowingScenario {
    days: usize,
    dormant_days: usize,
    observed: usize,
    finished: bool,
}

impl Scenario for GrowingScenario {
    fn name(&self) -> &'static str {
        "growing"
    }

    fn entities(&self) -> Vec<EntitySchema> {
        Vec::new()
    }

    fn units(&self) -> Vec<WorkUnit> {
        (0..self.days)
            .map(|day| WorkUnit {
                stage: 0,
                indices: [day as u64, 0],
            })
            .chain(std::iter::once(WorkUnit {
                stage: 1,
                indices: [0, 0],
            }))
            .collect()
    }

    fn generate(&self, _seed: u64, unit: WorkUnit) -> Result<UnitRows> {
        if unit.stage == 1 {
            anyhow::ensure!(self.finished, "order stage must be complete");
            return Ok(vec![("customers", vec![]); usize::from(self.observed > 0)]);
        }
        let day = usize::try_from(unit.indices[0])? + 1;
        Ok(vec![
            ("orders", vec![]);
            day.saturating_sub(self.dormant_days)
        ])
    }

    fn observe(&mut self, rows: &UnitRows) -> Result<()> {
        self.observed += rows
            .iter()
            .filter(|(entity, _)| *entity == "orders")
            .count();
        Ok(())
    }

    fn complete_stage(&mut self, stage: u32) -> Result<()> {
        if stage == 0 {
            self.finished = true;
        }
        Ok(())
    }
}

fn growing(days: usize, dormant_days: usize) -> GrowingScenario {
    GrowingScenario {
        days,
        dormant_days,
        observed: 0,
        finished: false,
    }
}

#[test]
fn nonlinear_growth_is_refined_to_within_five_percent() {
    let result = calibrate(|days| Ok(growing(days, 0)), 42, "orders", 3_000, 365).unwrap();
    assert!(result.rows["orders"].abs_diff(3_000) <= 150);
    let mut scenario = growing(result.days, 0);
    assert_eq!(sample(&mut scenario, 42).unwrap(), result.rows);
    assert_eq!(result.rows["customers"], 1);
}

#[test]
fn zero_initial_samples_expand_until_the_population_activates() {
    let result = calibrate(|days| Ok(growing(days, 100)), 42, "orders", 500, 365).unwrap();
    assert!(result.days > 100);
    assert!(result.rows["orders"].abs_diff(500) <= 25);
}

#[test]
fn unattainable_targets_stop_at_the_calendar_bound() {
    let error = calibrate(|days| Ok(growing(days, 50)), 42, "orders", 1_000, 40)
        .unwrap_err()
        .to_string();
    assert!(error.contains("--target-rows 1000"));
    assert!(error.contains("calendar"));
    assert!(error.contains("smaller"));
}

#[test]
fn day_granularity_uses_the_closest_attainable_duration() {
    let result = calibrate(|days| Ok(growing(days, 0)), 42, "orders", 8, 365).unwrap();
    assert_eq!(result.days, 3);
    assert_eq!(result.rows["orders"], 6);
}

#[test]
fn zero_target_is_rejected_before_sampling() {
    let error = calibrate(
        |_| -> Result<GrowingScenario> { panic!("invalid targets cannot sample") },
        42,
        "orders",
        0,
        365,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("--target-rows 0"));
    assert!(error.contains("positive"));
}

#[test]
fn ecommerce_order_targets_reproduce_within_five_percent() {
    use crate::{engine::calendar::precompute, scenario::ecommerce::Ecommerce};
    use jiff::civil::date;

    for seed in [42, 73] {
        let factory = |days| Ecommerce::new(seed, 1, precompute(date(2023, 1, 1), days)?);
        let result = calibrate(factory, seed, "orders", 3_000, 2_000).unwrap();
        let mut scenario = factory(result.days).unwrap();
        let counts = sample(&mut scenario, seed).unwrap();
        assert!(counts["orders"].abs_diff(3_000) <= 150);
        assert_eq!(counts, result.rows);
    }
}
