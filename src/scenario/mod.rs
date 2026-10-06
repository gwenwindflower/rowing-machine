//! The business models the engine can simulate.

use anyhow::Result;

use crate::output::{EntitySchema, Row};

pub mod ecommerce;
pub mod saas;
pub mod travel;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum ScenarioKind {
    #[default]
    Ecommerce,
    Saas,
    Travel,
}

impl ScenarioKind {
    pub const ALL: [Self; 3] = [Self::Ecommerce, Self::Saas, Self::Travel];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ecommerce => "ecommerce",
            Self::Saas => "saas",
            Self::Travel => "travel",
        }
    }

    /// The bundled theme a run uses when `--theme` is not passed.
    #[must_use]
    pub const fn default_theme(self) -> &'static str {
        match self {
            Self::Ecommerce | Self::Saas => "plain",
            Self::Travel => "airline",
        }
    }

    #[must_use]
    pub fn theme_requirements(self) -> crate::theme::ThemeRequirements {
        match self {
            Self::Ecommerce => ecommerce::Ecommerce::theme_requirements(),
            Self::Saas => saas::Saas::theme_requirements(),
            Self::Travel => travel::Travel::theme_requirements(),
        }
    }
}

/// One independently generated slice of a scenario stage, such as one market-day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorkUnit {
    pub stage: u32,
    pub indices: [u64; 2],
}

/// Rows produced by one work unit, tagged with their entity name.
pub type UnitRows = Vec<(&'static str, Row)>;

/// A simulated business: its entities and how each work unit generates rows (`sm-R025`, `sm-R030`).
pub trait Scenario: Sync {
    fn name(&self) -> &'static str;

    fn entities(&self) -> Vec<EntitySchema>;

    /// Every work unit in the run, in output order (`sm-R033`).
    fn units(&self) -> Vec<WorkUnit>;

    /// Generates one unit's rows as a pure function of the run seed and the unit.
    ///
    /// # Errors
    ///
    /// Returns an error when the unit cannot be generated from the run configuration.
    fn generate(&self, seed: u64, unit: WorkUnit) -> Result<UnitRows>;

    /// Accumulates cross-unit facts from emitted rows.
    ///
    /// # Errors
    /// Returns an error if the rows violate scenario invariants.
    fn observe(&mut self, _rows: &UnitRows) -> Result<()> {
        Ok(())
    }

    /// Fixes the state consumed by subsequent stages.
    ///
    /// # Errors
    /// Returns an error if the stage cannot be completed.
    fn complete_stage(&mut self, _stage: u32) -> Result<()> {
        Ok(())
    }
}
