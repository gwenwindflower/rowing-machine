use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, bail};

pub type Statistics = BTreeMap<String, f64>;

pub fn summarize(
    directory: &Path,
    prefix: &str,
    personas: &BTreeMap<String, String>,
) -> Result<Statistics> {
    let mut stats = Statistics::new();
    let mut stores = BTreeMap::new();
    for row in read_rows(directory, prefix, "stores")? {
        stores.insert(
            field(&row, "id")?.to_owned(),
            field(&row, "name")?.to_owned(),
        );
        count(
            &mut stats,
            format!("rows.stores.{}", year(field(&row, "opened_at")?)?),
        );
    }
    let mut order_years = BTreeMap::new();
    let mut customer_years = BTreeMap::<String, String>::new();
    let mut store_totals = BTreeMap::<String, (f64, f64)>::new();
    let mut order_count = 0.0;
    for row in read_rows(directory, prefix, "orders")? {
        let order_year = year(field(&row, "ordered_at")?)?;
        let customer = field(&row, "customer")?;
        order_years.insert(field(&row, "id")?.to_owned(), order_year.to_owned());
        customer_years
            .entry(customer.to_owned())
            .and_modify(|first| {
                if order_year < first.as_str() {
                    order_year.clone_into(first);
                }
            })
            .or_insert_with(|| order_year.to_owned());
        count(&mut stats, format!("rows.orders.{order_year}"));
        if !personas.is_empty() {
            let persona = personas
                .get(customer)
                .with_context(|| format!("missing persona for customer {customer}"))?;
            count(&mut stats, format!("persona_order_share.{persona}"));
        }
        let store_id = field(&row, "store_id")?;
        let store = stores
            .get(store_id)
            .with_context(|| format!("unknown store {store_id}"))?;
        let totals = store_totals.entry(store.clone()).or_default();
        totals.0 += field(&row, "order_total")?
            .parse::<f64>()
            .context("invalid order total")?;
        totals.1 += 1.0;
        order_count += 1.0;
    }
    let mut item_count = 0.0;
    for row in read_rows(directory, prefix, "items")? {
        let order_id = field(&row, "order_id")?;
        let item_year = order_years
            .get(order_id)
            .with_context(|| format!("unknown order {order_id}"))?;
        count(&mut stats, format!("rows.items.{item_year}"));
        item_count += 1.0;
    }
    count_dimensions(directory, prefix, &customer_years, &mut stats)?;
    let mut sparrow_count = 0.0;
    for row in read_rows(directory, prefix, "sparrows")? {
        count(
            &mut stats,
            format!("rows.sparrows.{}", year(field(&row, "sent_at")?)?),
        );
        sparrow_count += 1.0;
    }
    for value in stats
        .iter_mut()
        .filter(|(key, _)| key.starts_with("persona_order_share."))
    {
        *value.1 /= order_count;
    }
    for (store, (total, orders)) in store_totals {
        stats.insert(format!("mean_order_total.{store}"), total / orders);
    }
    let ratio = |numerator| {
        if order_count == 0.0 {
            0.0
        } else {
            numerator / order_count
        }
    };
    stats.insert("mean_items_per_order".to_owned(), ratio(item_count));
    stats.insert("sparrow_rate".to_owned(), ratio(sparrow_count));
    Ok(stats)
}

fn count_dimensions(
    directory: &Path,
    prefix: &str,
    customer_years: &BTreeMap<String, String>,
    stats: &mut Statistics,
) -> Result<()> {
    for row in read_rows(directory, prefix, "customers")? {
        let customer = field(&row, "id")?;
        let first_year = customer_years
            .get(customer)
            .with_context(|| format!("customer {customer} has no orders"))?;
        count(stats, format!("rows.customers.{first_year}"));
        count(stats, format!("guild_rank.{}", field(&row, "guild_rank")?));
    }
    for entity in ["products", "supplies"] {
        stats.insert(format!("rows.{entity}.static"), 0.0);
        for _ in read_rows(directory, prefix, entity)? {
            count(stats, format!("rows.{entity}.static"));
        }
    }
    Ok(())
}

type Record = BTreeMap<String, String>;

fn read_rows(directory: &Path, prefix: &str, entity: &str) -> Result<Vec<Record>> {
    let path = directory.join(format!("{prefix}_{entity}.csv"));
    if !path.exists() {
        return Ok(Vec::new());
    }
    csv::Reader::from_path(&path)
        .with_context(|| format!("reading {}", path.display()))?
        .deserialize()
        .collect::<Result<_, _>>()
        .with_context(|| format!("parsing {}", path.display()))
}

fn field<'a>(record: &'a Record, name: &str) -> Result<&'a str> {
    record
        .get(name)
        .map(String::as_str)
        .with_context(|| format!("missing column {name}"))
}

fn year(timestamp: &str) -> Result<&str> {
    let Some(year) = timestamp.get(..4) else {
        bail!("invalid timestamp {timestamp}")
    };
    year.parse::<u16>()
        .with_context(|| format!("invalid year in {timestamp}"))?;
    Ok(year)
}

fn count(stats: &mut Statistics, key: String) {
    *stats.entry(key).or_default() += 1.0;
}
