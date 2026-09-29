mod lifecycle;
mod marketing;
mod schema;

use anyhow::{Context, Result, ensure};
use jiff::civil::Date;

use super::{Scenario, UnitRows, WorkUnit};
use crate::engine::stream::Stream;
use crate::output::{EntitySchema, Value};
use crate::theme::{Theme, ThemeRequirements};

pub struct Saas {
    seed: u64,
    start_date: Date,
    days: usize,
    scale: usize,
    theme: Theme,
    arrivals_by_day: Vec<Vec<usize>>,
    arrivals: Vec<Option<usize>>,
    name_indices: Vec<usize>,
    user_offsets: Vec<usize>,
}

impl Saas {
    #[must_use]
    pub fn theme_requirements() -> ThemeRequirements {
        ThemeRequirements {
            name_kinds: &["person", "organization", "plan", "feature", "campaign"],
            label_sets: &[
                ("industries", 6),
                ("roles", 3),
                ("regions", 4),
                ("plan_tiers", 3),
            ],
        }
    }

    /// Prepares day-indexed arrivals and account lifecycle slots.
    ///
    /// # Errors
    /// Rejects incompatible themes, empty runs, and overflowing populations.
    pub fn with_theme(
        seed: u64,
        scale: usize,
        start_date: Date,
        days: usize,
        theme: Theme,
    ) -> Result<Self> {
        theme.validate(&Self::theme_requirements())?;
        ensure!(days > 0, "saas requires at least one simulation day");
        let population = scale
            .checked_mul(20)
            .context("--scale exceeds the SaaS population limit; use a smaller value")?;
        let mut arrivals = Vec::new();
        arrivals
            .try_reserve_exact(population)
            .context("--scale cannot fit the account population in memory; use a smaller value")?;
        arrivals.resize(population, None);
        let mut arrivals_by_day = vec![Vec::new(); days];
        let mut name_indices = vec![0; population];
        let mut arrived = 0;
        for (index, name_index) in name_indices.iter_mut().enumerate() {
            let day = Stream::derive(seed, "saas.arrival", &[index as u64]).index(1460);
            if day < days {
                arrivals_by_day[day].push(index);
                *name_index = arrived;
                arrived += 1;
            }
        }
        Ok(Self {
            seed,
            start_date,
            days,
            scale,
            theme,
            arrivals_by_day,
            arrivals,
            name_indices,
            user_offsets: vec![0; population],
        })
    }

    fn arrival_rows(&self, seed: u64, day: usize) -> Result<UnitRows> {
        let timestamp = self
            .start_date
            .checked_add(jiff::Span::new().days(i64::try_from(day)?))?
            .at(0, 0, 0, 0)
            .to_zoned(jiff::tz::TimeZone::UTC)?
            .timestamp()
            .as_microsecond();
        let mut rows =
            marketing::generate(seed, self.scale, day, self.start_date, &self.theme)?.rows;
        if day == 0 {
            for index in 0..3 {
                rows.push((
                    "plans",
                    vec![
                        Value::Uuid(Stream::derive(seed, "saas.plan", &[index as u64]).uuid()),
                        Value::Text(self.theme.name(seed, "plan", index)),
                        Value::Text(self.theme.label("plan_tiers", index).into()),
                        Value::Cents([1500, 3000, 6000][index]),
                        Value::Cents([15000, 30000, 60000][index]),
                        Value::Integer([1, 5, 10][index]),
                    ],
                ));
            }
        }
        for &index in &self.arrivals_by_day[day] {
            let mut profile = Stream::derive(seed, "saas.profile", &[index as u64]);
            let band = profile.index(3);
            rows.push((
                "accounts",
                vec![
                    Value::Uuid(Stream::derive(seed, "saas.account", &[index as u64]).uuid()),
                    Value::Text(
                        self.theme
                            .name(seed, "organization", self.name_indices[index]),
                    ),
                    Value::Text(self.theme.label("industries", profile.index(6)).into()),
                    Value::Text(["small", "medium", "large"][band].into()),
                    Value::Text(self.theme.label("regions", profile.index(4)).into()),
                    Value::Timestamp(timestamp),
                    Value::Text("direct".into()),
                    Value::Null,
                ],
            ));
        }
        Ok(rows)
    }
}

impl Scenario for Saas {
    fn name(&self) -> &'static str {
        "saas"
    }
    fn entities(&self) -> Vec<EntitySchema> {
        schema::entities()
    }
    fn units(&self) -> Vec<WorkUnit> {
        (0..self.days)
            .map(|day| WorkUnit {
                stage: 0,
                indices: [day as u64, 0],
            })
            .chain((0..self.arrivals.len()).map(|index| WorkUnit {
                stage: 1,
                indices: [index as u64, 0],
            }))
            .collect()
    }
    fn generate(&self, seed: u64, unit: WorkUnit) -> Result<UnitRows> {
        let index = usize::try_from(unit.indices[0])?;
        match unit.stage {
            0 => self.arrival_rows(seed, index),
            1 => match self.arrivals[index] {
                Some(arrival) => lifecycle::generate(
                    seed,
                    index,
                    arrival,
                    self.start_date,
                    self.days,
                    &self.theme,
                    self.user_offsets[index],
                    &self
                        .theme
                        .name(seed, "organization", self.name_indices[index]),
                ),
                None => Ok(Vec::new()),
            },
            _ => anyhow::bail!("unknown SaaS stage {}", unit.stage),
        }
    }
    fn observe(&mut self, rows: &UnitRows) -> Result<()> {
        for (entity, row) in rows {
            if *entity == "accounts" {
                let Value::Timestamp(timestamp) = row[5] else {
                    anyhow::bail!("account arrival requires created_at")
                };
                let date = jiff::Timestamp::from_microsecond(timestamp)?
                    .to_zoned(jiff::tz::TimeZone::UTC)
                    .date();
                let day = usize::try_from(self.start_date.until(date)?.get_days())?;
                for &index in &self.arrivals_by_day[day] {
                    self.arrivals[index] = Some(day);
                }
            }
        }
        Ok(())
    }
    fn complete_stage(&mut self, stage: u32) -> Result<()> {
        if stage == 0 {
            let mut offset = 0;
            for (index, arrival) in self.arrivals.iter().enumerate() {
                self.user_offsets[index] = offset;
                if let Some(arrival) = arrival {
                    offset += lifecycle::user_count(
                        self.seed,
                        index,
                        *arrival,
                        self.start_date,
                        self.days,
                    )?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_units_read_completed_arrivals_and_repeat_in_any_generation_order() {
        let mut scenario = Saas::with_theme(
            42,
            4,
            "2023-01-01".parse().unwrap(),
            365,
            Theme::load("plain").unwrap(),
        )
        .unwrap();
        let units = scenario.units();
        for &unit in units.iter().filter(|unit| unit.stage == 0) {
            let rows = scenario.generate(42, unit).unwrap();
            scenario.observe(&rows).unwrap();
        }
        scenario.complete_stage(0).unwrap();
        let expected: Vec<_> = units
            .iter()
            .filter(|unit| unit.stage == 1)
            .map(|&unit| (unit, scenario.generate(42, unit).unwrap()))
            .collect();
        assert!(
            expected
                .iter()
                .any(|(_, rows)| rows.iter().any(|(entity, _)| *entity == "subscriptions"))
        );
        for (unit, rows) in expected.into_iter().rev() {
            assert_eq!(scenario.generate(42, unit).unwrap(), rows);
        }
    }
}
