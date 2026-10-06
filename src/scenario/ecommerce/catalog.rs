#[derive(Debug, Clone, Copy)]
pub(super) struct Product {
    pub sku: &'static str,
    pub category: usize,
    pub tier: usize,
}

pub(super) const PRODUCTS: [Product; 15] = [
    Product {
        sku: "WEP-001",
        category: 0,
        tier: 0,
    },
    Product {
        sku: "WEP-002",
        category: 0,
        tier: 1,
    },
    Product {
        sku: "WEP-003",
        category: 0,
        tier: 2,
    },
    Product {
        sku: "WEP-004",
        category: 0,
        tier: 3,
    },
    Product {
        sku: "WEP-005",
        category: 0,
        tier: 4,
    },
    Product {
        sku: "ARM-001",
        category: 1,
        tier: 0,
    },
    Product {
        sku: "ARM-002",
        category: 1,
        tier: 1,
    },
    Product {
        sku: "ARM-003",
        category: 1,
        tier: 2,
    },
    Product {
        sku: "ARM-004",
        category: 1,
        tier: 3,
    },
    Product {
        sku: "ARM-005",
        category: 1,
        tier: 4,
    },
    Product {
        sku: "ELX-001",
        category: 2,
        tier: 0,
    },
    Product {
        sku: "ELX-002",
        category: 2,
        tier: 1,
    },
    Product {
        sku: "ELX-003",
        category: 2,
        tier: 2,
    },
    Product {
        sku: "ELX-004",
        category: 2,
        tier: 3,
    },
    Product {
        sku: "ELX-005",
        category: 2,
        tier: 4,
    },
];

#[derive(Debug, Clone, Copy)]
pub(super) struct Supply {
    pub id: &'static str,
    pub volatile: bool,
    pub origin: usize,
    pub skus: &'static [&'static str],
}

const WEAPON_SKUS: &[&str] = &["WEP-001", "WEP-002", "WEP-003", "WEP-004", "WEP-005"];
const ARMOR_SKUS: &[&str] = &["ARM-001", "ARM-002", "ARM-003", "ARM-004", "ARM-005"];
const ELIXIR_SKUS: &[&str] = &["ELX-001", "ELX-002", "ELX-003", "ELX-004", "ELX-005"];

pub(super) const SUPPLIES: [Supply; 41] = [
    Supply {
        id: "SUP-001",
        volatile: false,
        origin: 0,
        skus: WEAPON_SKUS,
    },
    Supply {
        id: "SUP-002",
        volatile: false,
        origin: 0,
        skus: WEAPON_SKUS,
    },
    Supply {
        id: "SUP-003",
        volatile: false,
        origin: 0,
        skus: WEAPON_SKUS,
    },
    Supply {
        id: "SUP-004",
        volatile: false,
        origin: 0,
        skus: WEAPON_SKUS,
    },
    Supply {
        id: "SUP-005",
        volatile: false,
        origin: 0,
        skus: ARMOR_SKUS,
    },
    Supply {
        id: "SUP-006",
        volatile: false,
        origin: 0,
        skus: ARMOR_SKUS,
    },
    Supply {
        id: "SUP-007",
        volatile: false,
        origin: 0,
        skus: ARMOR_SKUS,
    },
    Supply {
        id: "SUP-008",
        volatile: false,
        origin: 0,
        skus: ELIXIR_SKUS,
    },
    Supply {
        id: "SUP-009",
        volatile: false,
        origin: 0,
        skus: ELIXIR_SKUS,
    },
    Supply {
        id: "SUP-010",
        volatile: false,
        origin: 0,
        skus: ELIXIR_SKUS,
    },
    Supply {
        id: "SUP-011",
        volatile: true,
        origin: 2,
        skus: WEAPON_SKUS,
    },
    Supply {
        id: "SUP-012",
        volatile: true,
        origin: 2,
        skus: &["WEP-001"],
    },
    Supply {
        id: "SUP-013",
        volatile: true,
        origin: 2,
        skus: &["WEP-001"],
    },
    Supply {
        id: "SUP-014",
        volatile: true,
        origin: 5,
        skus: &["WEP-002"],
    },
    Supply {
        id: "SUP-015",
        volatile: true,
        origin: 2,
        skus: &["WEP-003"],
    },
    Supply {
        id: "SUP-016",
        volatile: true,
        origin: 2,
        skus: &["WEP-003"],
    },
    Supply {
        id: "SUP-017",
        volatile: true,
        origin: 2,
        skus: &["WEP-004"],
    },
    Supply {
        id: "SUP-018",
        volatile: true,
        origin: 5,
        skus: &["WEP-004"],
    },
    Supply {
        id: "SUP-019",
        volatile: true,
        origin: 2,
        skus: &["WEP-004"],
    },
    Supply {
        id: "SUP-020",
        volatile: true,
        origin: 3,
        skus: &["WEP-005"],
    },
    Supply {
        id: "SUP-021",
        volatile: true,
        origin: 3,
        skus: &["WEP-005"],
    },
    Supply {
        id: "SUP-022",
        volatile: true,
        origin: 0,
        skus: ARMOR_SKUS,
    },
    Supply {
        id: "SUP-023",
        volatile: true,
        origin: 2,
        skus: &["ARM-001", "ARM-002", "ARM-003"],
    },
    Supply {
        id: "SUP-024",
        volatile: true,
        origin: 0,
        skus: &["ARM-001"],
    },
    Supply {
        id: "SUP-025",
        volatile: true,
        origin: 1,
        skus: &["ARM-002"],
    },
    Supply {
        id: "SUP-026",
        volatile: true,
        origin: 1,
        skus: &["ARM-002"],
    },
    Supply {
        id: "SUP-027",
        volatile: true,
        origin: 5,
        skus: &["ARM-003"],
    },
    Supply {
        id: "SUP-028",
        volatile: true,
        origin: 5,
        skus: &["ARM-003"],
    },
    Supply {
        id: "SUP-029",
        volatile: true,
        origin: 5,
        skus: &["ARM-004"],
    },
    Supply {
        id: "SUP-030",
        volatile: true,
        origin: 2,
        skus: &["ARM-004"],
    },
    Supply {
        id: "SUP-031",
        volatile: true,
        origin: 3,
        skus: &["ARM-005"],
    },
    Supply {
        id: "SUP-032",
        volatile: true,
        origin: 3,
        skus: &["ARM-005"],
    },
    Supply {
        id: "SUP-033",
        volatile: true,
        origin: 4,
        skus: &["ELX-001"],
    },
    Supply {
        id: "SUP-034",
        volatile: true,
        origin: 4,
        skus: &["ELX-001"],
    },
    Supply {
        id: "SUP-035",
        volatile: true,
        origin: 5,
        skus: &["ELX-002"],
    },
    Supply {
        id: "SUP-036",
        volatile: true,
        origin: 0,
        skus: &["ELX-002"],
    },
    Supply {
        id: "SUP-037",
        volatile: true,
        origin: 3,
        skus: &["ELX-002"],
    },
    Supply {
        id: "SUP-038",
        volatile: true,
        origin: 4,
        skus: &["ELX-003", "ELX-004"],
    },
    Supply {
        id: "SUP-039",
        volatile: true,
        origin: 1,
        skus: &["ELX-003"],
    },
    Supply {
        id: "SUP-040",
        volatile: true,
        origin: 0,
        skus: &["ELX-005"],
    },
    Supply {
        id: "SUP-041",
        volatile: true,
        origin: 4,
        skus: &["ELX-005"],
    },
];

#[derive(Debug, Clone, Copy)]
pub(super) struct Store {
    pub popularity: f64,
    pub opens: usize,
    pub tam: usize,
    pub tax_rate: f64,
}

pub(super) const STORES: [Store; 6] = [
    Store {
        popularity: 0.85,
        opens: 0,
        tam: 9,
        tax_rate: 0.0600,
    },
    Store {
        popularity: 0.95,
        opens: 192,
        tam: 14,
        tax_rate: 0.0400,
    },
    Store {
        popularity: 0.92,
        opens: 605,
        tam: 12,
        tax_rate: 0.0625,
    },
    Store {
        popularity: 0.87,
        opens: 615,
        tam: 11,
        tax_rate: 0.0750,
    },
    Store {
        popularity: 0.92,
        opens: 920,
        tam: 8,
        tax_rate: 0.0400,
    },
    Store {
        popularity: 0.87,
        opens: 1107,
        tam: 8,
        tax_rate: 0.0800,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn each_product_category_covers_every_type_once() {
        let types = BTreeSet::from([0, 1, 2, 3, 4]);
        for (category, prefix) in [(0, "WEP"), (1, "ARM"), (2, "ELX")] {
            let products: Vec<_> = PRODUCTS
                .iter()
                .filter(|product| product.category == category)
                .collect();
            assert_eq!(products.len(), 5);
            assert_eq!(
                products
                    .iter()
                    .map(|product| product.tier)
                    .collect::<BTreeSet<_>>(),
                types
            );
            for (index, product) in products.iter().enumerate() {
                assert_eq!(product.sku, format!("{prefix}-{:03}", index + 1));
            }
        }
    }

    #[test]
    fn supplies_form_92_unique_valid_product_relations() {
        let mut relations = BTreeSet::new();
        for (index, supply) in SUPPLIES.iter().enumerate() {
            assert_eq!(supply.id, format!("SUP-{:03}", index + 1));
            assert!(!supply.skus.is_empty());
            assert_eq!(supply.volatile, index >= 10);
            assert!(supply.origin < 6);
            for sku in supply.skus {
                assert!(PRODUCTS.iter().any(|product| product.sku == *sku));
                assert!(relations.insert((supply.id, sku)));
            }
        }
        assert_eq!(relations.len(), 92);
        for product in PRODUCTS {
            assert!(relations.iter().any(|(_, sku)| **sku == product.sku));
        }
    }

    #[test]
    fn store_stream_indices_follow_the_fixed_opening_sequence() {
        assert_eq!(
            STORES.map(|store| store.opens),
            [0, 192, 605, 615, 920, 1107]
        );
        assert_eq!(STORES.map(|store| store.tam), [9, 14, 12, 11, 8, 8]);
        for store in STORES {
            assert!((0.0..=1.0).contains(&store.popularity));
            assert!((0.0..=1.0).contains(&store.tax_rate));
        }
    }
}
