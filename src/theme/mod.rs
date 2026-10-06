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
    /// Namespace for this scenario's parameters under `[params.<scenario>]`.
    pub scenario: &'static str,
    pub name_kinds: &'static [&'static str],
    pub label_sets: &'static [(&'static str, usize)],
    pub catalogs: &'static [CatalogSpec],
    pub params: &'static [ParamSpec],
}

/// A tunable number a scenario exposes to themes and `--param`.
#[derive(Debug, Clone, Copy)]
pub struct ParamSpec {
    pub name: &'static str,
    pub default: f64,
    pub min: f64,
    pub max: f64,
    pub about: &'static str,
}

/// An ordered list of typed records a scenario reads from a theme.
#[derive(Debug, Clone, Copy)]
pub struct CatalogSpec {
    pub name: &'static str,
    pub min_len: usize,
    pub fields: &'static [(&'static str, FieldKind)],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Text,
    Number,
    Boolean,
}

/// One catalog entry, read through the fields its scenario declared.
#[derive(Debug, Clone)]
pub struct Record(toml::Table);

impl Record {
    /// # Panics
    /// Panics if the scenario did not declare the field as text.
    #[must_use]
    pub fn text(&self, field: &str) -> &str {
        self.0[field].as_str().expect("validated text field")
    }

    /// # Panics
    /// Panics if the scenario did not declare the field as a number.
    #[must_use]
    pub fn number(&self, field: &str) -> f64 {
        number(&self.0[field]).expect("validated number field")
    }

    /// # Panics
    /// Panics if the scenario did not declare the field as a boolean.
    #[must_use]
    pub fn boolean(&self, field: &str) -> bool {
        self.0[field].as_bool().expect("validated boolean field")
    }
}

#[allow(clippy::cast_precision_loss)]
fn number(value: &toml::Value) -> Option<f64> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|integer| integer as f64))
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
    #[serde(default)]
    catalogs: BTreeMap<String, Vec<toml::Table>>,
    #[serde(default)]
    params: BTreeMap<String, BTreeMap<String, toml::Value>>,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub description: String,
    source: String,
    names: BTreeMap<String, NameGenerator>,
    labels: BTreeMap<String, Vec<String>>,
    catalogs: BTreeMap<String, Vec<Record>>,
    params: BTreeMap<String, BTreeMap<String, toml::Value>>,
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
            format!("--theme {selector:?}: cannot read theme; use a bundled theme from `rowing-machine themes` or a readable TOML path")
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
            catalogs: config
                .catalogs
                .into_iter()
                .map(|(name, records)| (name, records.into_iter().map(Record).collect()))
                .collect(),
            params: config.params,
        })
    }

    /// Replaces this theme's parameter values for one scenario, as `--param` does.
    #[must_use]
    pub fn with_overrides(mut self, scenario: &str, overrides: &[(String, f64)]) -> Self {
        let values = self.params.entry(scenario.to_owned()).or_default();
        for (name, value) in overrides {
            values.insert(name.clone(), toml::Value::Float(*value));
        }
        self
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
        for spec in requirements.catalogs {
            let Some(records) = self.catalogs.get(spec.name) else {
                issues.push(format!("missing catalogs.{}", spec.name));
                continue;
            };
            if records.len() < spec.min_len {
                issues.push(format!(
                    "catalogs.{} has {} records; expected at least {}",
                    spec.name,
                    records.len(),
                    spec.min_len
                ));
            }
            for (index, Record(record)) in records.iter().enumerate() {
                let path = format!("catalogs.{}[{index}]", spec.name);
                for (field, kind) in spec.fields {
                    let valid = match (record.get(*field), kind) {
                        (Some(toml::Value::String(text)), FieldKind::Text) => {
                            !text.trim().is_empty()
                        }
                        (Some(value), FieldKind::Number) => number(value).is_some(),
                        (Some(value), FieldKind::Boolean) => value.is_bool(),
                        _ => false,
                    };
                    if !valid {
                        let kind = match kind {
                            FieldKind::Text => "non-empty string",
                            FieldKind::Number => "number",
                            FieldKind::Boolean => "boolean",
                        };
                        issues.push(format!("{path}.{field} must be a {kind}"));
                    }
                }
                for field in record.keys() {
                    if !spec.fields.iter().any(|(name, _)| name == field) {
                        issues.push(format!("{path}.{field} is not a known field"));
                    }
                }
            }
        }
        let known = requirements
            .params
            .iter()
            .map(|spec| spec.name)
            .collect::<Vec<_>>()
            .join(", ");
        for (name, value) in self.params.get(requirements.scenario).into_iter().flatten() {
            let path = format!("params.{}.{name}", requirements.scenario);
            match requirements.params.iter().find(|spec| spec.name == name) {
                None => issues.push(format!("{path} is not a parameter; use one of {known}")),
                Some(spec) => match number(value) {
                    Some(number) if (spec.min..=spec.max).contains(&number) => {}
                    _ => issues.push(format!(
                        "{path} must be a number from {} to {}",
                        spec.min, spec.max
                    )),
                },
            }
        }
        issues
    }

    /// Returns a scenario parameter: the theme's value or the scenario default.
    ///
    /// # Panics
    /// Panics if the scenario did not declare the parameter.
    #[must_use]
    pub fn param(&self, requirements: &ThemeRequirements, name: &str) -> f64 {
        let spec = requirements
            .params
            .iter()
            .find(|spec| spec.name == name)
            .expect("declared parameter");
        self.params
            .get(requirements.scenario)
            .and_then(|values| values.get(name))
            .and_then(number)
            .unwrap_or(spec.default)
    }

    /// Returns a catalog's records after compatibility validation.
    ///
    /// # Panics
    /// Panics if the scenario did not validate the catalog.
    #[must_use]
    pub fn catalog(&self, name: &str) -> &[Record] {
        &self.catalogs[name]
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
        scenario: "shop",
        name_kinds: &["person"],
        label_sets: &[("ranks", 2)],
        catalogs: &[],
        params: &[],
    };

    const PARAMETERS: &[ParamSpec] = &[
        ParamSpec {
            name: "price_scale",
            default: 1.0,
            min: 0.01,
            max: 1000.0,
            about: "Multiplies catalog prices",
        },
        ParamSpec {
            name: "density",
            default: 0.5,
            min: 0.0,
            max: 1.0,
            about: "Share of possible links",
        },
    ];

    const LOCATIONS: &[CatalogSpec] = &[CatalogSpec {
        name: "locations",
        min_len: 2,
        fields: &[
            ("name", FieldKind::Text),
            ("latitude", FieldKind::Number),
            ("base", FieldKind::Boolean),
        ],
    }];

    const WITH_PARAMETERS: ThemeRequirements = ThemeRequirements {
        params: PARAMETERS,
        catalogs: LOCATIONS,
        ..REQUIREMENTS
    };

    const CATALOG: &str = r#"
[[catalogs.locations]]
name = "Harbor"
latitude = 51.5
base = true
[[catalogs.locations]]
name = "Summit"
latitude = 46
base = false
"#;

    fn with_catalog(extra: &str) -> Theme {
        Theme::from_toml("custom.toml", &format!("{CUSTOM}{CATALOG}{extra}")).unwrap()
    }

    #[test]
    fn theme_parameters_override_defaults_for_their_scenario_only() {
        let theme = with_catalog("[params.shop]\nprice_scale = 2\n[params.other]\nanything = 9\n");
        theme.validate(&WITH_PARAMETERS).unwrap();
        assert!((theme.param(&WITH_PARAMETERS, "price_scale") - 2.0).abs() < f64::EPSILON);
        assert!((theme.param(&WITH_PARAMETERS, "density") - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn unknown_and_out_of_range_parameters_name_the_parameter_and_valid_choices() {
        for (params, expected) in [
            (
                "dnesity = 0.2",
                ["params.shop.dnesity", "price_scale, density"],
            ),
            ("price_scale = 0", ["params.shop.price_scale", "0.01"]),
        ] {
            let theme = with_catalog(&format!("[params.shop]\n{params}\n"));
            let error = format!("{:#}", theme.validate(&WITH_PARAMETERS).unwrap_err());
            for text in expected {
                assert!(error.contains(text), "{error}");
            }
        }
    }

    #[test]
    fn overrides_replace_theme_parameters_and_reject_unknown_names() {
        let theme = with_catalog("[params.shop]\nprice_scale = 2\n")
            .with_overrides("shop", &[("price_scale".to_owned(), 3.0)]);
        theme.validate(&WITH_PARAMETERS).unwrap();
        assert!((theme.param(&WITH_PARAMETERS, "price_scale") - 3.0).abs() < f64::EPSILON);
        let unknown = with_catalog("").with_overrides("shop", &[("speed".to_owned(), 1.0)]);
        let error = format!("{:#}", unknown.validate(&WITH_PARAMETERS).unwrap_err());
        assert!(error.contains("params.shop.speed"), "{error}");
    }

    #[test]
    fn catalog_records_expose_declared_fields() {
        let theme = with_catalog("");
        theme.validate(&WITH_PARAMETERS).unwrap();
        let locations = theme.catalog("locations");
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].text("name"), "Harbor");
        assert!((locations[1].number("latitude") - 46.0).abs() < f64::EPSILON);
        assert!(locations[0].boolean("base"));
    }

    #[test]
    fn catalog_records_with_missing_or_mistyped_fields_are_incompatible() {
        for (contents, expected) in [
            (
                CATALOG.replace("latitude = 46\n", ""),
                "catalogs.locations[1].latitude",
            ),
            (
                CATALOG.replace("base = true", "base = \"yes\""),
                "catalogs.locations[0].base",
            ),
            (
                CATALOG.replace("name = \"Summit\"", "name = \"Summit\"\nrunway = 3"),
                "catalogs.locations[1].runway",
            ),
            (
                CATALOG
                    .split("[[catalogs.locations]]")
                    .take(2)
                    .collect::<Vec<_>>()
                    .join("[[catalogs.locations]]"),
                "at least 2",
            ),
            (String::new(), "missing catalogs.locations"),
        ] {
            let theme = Theme::from_toml("custom.toml", &format!("{CUSTOM}{contents}")).unwrap();
            let error = format!("{:#}", theme.validate(&WITH_PARAMETERS).unwrap_err());
            assert!(error.contains(expected), "{error}");
        }
    }

    #[test]
    fn plain_covers_saas_names_and_labels_with_default_population_capacity() {
        let requirements = ThemeRequirements {
            scenario: "saas",
            catalogs: &[],
            params: &[],
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
            ..REQUIREMENTS
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
