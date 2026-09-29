//! Declarative naming packs, independent of scenario behavior.

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

pub mod names;

use names::{GeneratorConfig, NameGenerator};

const BUNDLED: &[(&str, &str)] = &[
    ("plain", include_str!("../../themes/plain.toml")),
    ("fantasy_rpg", include_str!("../../themes/fantasy_rpg.toml")),
];

#[derive(Debug, Clone, Copy)]
pub struct ThemeRequirements {
    pub name_kinds: &'static [&'static str],
    pub label_sets: &'static [(&'static str, usize)],
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ThemeConfig {
    name: String,
    description: String,
    #[serde(default)]
    names: BTreeMap<String, GeneratorConfig>,
    #[serde(default)]
    labels: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub description: String,
    source: String,
    names: BTreeMap<String, NameGenerator>,
    labels: BTreeMap<String, Vec<String>>,
}

impl Theme {
    /// Loads a bundled name or a UTF-8 TOML path.
    ///
    /// # Errors
    /// Names the selected file and invalid field when loading fails.
    pub fn load(selector: &str) -> Result<Self> {
        if let Some((_, contents)) = BUNDLED.iter().find(|(name, _)| *name == selector) {
            return Self::from_toml(&format!("themes/{selector}.toml"), contents);
        }
        let contents = std::fs::read_to_string(selector).with_context(|| {
            format!("--theme {selector:?}: cannot read theme; use plain, fantasy_rpg, or a readable TOML path")
        })?;
        Self::from_toml(selector, &contents)
            .with_context(|| format!("--theme {selector:?}: fix the theme file"))
    }

    /// Parses and validates the generators and label values in a theme.
    ///
    /// # Errors
    /// Returns the source and field for invalid TOML or naming data.
    pub fn from_toml(source: &str, contents: &str) -> Result<Self> {
        let config: ThemeConfig = toml::from_str(contents)
            .with_context(|| format!("theme file {source}: invalid TOML entry"))?;
        ensure!(
            !config.name.trim().is_empty(),
            "theme file {source}: name must not be empty"
        );
        ensure!(
            !config.description.trim().is_empty(),
            "theme file {source}: description must not be empty"
        );
        let mut names = BTreeMap::new();
        for (kind, generator) in config.names {
            let compiled = NameGenerator::compile(generator)
                .with_context(|| format!("theme file {source} ({}): names.{kind}", config.name))?;
            names.insert(kind, compiled);
        }
        for (set, values) in &config.labels {
            ensure!(
                !values.is_empty(),
                "theme file {source}: labels.{set} must not be empty"
            );
            for (index, value) in values.iter().enumerate() {
                ensure!(
                    !value.trim().is_empty(),
                    "theme file {source}: labels.{set}[{index}] must not be empty"
                );
            }
        }
        Ok(Self {
            name: config.name,
            description: config.description,
            source: source.to_owned(),
            names,
            labels: config.labels,
        })
    }

    /// Checks coverage and ordered label lengths for a scenario.
    ///
    /// # Errors
    /// Lists missing entries, invalid lengths, and compatible bundled themes.
    pub fn validate(&self, requirements: &ThemeRequirements) -> Result<()> {
        let issues = self.issues(requirements);
        if issues.is_empty() {
            return Ok(());
        }
        let compatible = Self::bundled()?
            .into_iter()
            .filter(|theme| theme.is_compatible(requirements))
            .map(|theme| theme.name)
            .collect::<Vec<_>>()
            .join(", ");
        anyhow::bail!(
            "theme file {} ({}): {}; compatible themes: {}; provide the required entries and lengths",
            self.source,
            self.name,
            issues.join("; "),
            compatible
        );
    }

    #[must_use]
    pub fn is_compatible(&self, requirements: &ThemeRequirements) -> bool {
        self.issues(requirements).is_empty()
    }

    fn issues(&self, requirements: &ThemeRequirements) -> Vec<String> {
        let mut issues = Vec::new();
        for kind in requirements.name_kinds {
            if !self.names.contains_key(*kind) {
                issues.push(format!("missing names.{kind}"));
            }
        }
        for (set, length) in requirements.label_sets {
            match self.labels.get(*set) {
                None => issues.push(format!("missing labels.{set}")),
                Some(values) if values.len() != *length => issues.push(format!(
                    "labels.{set} has {} values; expected {length}",
                    values.len()
                )),
                Some(_) => {}
            }
        }
        issues
    }

    /// Returns the bundled registry in display order.
    ///
    /// # Errors
    /// Returns an error if a compiled-in theme is invalid.
    pub fn bundled() -> Result<Vec<Self>> {
        BUNDLED.iter().map(|(name, _)| Self::load(name)).collect()
    }

    /// Generates a name for a declared kind after compatibility validation.
    ///
    /// # Panics
    /// Panics if the scenario did not validate its required kind.
    #[must_use]
    pub fn name(&self, seed: u64, kind: &str, index: usize) -> String {
        self.names[kind].generate(seed, kind, index)
    }

    /// Returns the number of distinct names for a declared kind.
    ///
    /// # Panics
    /// Panics if the theme does not declare the kind.
    #[must_use]
    pub fn capacity(&self, kind: &str) -> usize {
        self.names[kind].capacity()
    }

    /// Looks up an ordered label after compatibility validation.
    ///
    /// # Panics
    /// Panics if the scenario did not validate the set and its length.
    #[must_use]
    pub fn label(&self, set: &str, index: usize) -> &str {
        &self.labels[set][index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CUSTOM: &str = r#"
name = "small"
description = "A small naming pack"
[names.person]
formats = [{ format = "{given} {family}", weight = 1 }]
[names.person.components]
given = ["Ada", "Grace"]
family = ["River", "Meadow"]
[labels]
ranks = ["member", "regular"]
"#;

    const REQUIREMENTS: ThemeRequirements = ThemeRequirements {
        name_kinds: &["person"],
        label_sets: &[("ranks", 2)],
    };

    #[test]
    fn plain_covers_saas_names_and_labels_with_default_population_capacity() {
        let requirements = ThemeRequirements {
            name_kinds: &["person", "organization", "plan", "feature", "campaign"],
            label_sets: &[
                ("industries", 6),
                ("roles", 3),
                ("regions", 4),
                ("plan_tiers", 3),
            ],
        };
        let plain = Theme::load("plain").unwrap();
        plain.validate(&requirements).unwrap();
        for (kind, population) in [
            ("person", 100_000),
            ("organization", 5_000),
            ("plan", 3),
            ("feature", 12),
            ("campaign", 24),
        ] {
            assert!(
                plain.capacity(kind) >= population,
                "insufficient {kind} names"
            );
            let names = (0..population)
                .map(|index| plain.name(42, kind, index))
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(names.len(), population, "repeated {kind} names");
        }
        assert!(
            !Theme::load("fantasy_rpg")
                .unwrap()
                .is_compatible(&requirements)
        );
    }

    #[test]
    fn path_theme_supplies_only_its_declared_names_and_labels() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("custom.toml");
        std::fs::write(&path, CUSTOM).unwrap();
        let theme = Theme::load(path.to_str().unwrap()).unwrap();
        theme.validate(&REQUIREMENTS).unwrap();
        assert_eq!(theme.name, "small");
        assert_eq!(theme.capacity("person"), 4);
        assert_eq!(theme.label("ranks", 1), "regular");
        for index in 0..4 {
            assert!(
                ["Ada River", "Ada Meadow", "Grace River", "Grace Meadow"]
                    .contains(&theme.name(42, "person", index).as_str())
            );
        }
    }

    #[test]
    fn missing_entries_and_wrong_lengths_name_file_and_fields() {
        let theme = Theme::from_toml("small.toml", CUSTOM).unwrap();
        let requirements = ThemeRequirements {
            name_kinds: &["organization", "location"],
            label_sets: &[("ranks", 4), ("plans", 3)],
        };
        let error = format!("{:#}", theme.validate(&requirements).unwrap_err());
        for expected in [
            "small.toml",
            "names.organization",
            "names.location",
            "labels.ranks",
            "expected 4",
            "labels.plans",
            "compatible themes",
        ] {
            assert!(error.contains(expected), "{error}");
        }
        assert!(!theme.is_compatible(&requirements));
    }

    #[test]
    fn invalid_generator_reports_theme_source_kind_and_component() {
        let invalid = CUSTOM.replace("{given} {family}", "{missing}");
        let error = format!(
            "{:#}",
            Theme::from_toml("invalid.toml", &invalid).unwrap_err()
        );
        for expected in ["invalid.toml", "small", "names.person", "missing"] {
            assert!(error.contains(expected), "{error}");
        }
    }

    #[test]
    fn malformed_fields_and_empty_labels_are_rejected_with_the_source() {
        for (contents, field) in [
            (CUSTOM.replace("name = \"small\"", "name = 123"), "name"),
            (
                CUSTOM.replace(
                    "description = \"A small naming pack\"",
                    "description = \"\"",
                ),
                "description",
            ),
            (
                CUSTOM.replace("[\"member\", \"regular\"]", "[]"),
                "labels.ranks",
            ),
            (CUSTOM.replace("\"regular\"", "\" \""), "labels.ranks[1]"),
            (format!("unrecognized = true\n{CUSTOM}"), "unrecognized"),
        ] {
            let error = format!("{:#}", Theme::from_toml("bad.toml", &contents).unwrap_err());
            assert!(error.contains("bad.toml"), "{error}");
            assert!(error.contains(field), "{error}");
        }
    }
}
