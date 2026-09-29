use std::{
    sync::{Condvar, Mutex},
    time::Duration,
};

use anyhow::{Result, ensure};
use clap::Parser;

use super::run;
use crate::{
    cli::Cli,
    output::{Column, ColumnType, EntitySchema, Value},
    scenario::{Scenario, UnitRows, WorkUnit},
};

#[derive(Default)]
struct Generation {
    completed: Vec<WorkUnit>,
    pending: usize,
    peak_pending: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Failure {
    None,
    Generation,
    Observation,
    DuplicateKey,
}

struct OrderedScenario {
    units_per_stage: u64,
    generation: Mutex<Generation>,
    ready: Condvar,
    observed: Vec<String>,
    completed_stages: Vec<u32>,
    reverse_first: bool,
    failure: Failure,
}

impl OrderedScenario {
    fn new(units_per_stage: u64) -> Self {
        Self {
            units_per_stage,
            generation: Mutex::default(),
            ready: Condvar::new(),
            observed: Vec::new(),
            completed_stages: Vec::new(),
            reverse_first: false,
            failure: Failure::None,
        }
    }
}

impl Scenario for OrderedScenario {
    fn name(&self) -> &'static str {
        "ordered"
    }

    fn entities(&self) -> Vec<EntitySchema> {
        vec![EntitySchema {
            name: "records",
            columns: vec![Column {
                name: "id",
                column_type: ColumnType::Text,
                nullable: false,
            }],
            primary_key: vec!["id"],
        }]
    }

    fn units(&self) -> Vec<WorkUnit> {
        (0..2)
            .flat_map(|stage| {
                (0..self.units_per_stage).map(move |index| WorkUnit {
                    stage,
                    indices: [index, 0],
                })
            })
            .collect()
    }

    fn generate(&self, _seed: u64, unit: WorkUnit) -> Result<UnitRows> {
        ensure!(
            unit.stage == 0 || self.completed_stages == [0],
            "next stage started before the previous stage completed"
        );
        ensure!(
            self.failure != Failure::Generation || unit.indices[0] != 1,
            "generation failed"
        );
        let mut state = self.generation.lock().unwrap();
        if self.reverse_first && unit.stage == 0 && unit.indices[0] == 0 {
            let (guard, timeout) = self
                .ready
                .wait_timeout_while(state, Duration::from_secs(10), |state| {
                    state.completed.is_empty()
                })
                .unwrap();
            state = guard;
            ensure!(!timeout.timed_out(), "another worker did not finish a unit");
        }
        state.completed.push(unit);
        state.pending += 1;
        state.peak_pending = state.peak_pending.max(state.pending);
        self.ready.notify_all();
        Ok((0..2)
            .map(|row| {
                (
                    "records",
                    vec![Value::Text(format!(
                        "{}-{}-{row}",
                        unit.stage,
                        if self.failure == Failure::DuplicateKey {
                            0
                        } else {
                            unit.indices[0]
                        }
                    ))],
                )
            })
            .collect())
    }

    fn observe(&mut self, rows: &UnitRows) -> Result<()> {
        ensure!(self.failure != Failure::Observation, "observation failed");
        for (_, row) in rows {
            let Value::Text(id) = &row[0] else {
                anyhow::bail!("expected a text id");
            };
            self.observed.push(id.clone());
        }
        self.generation.lock().unwrap().pending -= 1;
        Ok(())
    }

    fn complete_stage(&mut self, stage: u32) -> Result<()> {
        ensure!(
            self.observed.len() as u64 == u64::from(stage + 1) * self.units_per_stage * 2,
            "stage completed before all its rows were observed"
        );
        self.completed_stages.push(stage);
        Ok(())
    }
}

#[test]
fn workers_emit_units_and_rows_in_order_and_finish_observing_each_stage() {
    let directory = tempfile::tempdir().unwrap();
    let mut config = Cli::parse_from(["rowing-machine", "--seed", "1", "--quiet"])
        .config()
        .unwrap();
    config.output_dir = directory.path().to_owned();
    config.workers = 2;
    let mut scenario = OrderedScenario::new(23);
    scenario.reverse_first = true;
    assert_eq!(run(&mut scenario, &config).unwrap()["records"], 92);
    assert_ne!(
        scenario.generation.lock().unwrap().completed[0].indices[0],
        0
    );
    let expected: Vec<_> = (0..2)
        .flat_map(|stage| {
            (0..23).flat_map(move |unit| (0..2).map(move |row| format!("{stage}-{unit}-{row}")))
        })
        .collect();
    assert_eq!(scenario.observed, expected);
    assert_eq!(scenario.completed_stages, [0, 1]);
    let csv = std::fs::read_to_string(directory.path().join("raw_records.csv")).unwrap();
    assert_eq!(csv, format!("id\n{}\n", expected.join("\n")));
}

#[test]
fn generated_units_waiting_for_observation_are_bounded_by_worker_count() {
    for workers in [1, 2, 3] {
        let directory = tempfile::tempdir().unwrap();
        let mut config = Cli::parse_from(["rowing-machine", "--seed", "1", "--quiet"])
            .config()
            .unwrap();
        config.output_dir = directory.path().to_owned();
        config.workers = workers;
        let mut scenario = OrderedScenario::new(41);
        run(&mut scenario, &config).unwrap();
        let state = scenario.generation.lock().unwrap();
        assert!(state.peak_pending <= 4 * workers);
        assert_eq!(state.pending, 0);
    }
}

#[test]
fn generation_and_observation_errors_prevent_stage_completion() {
    for fail_generation in [true, false] {
        let directory = tempfile::tempdir().unwrap();
        let mut config = Cli::parse_from(["rowing-machine", "--seed", "1", "--quiet"])
            .config()
            .unwrap();
        config.output_dir = directory.path().to_owned();
        config.workers = 2;
        let mut scenario = OrderedScenario::new(23);
        scenario.failure = if fail_generation {
            Failure::Generation
        } else {
            Failure::Observation
        };
        let error = run(&mut scenario, &config).unwrap_err();
        let expected = if fail_generation {
            "generation failed"
        } else {
            "observation failed"
        };
        assert!(format!("{error:#}").contains(expected));
        assert!(scenario.completed_stages.is_empty());
        assert!(
            scenario
                .generation
                .lock()
                .unwrap()
                .completed
                .iter()
                .all(|unit| unit.stage == 0)
        );
    }
}

#[test]
fn zero_workers_reports_a_flag_error_before_generating_rows() {
    let mut config = Cli::parse_from(["rowing-machine", "--seed", "1", "--quiet"])
        .config()
        .unwrap();
    config.workers = 0;
    let mut scenario = OrderedScenario::new(1);
    assert!(
        run(&mut scenario, &config)
            .unwrap_err()
            .to_string()
            .contains("--workers 0")
    );
    assert!(scenario.generation.lock().unwrap().completed.is_empty());
}

#[test]
fn output_errors_stop_the_run_before_completing_a_stage() {
    let directory = tempfile::tempdir().unwrap();
    let mut config = Cli::parse_from(["rowing-machine", "--seed", "1", "--quiet"])
        .config()
        .unwrap();
    config.output_dir = directory.path().to_owned();
    config.workers = 2;
    let mut scenario = OrderedScenario::new(41);
    scenario.failure = Failure::DuplicateKey;
    let error = run(&mut scenario, &config).unwrap_err();
    assert!(format!("{error:#}").contains("duplicate primary key"));
    assert!(scenario.completed_stages.is_empty());
    assert!(
        scenario
            .generation
            .lock()
            .unwrap()
            .completed
            .iter()
            .all(|unit| unit.stage == 0)
    );
}
