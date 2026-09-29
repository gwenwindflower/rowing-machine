mod funnel;
mod lifecycle;
mod marketing;
mod schema;

use anyhow::{Context, Result, ensure};
use jiff::civil::Date;
use std::collections::BTreeMap;

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
    accounts: Vec<AccountSlot>,
    account_indices: BTreeMap<[u8; 16], usize>,
    lead_offsets: Vec<usize>,
    observed_leads: usize,
}

struct AccountSlot {
    visitor: usize,
    arrival: Option<usize>,
    user_offset: usize,
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

    /// Prepares lifecycle slots for visitors eligible to start a trial.
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
        marketing::daily_capacity(scale)?;
        let mut accounts = Vec::new();
        let mut account_indices = BTreeMap::new();
        let mut lead_offsets = Vec::with_capacity(days);
        let mut lead_count = 0;
        for day in 0..days {
            lead_offsets.push(lead_count);
            for visitor in marketing::visitors(seed, scale, day)? {
                let status = funnel::status(seed, &visitor);
                lead_count += usize::from(status.is_some());
                if status == Some("converted") && day + 1 < days {
                    let id = Stream::derive(seed, "saas.account", &[visitor.index as u64]).uuid();
                    account_indices.insert(id, accounts.len());
                    accounts.push(AccountSlot {
                        visitor: visitor.index,
                        arrival: None,
                        user_offset: 0,
                    });
                }
            }
        }
        Ok(Self {
            seed,
            start_date,
            days,
            scale,
            theme,
            accounts,
            account_indices,
            lead_offsets,
            observed_leads: 0,
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
        let marketing = marketing::generate(seed, self.scale, day, self.start_date, &self.theme)?;
        let mut rows = marketing.rows;
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
        let mut lead_index = self.lead_offsets[day];
        for visitor in marketing.visitors {
            let Some(mut status) = funnel::status(seed, &visitor) else {
                continue;
            };
            if status == "converted" && day + 1 == self.days {
                status = "qualified";
            }
            let index = visitor.index;
            let account_id = Stream::derive(seed, "saas.account", &[index as u64]).uuid();
            let name = self.theme.name(seed, "person", lead_index);
            lead_index += 1;
            rows.push((
                "leads",
                vec![
                    Value::Uuid(Stream::derive(seed, "saas.lead", &[index as u64]).uuid()),
                    Value::Uuid(visitor.id),
                    Value::Text(name),
                    Value::Text(format!("lead-{index}@prospect.example")),
                    Value::Timestamp(timestamp + 2_000_000),
                    Value::Text(visitor.channel.into()),
                    Value::Uuid(visitor.first_touch_id),
                    Value::Uuid(visitor.last_touch_id),
                    Value::Text(status.into()),
                    if status == "converted" {
                        Value::Uuid(account_id)
                    } else {
                        Value::Null
                    },
                ],
            ));
            if status != "converted" {
                continue;
            }
            let name_index = self.account_indices[&account_id];
            let mut profile = Stream::derive(seed, "saas.profile", &[index as u64]);
            let band = profile.index(3);
            rows.push((
                "accounts",
                vec![
                    Value::Uuid(account_id),
                    Value::Text(self.theme.name(seed, "organization", name_index)),
                    Value::Text(self.theme.label("industries", profile.index(6)).into()),
                    Value::Text(["small", "medium", "large"][band].into()),
                    Value::Text(self.theme.label("regions", profile.index(4)).into()),
                    Value::Timestamp(timestamp + 86_400_000_000),
                    Value::Text(visitor.channel.into()),
                    Value::Uuid(visitor.first_touch_id),
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
            .chain((0..self.accounts.len()).map(|index| WorkUnit {
                stage: 1,
                indices: [index as u64, 0],
            }))
            .collect()
    }
    fn generate(&self, seed: u64, unit: WorkUnit) -> Result<UnitRows> {
        let index = usize::try_from(unit.indices[0])?;
        match unit.stage {
            0 => self.arrival_rows(seed, index),
            1 => match self.accounts[index].arrival {
                Some(arrival) => lifecycle::generate(
                    seed,
                    self.accounts[index].visitor,
                    arrival,
                    self.start_date,
                    self.days,
                    &self.theme,
                    self.accounts[index].user_offset,
                    &self.theme.name(seed, "organization", index),
                ),
                None => Ok(Vec::new()),
            },
            _ => anyhow::bail!("unknown SaaS stage {}", unit.stage),
        }
    }
    fn observe(&mut self, rows: &UnitRows) -> Result<()> {
        for (entity, row) in rows {
            if *entity == "leads" {
                self.observed_leads += 1;
            }
            if *entity == "accounts" {
                let Value::Timestamp(timestamp) = row[5] else {
                    anyhow::bail!("account arrival requires created_at")
                };
                let date = jiff::Timestamp::from_microsecond(timestamp)?
                    .to_zoned(jiff::tz::TimeZone::UTC)
                    .date();
                let day = usize::try_from(self.start_date.until(date)?.get_days())?;
                let Value::Uuid(id) = row[0] else {
                    anyhow::bail!("account arrival requires id")
                };
                let index = self
                    .account_indices
                    .get(&id)
                    .context("account has no lifecycle slot")?;
                self.accounts[*index].arrival = Some(day);
            }
        }
        Ok(())
    }
    fn complete_stage(&mut self, stage: u32) -> Result<()> {
        if stage == 0 {
            let mut offset = self.observed_leads;
            for account in &mut self.accounts {
                account.user_offset = offset;
                if let Some(arrival) = account.arrival {
                    offset += lifecycle::user_count(
                        self.seed,
                        account.visitor,
                        arrival,
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
        for &unit in units.iter().filter(|unit| unit.stage == 1) {
            assert!(scenario.generate(42, unit).unwrap().is_empty());
        }
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
