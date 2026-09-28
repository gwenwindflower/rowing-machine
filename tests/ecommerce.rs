use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use assert_cmd::cargo::cargo_bin_cmd;
use csv::StringRecord;

const SCHEMAS: [(&str, &str); 7] = [
    ("stores", "id,name,opened_at,tax_rate"),
    ("customers", "id,name,guild_rank"),
    (
        "orders",
        "id,customer,ordered_at,store_id,subtotal,tax_paid,order_total",
    ),
    ("items", "id,order_id,sku"),
    ("products", "sku,name,type,price,description,power_level"),
    ("supplies", "id,name,cost,volatile,origin_region,sku"),
    ("sparrows", "id,user_id,sent_at,content"),
];

fn generate(directory: &Path, seed: &str) {
    cargo_bin_cmd!("rowing-machine")
        .args([
            "--years",
            "4",
            "--scale",
            "1",
            "--seed",
            seed,
            "--quiet",
            "--output-dir",
        ])
        .arg(directory)
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

fn read_entity(directory: &Path, entity: &str, header: &str) -> Vec<StringRecord> {
    let mut reader = csv::Reader::from_path(directory.join(format!("raw_{entity}.csv"))).unwrap();
    assert_eq!(
        reader.headers().unwrap(),
        &StringRecord::from(header.split(',').collect::<Vec<_>>())
    );
    reader.records().map(Result::unwrap).collect()
}

fn valid_uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            if [8, 13, 18, 23].contains(&index) {
                byte == b'-'
            } else {
                byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
            }
        })
        && value.as_bytes()[14] == b'4'
        && b"89ab".contains(&value.as_bytes()[19])
}

#[test]
fn seed_reproduces_every_file_and_a_different_seed_changes_orders() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let third = tempfile::tempdir().unwrap();
    generate(first.path(), "42");
    generate(second.path(), "42");
    generate(third.path(), "43");
    for (entity, _) in SCHEMAS {
        let filename = format!("raw_{entity}.csv");
        assert_eq!(
            std::fs::read(first.path().join(&filename)).unwrap(),
            std::fs::read(second.path().join(&filename)).unwrap(),
            "{entity}"
        );
    }
    assert_ne!(
        std::fs::read(first.path().join("raw_orders.csv")).unwrap(),
        std::fs::read(third.path().join("raw_orders.csv")).unwrap()
    );
}

#[test]
fn ecommerce_files_preserve_schemas_relations_money_ranks_and_unit_order() {
    let directory = tempfile::tempdir().unwrap();
    generate(directory.path(), "42");
    let tables: BTreeMap<_, _> = SCHEMAS
        .into_iter()
        .map(|(entity, header)| (entity, read_entity(directory.path(), entity, header)))
        .collect();
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 7);
    for (entity, expected) in [("stores", 6), ("products", 15), ("supplies", 92)] {
        assert_eq!(tables[entity].len(), expected, "{entity}");
    }
    for (entity, rows) in &tables {
        assert!(!rows.is_empty(), "{entity}");
        let mut keys = BTreeSet::new();
        for row in rows {
            assert!(
                row.iter().all(|field| !field.is_empty()),
                "{entity}: {row:?}"
            );
            let key = if *entity == "supplies" {
                vec![&row[0], &row[5]]
            } else {
                vec![&row[0]]
            };
            assert!(keys.insert(key), "duplicate {entity} key: {row:?}");
            if !["products", "supplies"].contains(entity) {
                assert!(valid_uuid(&row[0]), "{entity}: {}", &row[0]);
            }
        }
    }
    let products: BTreeMap<_, _> = tables["products"]
        .iter()
        .map(|row| (&row[0], row[3].parse::<i64>().unwrap()))
        .collect();
    let stores: BTreeMap<_, _> = tables["stores"]
        .iter()
        .enumerate()
        .map(|(index, row)| (&row[0], (index, row)))
        .collect();
    let customers: BTreeMap<_, _> = tables["customers"]
        .iter()
        .map(|row| (&row[0], &row[2]))
        .collect();
    let orders: BTreeMap<_, _> = tables["orders"].iter().map(|row| (&row[0], row)).collect();
    let mut subtotals = BTreeMap::<&str, i64>::new();
    for item in &tables["items"] {
        assert!(orders.contains_key(&item[1]));
        *subtotals.entry(&item[1]).or_default() += products[&item[2]];
    }
    for supply in &tables["supplies"] {
        assert!(products.contains_key(&supply[5]));
        assert!(supply[2].parse::<i64>().unwrap() > 0);
        assert!(["True", "False"].contains(&&supply[3]));
    }
    let mut order_counts = BTreeMap::<&str, usize>::new();
    let mut previous_unit = None;
    for order in &tables["orders"] {
        assert!(customers.contains_key(&order[1]));
        let (store_index, store) = stores[&order[3]];
        let unit = (&order[2][..10], store_index);
        if let Some(previous) = previous_unit {
            assert!(
                previous <= unit,
                "units out of order: {previous:?}, {unit:?}"
            );
        }
        previous_unit = Some(unit);
        assert!(order[2] >= store[2]);
        let subtotal = order[4].parse::<i64>().unwrap();
        let tax = order[5].parse::<i64>().unwrap();
        let total = order[6].parse::<i64>().unwrap();
        let basis_points = format!("{:.0}", store[3].parse::<f64>().unwrap() * 10_000.0)
            .parse::<i64>()
            .unwrap();
        assert_eq!(subtotal, subtotals[&order[0]]);
        assert_eq!(tax, (subtotal * basis_points + 5_000) / 10_000);
        assert_eq!(total, subtotal + tax);
        *order_counts.entry(&order[1]).or_default() += 1;
    }
    for sparrow in &tables["sparrows"] {
        assert!(customers.contains_key(&sparrow[1]));
    }
    assert_customer_cohorts(&customers, order_counts);
}

fn assert_customer_cohorts(customers: &BTreeMap<&str, &str>, order_counts: BTreeMap<&str, usize>) {
    assert_eq!(order_counts.len(), customers.len());
    let mut ordered_customers: Vec<_> = order_counts
        .into_iter()
        .map(|(id, count)| (count, id))
        .collect();
    ordered_customers.sort_unstable();
    let ranks = ["initiate", "journeyman", "adept", "master"];
    let mut rank_sizes = [0_usize; 4];
    let mut previous_rank = 0;
    for (_, id) in ordered_customers {
        let rank = ranks
            .iter()
            .position(|rank| *rank == customers[id])
            .unwrap();
        assert!(
            rank >= previous_rank,
            "rank decreases in order-count and UUID order"
        );
        rank_sizes[rank] += 1;
        previous_rank = rank;
    }
    assert!(rank_sizes.iter().max().unwrap() - rank_sizes.iter().min().unwrap() <= 1);
}
