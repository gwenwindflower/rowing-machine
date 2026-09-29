//! Whole-token names assigned through seeded, weighted permutations.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use anyhow::{Result, bail, ensure};
use serde::Deserialize;

use crate::engine::stream::Stream;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratorConfig {
    pub formats: Vec<FormatConfig>,
    pub components: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormatConfig {
    pub format: String,
    #[serde(default = "default_weight")]
    pub weight: u64,
}

const fn default_weight() -> u64 {
    1
}

#[derive(Debug, Clone)]
pub struct NameGenerator {
    names: Vec<(String, f64)>,
    permutations: Arc<Mutex<Permutations>>,
}

type Permutations = BTreeMap<(u64, String), Vec<usize>>;

impl NameGenerator {
    /// Expands formats and combines the weights of overlapping full names.
    ///
    /// # Errors
    /// Rejects malformed formats, missing or empty pools, and zero weights.
    #[allow(clippy::cast_precision_loss)]
    pub fn compile(config: GeneratorConfig) -> Result<Self> {
        ensure!(
            !config.formats.is_empty(),
            "formats must contain at least one format"
        );
        let mut names = BTreeMap::<String, f64>::new();
        for (index, format) in config.formats.into_iter().enumerate() {
            ensure!(
                format.weight > 0,
                "formats[{index}].weight must be greater than zero"
            );
            let expanded = expand(&format.format, &config.components, index)?;
            let weight = format.weight as f64 / expanded.len() as f64;
            for name in expanded {
                *names.entry(name).or_default() += weight;
            }
        }
        Ok(Self {
            names: names.into_iter().collect(),
            permutations: Arc::default(),
        })
    }

    #[must_use]
    pub fn capacity(&self) -> usize {
        self.names.len()
    }

    /// Assigns names by a weighted permutation, repeating that order after exhaustion.
    #[must_use]
    pub fn generate(&self, seed: u64, kind: &str, index: usize) -> String {
        let mut permutations = self
            .permutations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let permutation = permutations
            .entry((seed, kind.into()))
            .or_insert_with(|| self.permutation(seed, kind));
        self.names[permutation[index % self.capacity()]].0.clone()
    }

    fn permutation(&self, seed: u64, kind: &str) -> Vec<usize> {
        let mut stream = Stream::derive(seed, &format!("theme.names.{kind}"), &[]);
        let mut ranked: Vec<_> = self
            .names
            .iter()
            .enumerate()
            .map(|(index, (_, weight))| {
                let priority = -(1.0 - stream.uniform()).ln() / weight;
                (priority, index)
            })
            .collect();
        ranked.sort_by(|left, right| left.0.total_cmp(&right.0).then(left.1.cmp(&right.1)));
        ranked.into_iter().map(|(_, index)| index).collect()
    }
}

fn expand(
    format: &str,
    components: &BTreeMap<String, Vec<String>>,
    index: usize,
) -> Result<BTreeSet<String>> {
    let mut names = BTreeSet::from([String::new()]);
    let mut rest = format;
    while !rest.is_empty() {
        let next = rest.find(['{', '}']).unwrap_or(rest.len());
        if next > 0 {
            names = append(&names, &[rest[..next].to_owned()]);
            rest = &rest[next..];
            continue;
        }
        ensure!(
            rest.starts_with('{'),
            "formats[{index}] has an unmatched closing brace"
        );
        let Some(end) = rest.find('}') else {
            bail!("formats[{index}] has an unclosed component reference");
        };
        let component = &rest[1..end];
        ensure!(
            !component.is_empty() && !component.contains('{'),
            "formats[{index}] has an invalid component reference"
        );
        let Some(pool) = components.get(component) else {
            bail!("formats[{index}] references unknown component '{component}'");
        };
        ensure!(
            !pool.is_empty() && pool.iter().all(|token| !token.trim().is_empty()),
            "formats[{index}] requires a nonempty components.{component} pool with nonempty tokens"
        );
        names = append(&names, pool);
        rest = &rest[end + 1..];
    }
    ensure!(
        names.iter().all(|name| !name.trim().is_empty()),
        "formats[{index}] produces an empty name"
    );
    Ok(names)
}

fn append(prefixes: &BTreeSet<String>, suffixes: &[String]) -> BTreeSet<String> {
    prefixes
        .iter()
        .flat_map(|prefix| {
            suffixes
                .iter()
                .map(move |suffix| format!("{prefix}{suffix}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{FormatConfig, GeneratorConfig, NameGenerator};

    fn config(formats: &[(&str, u64)]) -> GeneratorConfig {
        GeneratorConfig {
            formats: formats
                .iter()
                .map(|&(format, weight)| FormatConfig {
                    format: format.into(),
                    weight,
                })
                .collect(),
            components: BTreeMap::from([
                (
                    "first".into(),
                    vec!["River".into(), "Moon".into(), "River".into()],
                ),
                ("last".into(), vec!["Stone".into(), "Vale".into()]),
            ]),
        }
    }

    #[test]
    fn whole_token_names_exhaust_unique_combinations_before_repeating() {
        let generator =
            NameGenerator::compile(config(&[("{first} {last}", 3), ("River {last}", 1)])).unwrap();
        assert_eq!(generator.capacity(), 4);
        let names: Vec<_> = (0..4)
            .map(|index| generator.generate(42, "person", index))
            .collect();
        assert_eq!(names.iter().collect::<BTreeSet<_>>().len(), 4);
        assert_eq!(
            names.iter().map(String::as_str).collect::<BTreeSet<_>>(),
            BTreeSet::from(["River Stone", "River Vale", "Moon Stone", "Moon Vale"])
        );
        for index in 0..12 {
            assert_eq!(generator.generate(42, "person", index), names[index % 4]);
        }
    }

    #[test]
    fn assignments_depend_only_on_seed_kind_and_index() {
        let generator = NameGenerator::compile(config(&[("{first} {last}", 1)])).unwrap();
        let other = NameGenerator::compile(config(&[("{first} {last}", 1)])).unwrap();
        let expected: Vec<_> = (0..4)
            .map(|index| generator.generate(42, "person", index))
            .collect();
        for index in [3, 1, 0, 2] {
            let _ = other.generate(9, "organization", index);
            assert_eq!(other.generate(42, "person", index), expected[index]);
        }
        assert!((0..32).any(|seed| generator.generate(seed, "person", 0) != expected[0]));
        assert!((0..32).any(|seed| generator.generate(seed, "person", 0)
            != generator.generate(seed, "organization", 0)));
    }

    #[test]
    fn format_weights_bias_first_assignment_without_preventing_exhaustion() {
        let generator =
            NameGenerator::compile(config(&[("Preferred", 99), ("{first} {last}", 1)])).unwrap();
        let preferred_first = (0..200)
            .filter(|&seed| generator.generate(seed, "product", 0) == "Preferred")
            .count();
        assert!(
            preferred_first > 180,
            "preferred first in {preferred_first} seeds"
        );
        assert_eq!(
            (0..5)
                .map(|index| generator.generate(42, "product", index))
                .collect::<BTreeSet<_>>()
                .len(),
            5
        );
    }

    #[test]
    fn invalid_formats_and_required_pools_are_rejected() {
        for format in [
            "",
            "   ",
            "{",
            "}",
            "{}",
            "{{first}}",
            "{unknown}",
            "{first",
        ] {
            let error = NameGenerator::compile(config(&[(format, 1)]))
                .unwrap_err()
                .to_string();
            assert!(error.contains("formats[0]"), "{error}");
        }
        let error = NameGenerator::compile(config(&[("{first}", 0)]))
            .unwrap_err()
            .to_string();
        assert!(error.contains("weight"));
        let mut empty_pool = config(&[("{first}", 1)]);
        empty_pool.components.insert("first".into(), vec![]);
        let error = NameGenerator::compile(empty_pool).unwrap_err().to_string();
        assert!(error.contains("first"));
        assert!(NameGenerator::compile(config(&[])).is_err());
    }
}
