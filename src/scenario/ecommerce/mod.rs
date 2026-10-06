mod catalog;
mod persona;

use super::{Scenario, UnitRows, WorkUnit};
use crate::engine::{
    calendar::{DayState, penetration},
    stream::Stream,
};
use crate::output::{Column, ColumnType, EntitySchema, Value};
use crate::theme::{ParamSpec, Theme, ThemeRequirements};
use anyhow::{Context, Result, ensure};
use catalog::{PRODUCTS, STORES, SUPPLIES};
use persona::Persona;
use std::collections::BTreeMap;

const STORE_STREAM: &str = "stores";
const CUSTOMER_STREAM: &str = "customers";
const PERSONA_STREAM: &str = "persona-block";
const ORDER_STREAM: &str = "market-day-customer";
const SPARROW_TEXT_STREAM: &str = "sparrow-text";

const PARAMS: &[ParamSpec] = &[ParamSpec {
    name: "price_scale",
    default: 1.0,
    min: 0.01,
    max: 1000.0,
    about: "multiplies product prices and supply costs",
}];

#[derive(Debug, PartialEq)]
struct Customer {
    id: [u8; 16],
    persona: Persona,
    favorite: u32,
    fan: usize,
    activation: usize,
}

pub struct Ecommerce {
    days: Vec<DayState>,
    theme: Theme,
    customers: Vec<Vec<Customer>>,
    stores: Vec<[u8; 16]>,
    counts: BTreeMap<[u8; 16], u64>,
    ranks: BTreeMap<[u8; 16], usize>,
    name_indices: BTreeMap<[u8; 16], usize>,
    price_scale: f64,
}

impl Ecommerce {
    /// Declares the names and ordered labels required by ecommerce.
    #[must_use]
    pub fn theme_requirements() -> ThemeRequirements {
        ThemeRequirements {
            scenario: "ecommerce",
            catalogs: &[],
            params: PARAMS,
            name_kinds: &["person"],
            label_sets: &[
                ("stores", 6),
                ("products", 15),
                ("product_descriptions", 15),
                ("product_types", 3),
                ("power_levels", 5),
                ("supplies", 41),
                ("ranks", 4),
                ("rank_voices", 4),
                ("positive_adjectives", 7),
                ("negative_adjectives", 7),
                ("neutral_adjectives", 8),
                ("sparrow_templates", 3),
                ("acquired_templates", 3),
                ("item_separator", 1),
            ],
        }
    }

    /// Returns customer identity and persona names for distribution analysis.
    #[must_use]
    pub fn customer_personas(&self) -> BTreeMap<String, String> {
        self.customers
            .iter()
            .flatten()
            .map(|customer| (uuid_text(customer.id), format!("{:?}", customer.persona)))
            .collect()
    }

    /// Constructs indexed customer pools and fixed calendar state.
    ///
    /// # Errors
    /// Returns an error for empty calendars or overflowing customer pools.
    pub fn new(seed: u64, scale: usize, days: Vec<DayState>) -> Result<Self> {
        Self::with_theme(seed, scale, days, Theme::load("plain")?)
    }

    /// Constructs indexed customer pools using a compatible naming theme.
    ///
    /// # Errors
    /// Returns an error for incompatible themes, empty calendars, or overflowing pools.
    pub fn with_theme(seed: u64, scale: usize, days: Vec<DayState>, theme: Theme) -> Result<Self> {
        theme.validate(&Self::theme_requirements())?;
        ensure!(
            !days.is_empty(),
            "ecommerce requires at least one simulation day"
        );
        let mut customers = Vec::new();
        let mut stores = Vec::new();
        for (market, store) in STORES.iter().enumerate() {
            stores.push(Stream::derive(seed, STORE_STREAM, &[market as u64]).uuid());
            let count = store.tam.checked_mul(scale).ok_or_else(|| {
                anyhow::anyhow!(
                    "--scale {scale} exceeds the customer pool limit; use a smaller value"
                )
            })?;
            let mut pool = Vec::new();
            pool.try_reserve_exact(count).with_context(|| {
                format!(
                    "--scale {scale} cannot fit the customer pool in memory; use a smaller value"
                )
            })?;
            let mut personas = [Persona::Courier; 20];
            for index in 0..count {
                if index % 20 == 0 {
                    personas = Persona::block(&mut Stream::derive(
                        seed,
                        PERSONA_STREAM,
                        &[market as u64, (index / 20) as u64],
                    ));
                }
                let mut rng = Stream::derive(seed, CUSTOMER_STREAM, &[market as u64, index as u64]);
                let id = rng.uuid();
                let persona = personas[index % 20];
                let favorite = u32::try_from(rng.index(100) + 1)?;
                let fan = rng.index(5) + 1;
                let threshold = rng.uniform();
                let activation = (1..=365)
                    .find(|day| penetration(*day) >= threshold)
                    .unwrap_or(365)
                    + store.opens;
                pool.push(Customer {
                    id,
                    persona,
                    favorite,
                    fan,
                    activation,
                });
            }
            customers.push(pool);
        }
        let price_scale = theme.param(&Self::theme_requirements(), "price_scale");
        Ok(Self {
            days,
            theme,
            price_scale,
            customers,
            stores,
            counts: BTreeMap::new(),
            ranks: BTreeMap::new(),
            name_indices: BTreeMap::new(),
        })
    }

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    fn scaled(&self, cents: i64) -> i64 {
        (cents as f64 * self.price_scale).round() as i64
    }

    fn static_rows(&self) -> Result<UnitRows> {
        let mut rows = Vec::new();
        for (index, store) in STORES.iter().enumerate() {
            let opened = self.days[0]
                .date
                .checked_add(jiff::Span::new().days(i64::try_from(store.opens)?))
                .with_context(|| format!("--start-date {} puts {} outside the supported calendar; use an earlier date", self.days[0].date, self.theme.label("stores", index)))?;
            let timestamp = opened
                .at(0, 0, 0, 0)
                .to_zoned(jiff::tz::TimeZone::UTC)?
                .timestamp()
                .as_microsecond();
            rows.push((
                "stores",
                vec![
                    Value::Uuid(self.stores[index]),
                    text(self.theme.label("stores", index)),
                    Value::Timestamp(timestamp),
                    Value::Float(store.tax_rate),
                ],
            ));
        }
        for (index, product) in PRODUCTS.iter().enumerate() {
            rows.push((
                "products",
                vec![
                    text(product.sku),
                    text(self.theme.label("products", index)),
                    text(self.theme.label("product_types", product.kind)),
                    Value::Cents(self.scaled(product.price)),
                    text(self.theme.label("product_descriptions", index)),
                    text(self.theme.label("power_levels", product.power_level)),
                ],
            ));
        }
        for (index, supply) in SUPPLIES.iter().enumerate() {
            for sku in supply.skus {
                rows.push((
                    "supplies",
                    vec![
                        text(supply.id),
                        text(self.theme.label("supplies", index)),
                        Value::Cents(self.scaled(supply.cost)),
                        Value::Boolean(supply.volatile),
                        text(self.theme.label("stores", supply.origin_region)),
                        text(sku),
                    ],
                ));
            }
        }
        Ok(rows)
    }

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    fn market_day(
        &self,
        seed: u64,
        day_index: usize,
        market: usize,
        sparrows: bool,
    ) -> Result<UnitRows> {
        let day = self
            .days
            .get(day_index)
            .ok_or_else(|| anyhow::anyhow!("invalid ecommerce day {day_index}"))?;
        let store = STORES
            .get(market)
            .ok_or_else(|| anyhow::anyhow!("invalid ecommerce market {market}"))?;
        let midnight = day
            .date
            .at(0, 0, 0, 0)
            .to_zoned(jiff::tz::TimeZone::UTC)?
            .timestamp()
            .as_microsecond();
        let mut rows = Vec::new();
        for (index, customer) in self.customers[market].iter().enumerate() {
            if day.index < customer.activation {
                continue;
            }
            let mut rng = Stream::derive(
                seed,
                ORDER_STREAM,
                &[market as u64, day.index as u64, index as u64],
            );
            let probability = (store.popularity
                * day.effect
                * customer
                    .persona
                    .probability(day.is_weekend, day.season, customer.favorite))
            .sqrt();
            if rng.uniform() >= probability {
                continue;
            }
            let minute = customer.persona.minute(&mut rng, customer.favorite);
            if minute < day.opens_at || minute >= day.closes_at {
                continue;
            }
            let items = customer.persona.items(&mut rng, customer.favorite);
            if items.is_empty() {
                continue;
            }
            let subtotal: i64 = items
                .iter()
                .map(|index| self.scaled(PRODUCTS[*index].price))
                .sum();
            let tax = (subtotal as f64 * store.tax_rate).round() as i64;
            let ordered_at = midnight + i64::from(minute) * 60_000_000;
            let order_id = rng.uuid();
            if !sparrows {
                rows.push((
                    "orders",
                    vec![
                        Value::Uuid(order_id),
                        Value::Uuid(customer.id),
                        Value::Timestamp(ordered_at),
                        Value::Uuid(self.stores[market]),
                        Value::Cents(subtotal),
                        Value::Cents(tax),
                        Value::Cents(subtotal + tax),
                    ],
                ));
            }
            for item in &items {
                let id = rng.uuid();
                if !sparrows {
                    rows.push((
                        "items",
                        vec![
                            Value::Uuid(id),
                            Value::Uuid(order_id),
                            text(PRODUCTS[*item].sku),
                        ],
                    ));
                }
            }
            if rng.uniform() < customer.persona.sparrow_probability() && sparrows {
                let delay = i64::try_from(rng.index(20))?;
                let rank = *self.ranks.get(&customer.id).ok_or_else(|| {
                    anyhow::anyhow!(
                        "customer guild ranks must be finalized before sparrow generation"
                    )
                })?;
                let mut text_rng = Stream::derive(
                    seed,
                    SPARROW_TEXT_STREAM,
                    &[market as u64, day.index as u64, index as u64],
                );
                let content =
                    sparrow_content(&self.theme, &mut text_rng, customer.fan, rank, &items);
                rows.push((
                    "sparrows",
                    vec![
                        Value::Uuid(rng.uuid()),
                        Value::Uuid(customer.id),
                        Value::Timestamp(ordered_at + delay * 60_000_000),
                        text(&content),
                    ],
                ));
            }
        }
        Ok(rows)
    }
}

impl Scenario for Ecommerce {
    fn name(&self) -> &'static str {
        "ecommerce"
    }
    fn entities(&self) -> Vec<EntitySchema> {
        use ColumnType::{Boolean, Cents, Float, Text, Timestamp, Uuid};
        vec![
            schema(
                "stores",
                &[
                    ("id", Uuid),
                    ("name", Text),
                    ("opened_at", Timestamp),
                    ("tax_rate", Float),
                ],
                &["id"],
            ),
            schema(
                "customers",
                &[("id", Uuid), ("name", Text), ("guild_rank", Text)],
                &["id"],
            ),
            schema(
                "orders",
                &[
                    ("id", Uuid),
                    ("customer", Uuid),
                    ("ordered_at", Timestamp),
                    ("store_id", Uuid),
                    ("subtotal", Cents),
                    ("tax_paid", Cents),
                    ("order_total", Cents),
                ],
                &["id"],
            ),
            schema(
                "items",
                &[("id", Uuid), ("order_id", Uuid), ("sku", Text)],
                &["id"],
            ),
            schema(
                "products",
                &[
                    ("sku", Text),
                    ("name", Text),
                    ("type", Text),
                    ("price", Cents),
                    ("description", Text),
                    ("power_level", Text),
                ],
                &["sku"],
            ),
            schema(
                "supplies",
                &[
                    ("id", Text),
                    ("name", Text),
                    ("cost", Cents),
                    ("volatile", Boolean),
                    ("origin_region", Text),
                    ("sku", Text),
                ],
                &["id", "sku"],
            ),
            schema(
                "sparrows",
                &[
                    ("id", Uuid),
                    ("user_id", Uuid),
                    ("sent_at", Timestamp),
                    ("content", Text),
                ],
                &["id"],
            ),
        ]
    }
    fn units(&self) -> Vec<WorkUnit> {
        let mut units = vec![WorkUnit {
            stage: 0,
            indices: [0, 0],
        }];
        for stage in [1, 2] {
            for day in 0..self.days.len() {
                for market in 0..STORES.len() {
                    units.push(WorkUnit {
                        stage,
                        indices: [day as u64, market as u64],
                    });
                }
            }
        }
        units.extend((0..STORES.len()).map(|market| WorkUnit {
            stage: 3,
            indices: [0, market as u64],
        }));
        units
    }
    fn generate(&self, seed: u64, unit: WorkUnit) -> Result<UnitRows> {
        let day = usize::try_from(unit.indices[0])?;
        let market = usize::try_from(unit.indices[1])?;
        match unit.stage {
            0 => self.static_rows(),
            1 | 2 => self.market_day(seed, day, market, unit.stage == 2),
            3 => {
                let pool = self
                    .customers
                    .get(market)
                    .ok_or_else(|| anyhow::anyhow!("invalid ecommerce market {market}"))?;
                Ok(pool
                    .iter()
                    .filter_map(|customer| {
                        self.ranks.get(&customer.id).map(|rank| {
                            (
                                "customers",
                                vec![
                                    Value::Uuid(customer.id),
                                    text(&self.theme.name(
                                        seed,
                                        "person",
                                        self.name_indices[&customer.id],
                                    )),
                                    text(self.theme.label("ranks", *rank)),
                                ],
                            )
                        })
                    })
                    .collect())
            }
            _ => anyhow::bail!("invalid ecommerce stage {}", unit.stage),
        }
    }
    fn observe(&mut self, rows: &UnitRows) -> Result<()> {
        for (entity, row) in rows {
            if *entity == "orders" {
                if let Some(Value::Uuid(id)) = row.get(1) {
                    *self.counts.entry(*id).or_default() += 1;
                } else {
                    anyhow::bail!("order row has no customer UUID");
                }
            }
        }
        Ok(())
    }
    fn complete_stage(&mut self, stage: u32) -> Result<()> {
        if stage == 1 {
            self.name_indices = self
                .customers
                .iter()
                .flatten()
                .filter(|customer| self.counts.contains_key(&customer.id))
                .enumerate()
                .map(|(index, customer)| (customer.id, index))
                .collect();
            let mut counts: Vec<_> = self.counts.iter().collect();
            counts.sort_by_key(|(id, count)| (**count, **id));
            let length = counts.len();
            self.ranks.clear();
            for (index, (id, _)) in counts.into_iter().enumerate() {
                self.ranks.insert(*id, index * 4 / length);
            }
        }
        Ok(())
    }
}

fn text(value: &str) -> Value {
    Value::Text(value.to_owned())
}
fn uuid_text(id: [u8; 16]) -> String {
    use std::fmt::Write;
    let mut output = String::with_capacity(36);
    for (index, byte) in id.iter().enumerate() {
        if [4, 6, 8, 10].contains(&index) {
            output.push('-');
        }
        write!(output, "{byte:02x}").expect("writing to a string cannot fail");
    }
    output
}
fn schema(
    name: &'static str,
    fields: &[(&'static str, ColumnType)],
    key: &[&'static str],
) -> EntitySchema {
    EntitySchema {
        name,
        columns: fields
            .iter()
            .map(|(name, column_type)| Column {
                name,
                column_type: *column_type,
                nullable: false,
            })
            .collect(),
        primary_key: key.to_vec(),
    }
}

#[allow(clippy::comparison_chain)]
fn sparrow_content(
    theme: &Theme,
    rng: &mut Stream,
    fan: usize,
    rank: usize,
    items: &[usize],
) -> String {
    let (pool, length, template) = if fan > 3 {
        ("positive_adjectives", 7, 0)
    } else if fan < 3 {
        ("negative_adjectives", 7, 1)
    } else {
        ("neutral_adjectives", 8, 2)
    };
    let adjective = theme.label(pool, rng.index(length));
    let names: Vec<_> = items
        .iter()
        .map(|index| theme.label("products", *index))
        .collect();
    let acquired = match names.as_slice() {
        [one] => theme.label("acquired_templates", 0).replace("{one}", one),
        [one, two] => theme
            .label("acquired_templates", 1)
            .replace("{one}", one)
            .replace("{two}", two),
        _ => theme
            .label("acquired_templates", 2)
            .replace(
                "{one}",
                &names[..names.len() - 1].join(theme.label("item_separator", 0)),
            )
            .replace("{two}", names[names.len() - 1]),
    };
    let voice = theme.label("rank_voices", rank);
    let content = theme
        .label("sparrow_templates", template)
        .replace("{adjective}", adjective)
        .replace("{acquired}", &acquired);
    format!("{voice}: {content}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::calendar::precompute;
    use jiff::civil::date;

    #[test]
    fn bundled_names_repeat_the_reviewed_seed_snapshot() {
        let snapshots: Vec<_> = ["plain", "fantasy_rpg"]
            .into_iter()
            .map(|selector| {
                let theme = Theme::load(selector).unwrap();
                (0..5)
                    .map(|index| theme.name(42, "person", index))
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(
            snapshots,
            vec![
                vec![
                    "Micah Ira Morris",
                    "Luca Sanchez",
                    "Avery Carter",
                    "Wren Flores",
                    "Blake Wilson"
                ],
                vec![
                    "Fenris Ironbrook",
                    "Kaida Rainbrook",
                    "Faye Firelight",
                    "Kaida Wintervale",
                    "Amaris Redbrook"
                ],
            ]
        );
    }

    #[test]
    fn themes_preserve_ids_dates_and_numbers_across_all_stages() {
        let days = precompute(date(2023, 1, 1), 400).unwrap();
        let mut plain = Ecommerce::with_theme(
            42,
            2,
            days.clone(),
            crate::theme::Theme::load("plain").unwrap(),
        )
        .unwrap();
        let mut fantasy = Ecommerce::with_theme(
            42,
            2,
            days,
            crate::theme::Theme::load("fantasy_rpg").unwrap(),
        )
        .unwrap();
        for stage in 0..=3 {
            for unit in plain.units().into_iter().filter(|unit| unit.stage == stage) {
                let left = plain.generate(42, unit).unwrap();
                let right = fantasy.generate(42, unit).unwrap();
                assert_eq!(left.len(), right.len());
                for ((entity, row), (other_entity, other_row)) in left.iter().zip(&right) {
                    assert_eq!(entity, other_entity);
                    for (index, (value, other)) in row.iter().zip(other_row).enumerate() {
                        if !matches!(value, Value::Text(_))
                            || matches!(
                                (*entity, index),
                                ("products" | "supplies", 0) | ("supplies", 5) | ("items", 2)
                            )
                        {
                            assert_eq!(value, other, "{entity} column {index}");
                        }
                    }
                }
                plain.observe(&left).unwrap();
                fantasy.observe(&right).unwrap();
            }
            plain.complete_stage(stage).unwrap();
            fantasy.complete_stage(stage).unwrap();
        }
    }

    #[test]
    fn sparse_ordering_customers_exhaust_names_before_repeating_across_markets() {
        let (_, labels) = include_str!("../../../themes/plain.toml")
            .split_once("[labels]")
            .unwrap();
        let contents = format!(
            r#"
name = "tiny"
description = "Four people"
[names.person]
formats = [{{ format = "{{given}} {{family}}", weight = 1 }}]
[names.person.components]
given = ["Ada", "Grace"]
family = ["River", "Hill"]
[labels]
{labels}
"#
        );
        let theme = Theme::from_toml("tiny.toml", &contents).unwrap();
        let mut scenario =
            Ecommerce::with_theme(42, 1, precompute(date(2023, 1, 1), 1).unwrap(), theme).unwrap();
        for (market, index) in [(0, 0), (0, 4), (1, 3), (2, 1)] {
            scenario
                .counts
                .insert(scenario.customers[market][index].id, 1);
        }
        scenario.complete_stage(1).unwrap();
        let mut names = std::collections::BTreeSet::new();
        for unit in scenario.units().into_iter().filter(|unit| unit.stage == 3) {
            for (_, row) in scenario.generate(42, unit).unwrap() {
                let Value::Text(name) = &row[1] else { panic!() };
                assert!(
                    names.insert(name.clone()),
                    "name repeated before exhaustion: {name}"
                );
            }
        }
        assert_eq!(names.len(), 4);
    }

    #[test]
    fn bundled_themes_name_the_default_population_without_repeating_across_markets() {
        for selector in ["plain", "fantasy_rpg", "sneakers"] {
            let theme = Theme::load(selector).unwrap();
            assert!(theme.capacity("person") >= 6200);
            let mut scenario =
                Ecommerce::with_theme(42, 100, precompute(date(2023, 1, 1), 1).unwrap(), theme)
                    .unwrap();
            for customer in scenario.customers.iter().flatten() {
                scenario.counts.insert(customer.id, 1);
            }
            scenario.complete_stage(1).unwrap();
            let mut names = std::collections::BTreeSet::new();
            for unit in scenario.units().into_iter().filter(|unit| unit.stage == 3) {
                for (_, row) in scenario.generate(42, unit).unwrap() {
                    let Value::Text(name) = &row[1] else { panic!() };
                    assert!(
                        names.insert(name.clone()),
                        "repeated {selector} person {name}"
                    );
                }
            }
            assert_eq!(names.len(), 6200);
        }
    }

    #[test]
    fn changing_person_formats_changes_only_customer_names() {
        let contents = include_str!("../../../themes/plain.toml")
            .replace("{given} {family}", "{family}, {given}");
        let theme = Theme::from_toml("reversed.toml", &contents).unwrap();
        let days = precompute(date(2023, 1, 1), 400).unwrap();
        let mut plain = Ecommerce::new(42, 1, days.clone()).unwrap();
        let mut reversed = Ecommerce::with_theme(42, 1, days, theme).unwrap();
        let mut renamed = 0;
        for stage in 0..=3 {
            for unit in plain.units().into_iter().filter(|unit| unit.stage == stage) {
                let rows = plain.generate(42, unit).unwrap();
                let other = reversed.generate(42, unit).unwrap();
                assert_eq!(rows.len(), other.len());
                for ((entity, row), (other_entity, other_row)) in rows.iter().zip(&other) {
                    assert_eq!(entity, other_entity);
                    if *entity == "customers" {
                        assert_ne!(row[1], other_row[1]);
                        assert_eq!(row[0], other_row[0]);
                        assert_eq!(row[2], other_row[2]);
                        renamed += 1;
                    } else {
                        assert_eq!(row, other_row);
                    }
                }
                plain.observe(&rows).unwrap();
                reversed.observe(&other).unwrap();
            }
            plain.complete_stage(stage).unwrap();
            reversed.complete_stage(stage).unwrap();
        }
        assert!(renamed > 0);
    }

    #[test]
    fn fantasy_catalog_preserves_the_arcanum_collective_labels() {
        let scenario = Ecommerce::with_theme(
            42,
            1,
            precompute(date(2023, 1, 1), 1).unwrap(),
            Theme::load("fantasy_rpg").unwrap(),
        )
        .unwrap();
        let rows = scenario.static_rows().unwrap();
        let stores: Vec<_> = rows
            .iter()
            .filter(|(entity, _)| *entity == "stores")
            .map(|(_, row)| row[1].clone())
            .collect();
        assert_eq!(
            stores,
            [
                "Thornwall",
                "Misthollow",
                "Ironvale",
                "Starfen",
                "Duskmarsh",
                "Sunspire"
            ]
            .map(text)
        );
        let product = &rows
            .iter()
            .find(|(entity, _)| *entity == "products")
            .unwrap()
            .1;
        assert_eq!(
            product,
            &vec![
                text("WEP-001"),
                text("wyrmfang edge"),
                text("weapon"),
                Value::Cents(1100),
                text("iron short sword tempered in drake fire"),
                text("common")
            ]
        );
        let supply = &rows
            .iter()
            .find(|(entity, _)| *entity == "supplies")
            .unwrap()
            .1;
        assert_eq!(
            supply,
            &vec![
                text("SUP-001"),
                text("enchanted wrapping cloth"),
                Value::Cents(7),
                Value::Boolean(false),
                text("Thornwall"),
                text("WEP-001")
            ]
        );
    }

    #[test]
    fn pools_have_exact_tam_and_indexed_identity() {
        let days = precompute(date(2023, 1, 1), 400).unwrap();
        let small = Ecommerce::new(42, 1, days.clone()).unwrap();
        let large = Ecommerce::new(42, 2, days).unwrap();
        for (index, store) in STORES.iter().enumerate() {
            assert_eq!(small.customers[index].len(), store.tam);
            assert_eq!(large.customers[index].len(), store.tam * 2);
            assert_eq!(small.customers[index][0], large.customers[index][0]);
        }
    }

    #[test]
    fn each_complete_customer_block_has_the_fixed_persona_mix() {
        for seed in [0, 42, 123] {
            let scenario =
                Ecommerce::new(seed, 10, precompute(date(2023, 1, 1), 1).unwrap()).unwrap();
            for pool in &scenario.customers {
                for block in pool.as_chunks::<20>().0 {
                    let counts = [
                        Persona::Courier,
                        Persona::Artificer,
                        Persona::FeastReveler,
                        Persona::Apprentice,
                        Persona::Wanderer,
                        Persona::Herbalist,
                    ]
                    .map(|persona| {
                        block
                            .iter()
                            .filter(|customer| customer.persona == persona)
                            .count()
                    });
                    assert_eq!(counts, [5, 5, 2, 4, 2, 2]);
                }
            }
        }
    }

    #[test]
    fn market_day_repeats_and_orders_reconcile_with_items() {
        let scenario = Ecommerce::new(42, 10, precompute(date(2023, 1, 1), 400).unwrap()).unwrap();
        let unit = WorkUnit {
            stage: 1,
            indices: [365, 0],
        };
        let rows = scenario.generate(42, unit).unwrap();
        assert!(!rows.is_empty());
        assert_eq!(rows, scenario.generate(42, unit).unwrap());
        for (_, order) in rows.iter().filter(|(entity, _)| *entity == "orders") {
            let items: Vec<_> = rows
                .iter()
                .filter(|(entity, row)| *entity == "items" && row[1] == order[0])
                .collect();
            assert!(!items.is_empty());
            let subtotal: i64 = items
                .iter()
                .map(|(_, item)| {
                    PRODUCTS
                        .iter()
                        .find(|p| Value::Text(p.sku.into()) == item[2])
                        .unwrap()
                        .price
                })
                .sum();
            assert_eq!(order[4], Value::Cents(subtotal));
            let Value::Cents(tax) = order[5] else {
                panic!()
            };
            assert_eq!(order[6], Value::Cents(subtotal + tax));
        }
    }

    #[test]
    fn ranks_balance_emitted_customer_counts_and_break_ties_by_uuid() {
        let mut scenario = Ecommerce::new(42, 1, precompute(date(2023, 1, 1), 1).unwrap()).unwrap();
        for i in 0..11u8 {
            scenario.counts.insert([i; 16], u64::from(i / 2 + 1));
        }
        scenario.complete_stage(1).unwrap();
        let mut sizes = [0; 4];
        let mut prior = 0;
        for rank in scenario.ranks.values() {
            sizes[*rank] += 1;
            assert!(*rank >= prior);
            prior = *rank;
        }
        assert!(sizes.iter().max().unwrap() - sizes.iter().min().unwrap() <= 1);
    }

    #[test]
    fn stages_emit_only_ordering_customers_and_ranked_sparrows_after_orders() {
        let mut scenario =
            Ecommerce::new(42, 2, precompute(date(2023, 1, 1), 400).unwrap()).unwrap();
        let mut orders = BTreeMap::<[u8; 16], Vec<i64>>::new();
        let mut observed = BTreeMap::<[u8; 16], u64>::new();
        let mut customer_ids = std::collections::BTreeSet::new();
        let mut sparrow_count = 0;
        let units = scenario.units();
        for unit in units.iter().filter(|unit| unit.stage == 1) {
            let rows = scenario.generate(42, *unit).unwrap();
            for (_, row) in rows.iter().filter(|(entity, _)| *entity == "orders") {
                let (Value::Uuid(customer), Value::Timestamp(timestamp)) = (&row[1], &row[2])
                else {
                    panic!()
                };
                orders.entry(*customer).or_default().push(*timestamp);
                *observed.entry(*customer).or_default() += 1;
                let minute = (timestamp / 60_000_000).rem_euclid(1440);
                let day = &scenario.days[usize::try_from(unit.indices[0]).unwrap()];
                assert!((i64::from(day.opens_at)..i64::from(day.closes_at)).contains(&minute));
            }
            scenario.observe(&rows).unwrap();
            assert_eq!(rows, scenario.generate(42, *unit).unwrap());
        }
        assert_eq!(scenario.counts, observed);
        scenario.complete_stage(1).unwrap();
        for unit in units.iter().filter(|unit| unit.stage >= 2) {
            for (entity, row) in scenario.generate(42, *unit).unwrap() {
                if entity == "customers" {
                    let Value::Uuid(id) = row[0] else { panic!() };
                    assert!(orders.contains_key(&id));
                    assert!(customer_ids.insert(id));
                } else {
                    assert_eq!(entity, "sparrows");
                    let (Value::Uuid(id), Value::Timestamp(sent), Value::Text(content)) =
                        (&row[1], &row[2], &row[3])
                    else {
                        panic!()
                    };
                    assert!(
                        orders[id]
                            .iter()
                            .any(|ordered| (0..20 * 60_000_000).contains(&(sent - ordered)))
                    );
                    assert!(
                        content
                            .starts_with(scenario.theme.label("rank_voices", scenario.ranks[id]))
                    );
                    sparrow_count += 1;
                }
            }
        }
        assert!(sparrow_count > 0);
        assert_eq!(customer_ids.len(), orders.len());
    }

    #[test]
    fn opening_day_has_no_orders_and_catalogs_have_six_fifteen_and_ninety_two_rows() {
        let scenario = Ecommerce::new(42, 1, precompute(date(2023, 1, 1), 1).unwrap()).unwrap();
        assert!(
            scenario
                .generate(
                    42,
                    WorkUnit {
                        stage: 1,
                        indices: [0, 0]
                    }
                )
                .unwrap()
                .is_empty()
        );
        let rows = scenario
            .generate(
                42,
                WorkUnit {
                    stage: 0,
                    indices: [0, 0],
                },
            )
            .unwrap();
        for (entity, expected) in [("stores", 6), ("products", 15), ("supplies", 92)] {
            assert_eq!(
                rows.iter().filter(|(name, _)| *name == entity).count(),
                expected
            );
        }
        assert!(Ecommerce::new(42, usize::MAX, scenario.days.clone()).is_err());
    }
}
