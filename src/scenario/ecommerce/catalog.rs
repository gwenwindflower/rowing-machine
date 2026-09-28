#[derive(Debug, Clone, Copy)]
pub(super) struct Product {
    pub sku: &'static str,
    pub name: &'static str,
    pub kind: &'static str,
    pub price: i64,
    pub description: &'static str,
    pub power_level: &'static str,
}

pub(super) const PRODUCTS: [Product; 15] = [
    Product {
        sku: "WEP-001",
        name: "wyrmfang edge",
        kind: "weapon",
        price: 1100,
        description: "iron short sword tempered in drake fire",
        power_level: "common",
    },
    Product {
        sku: "WEP-002",
        name: "stormcaller bow",
        kind: "weapon",
        price: 1100,
        description: "recurve bow strung with thunderhawk sinew",
        power_level: "uncommon",
    },
    Product {
        sku: "WEP-003",
        name: "emberveil dagger",
        kind: "weapon",
        price: 1200,
        description: "obsidian blade that trails embers when drawn",
        power_level: "rare",
    },
    Product {
        sku: "WEP-004",
        name: "inferno maul",
        kind: "weapon",
        price: 1400,
        description: "warhammer forged in volcanic glass with phoenix core",
        power_level: "epic",
    },
    Product {
        sku: "WEP-005",
        name: "void sigil staff",
        kind: "weapon",
        price: 1200,
        description: "quarterstaff inscribed with a dimensional rift glyph",
        power_level: "legendary",
    },
    Product {
        sku: "ARM-001",
        name: "ironbark buckler",
        kind: "armor",
        price: 800,
        description: "small round shield carved from ironbark heartwood",
        power_level: "common",
    },
    Product {
        sku: "ARM-002",
        name: "glacial bulwark",
        kind: "armor",
        price: 1200,
        description: "tower shield reinforced with permafrost oak",
        power_level: "uncommon",
    },
    Product {
        sku: "ARM-003",
        name: "drake scale cuirass",
        kind: "armor",
        price: 1500,
        description: "chest plate layered with drake scale resin",
        power_level: "rare",
    },
    Product {
        sku: "ARM-004",
        name: "phoenix ward mantle",
        kind: "armor",
        price: 1800,
        description: "shoulder guard woven with phoenix feathers",
        power_level: "epic",
    },
    Product {
        sku: "ARM-005",
        name: "voidweave vestments",
        kind: "armor",
        price: 2000,
        description: "robes threaded with dimensional rift silk",
        power_level: "legendary",
    },
    Product {
        sku: "ELX-001",
        name: "sunfire tonic",
        kind: "elixir",
        price: 600,
        description: "mango and tangerine essence energy brew",
        power_level: "common",
    },
    Product {
        sku: "ELX-002",
        name: "ironbark draught",
        kind: "elixir",
        price: 500,
        description: "oatmilk and spice fortification potion",
        power_level: "uncommon",
    },
    Product {
        sku: "ELX-003",
        name: "frostmint vial",
        kind: "elixir",
        price: 600,
        description: "chilled coffee infused with vanilla frost crystals",
        power_level: "rare",
    },
    Product {
        sku: "ELX-004",
        name: "oracle's brew",
        kind: "elixir",
        price: 700,
        description: "single-origin bean vision-enhancing elixir",
        power_level: "epic",
    },
    Product {
        sku: "ELX-005",
        name: "serpent's kiss",
        kind: "elixir",
        price: 400,
        description: "kiwi and lime venom-neutralizing tincture",
        power_level: "legendary",
    },
];

#[derive(Debug, Clone, Copy)]
pub(super) struct Supply {
    pub id: &'static str,
    pub name: &'static str,
    pub cost: i64,
    pub volatile: bool,
    pub origin_region: &'static str,
    pub skus: &'static [&'static str],
}

const WEAPON_SKUS: &[&str] = &["WEP-001", "WEP-002", "WEP-003", "WEP-004", "WEP-005"];
const ARMOR_SKUS: &[&str] = &["ARM-001", "ARM-002", "ARM-003", "ARM-004", "ARM-005"];
const ELIXIR_SKUS: &[&str] = &["ELX-001", "ELX-002", "ELX-003", "ELX-004", "ELX-005"];

pub(super) const SUPPLIES: [Supply; 41] = [
    Supply {
        id: "SUP-001",
        name: "enchanted wrapping cloth",
        cost: 7,
        volatile: false,
        origin_region: "Thornwall",
        skus: WEAPON_SKUS,
    },
    Supply {
        id: "SUP-002",
        name: "binding rune seal",
        cost: 7,
        volatile: false,
        origin_region: "Thornwall",
        skus: WEAPON_SKUS,
    },
    Supply {
        id: "SUP-003",
        name: "display crystal case",
        cost: 11,
        volatile: false,
        origin_region: "Thornwall",
        skus: WEAPON_SKUS,
    },
    Supply {
        id: "SUP-004",
        name: "arcane parchment label",
        cost: 4,
        volatile: false,
        origin_region: "Thornwall",
        skus: WEAPON_SKUS,
    },
    Supply {
        id: "SUP-005",
        name: "reinforcement stitching",
        cost: 7,
        volatile: false,
        origin_region: "Thornwall",
        skus: ARMOR_SKUS,
    },
    Supply {
        id: "SUP-006",
        name: "fitting buckles",
        cost: 7,
        volatile: false,
        origin_region: "Thornwall",
        skus: ARMOR_SKUS,
    },
    Supply {
        id: "SUP-007",
        name: "padding lining",
        cost: 11,
        volatile: false,
        origin_region: "Thornwall",
        skus: ARMOR_SKUS,
    },
    Supply {
        id: "SUP-008",
        name: "glass vial - large",
        cost: 13,
        volatile: false,
        origin_region: "Thornwall",
        skus: ELIXIR_SKUS,
    },
    Supply {
        id: "SUP-009",
        name: "cork stopper - waxed",
        cost: 4,
        volatile: false,
        origin_region: "Thornwall",
        skus: ELIXIR_SKUS,
    },
    Supply {
        id: "SUP-010",
        name: "silkworm thread seal",
        cost: 13,
        volatile: false,
        origin_region: "Thornwall",
        skus: ELIXIR_SKUS,
    },
    Supply {
        id: "SUP-011",
        name: "iron ore ingot",
        cost: 33,
        volatile: true,
        origin_region: "Ironvale",
        skus: WEAPON_SKUS,
    },
    Supply {
        id: "SUP-012",
        name: "drake fire ember",
        cost: 46,
        volatile: true,
        origin_region: "Ironvale",
        skus: &["WEP-001"],
    },
    Supply {
        id: "SUP-013",
        name: "raw iron bar",
        cost: 13,
        volatile: true,
        origin_region: "Ironvale",
        skus: &["WEP-001"],
    },
    Supply {
        id: "SUP-014",
        name: "thunderhawk sinew",
        cost: 169,
        volatile: true,
        origin_region: "Sunspire",
        skus: &["WEP-002"],
    },
    Supply {
        id: "SUP-015",
        name: "emberveil obsidian",
        cost: 234,
        volatile: true,
        origin_region: "Ironvale",
        skus: &["WEP-003"],
    },
    Supply {
        id: "SUP-016",
        name: "flickerflame oil",
        cost: 43,
        volatile: true,
        origin_region: "Ironvale",
        skus: &["WEP-003"],
    },
    Supply {
        id: "SUP-017",
        name: "volcanic glass shard",
        cost: 215,
        volatile: true,
        origin_region: "Ironvale",
        skus: &["WEP-004"],
    },
    Supply {
        id: "SUP-018",
        name: "phoenix core feather",
        cost: 26,
        volatile: true,
        origin_region: "Sunspire",
        skus: &["WEP-004"],
    },
    Supply {
        id: "SUP-019",
        name: "molten fire salt",
        cost: 20,
        volatile: true,
        origin_region: "Ironvale",
        skus: &["WEP-004"],
    },
    Supply {
        id: "SUP-020",
        name: "void crystal",
        cost: 33,
        volatile: true,
        origin_region: "Starfen",
        skus: &["WEP-005"],
    },
    Supply {
        id: "SUP-021",
        name: "dimensional rift ink",
        cost: 124,
        volatile: true,
        origin_region: "Starfen",
        skus: &["WEP-005"],
    },
    Supply {
        id: "SUP-022",
        name: "leather strapping",
        cost: 20,
        volatile: true,
        origin_region: "Thornwall",
        skus: ARMOR_SKUS,
    },
    Supply {
        id: "SUP-023",
        name: "steel rivets",
        cost: 7,
        volatile: true,
        origin_region: "Ironvale",
        skus: &["ARM-001", "ARM-002", "ARM-003"],
    },
    Supply {
        id: "SUP-024",
        name: "ironbark heartwood",
        cost: 98,
        volatile: true,
        origin_region: "Thornwall",
        skus: &["ARM-001"],
    },
    Supply {
        id: "SUP-025",
        name: "permafrost oak plank",
        cost: 234,
        volatile: true,
        origin_region: "Misthollow",
        skus: &["ARM-002"],
    },
    Supply {
        id: "SUP-026",
        name: "glacial ward rune",
        cost: 43,
        volatile: true,
        origin_region: "Misthollow",
        skus: &["ARM-002"],
    },
    Supply {
        id: "SUP-027",
        name: "drake scale plates",
        cost: 215,
        volatile: true,
        origin_region: "Sunspire",
        skus: &["ARM-003"],
    },
    Supply {
        id: "SUP-028",
        name: "drake scale resin",
        cost: 20,
        volatile: true,
        origin_region: "Sunspire",
        skus: &["ARM-003"],
    },
    Supply {
        id: "SUP-029",
        name: "phoenix down plume",
        cost: 169,
        volatile: true,
        origin_region: "Sunspire",
        skus: &["ARM-004"],
    },
    Supply {
        id: "SUP-030",
        name: "fireweave thread",
        cost: 72,
        volatile: true,
        origin_region: "Ironvale",
        skus: &["ARM-004"],
    },
    Supply {
        id: "SUP-031",
        name: "dimensional rift silk",
        cost: 234,
        volatile: true,
        origin_region: "Starfen",
        skus: &["ARM-005"],
    },
    Supply {
        id: "SUP-032",
        name: "void essence dye",
        cost: 124,
        volatile: true,
        origin_region: "Starfen",
        skus: &["ARM-005"],
    },
    Supply {
        id: "SUP-033",
        name: "sunfruit pulp",
        cost: 32,
        volatile: true,
        origin_region: "Duskmarsh",
        skus: &["ELX-001"],
    },
    Supply {
        id: "SUP-034",
        name: "citrine essence",
        cost: 20,
        volatile: true,
        origin_region: "Duskmarsh",
        skus: &["ELX-001"],
    },
    Supply {
        id: "SUP-035",
        name: "stormspice blend",
        cost: 98,
        volatile: true,
        origin_region: "Sunspire",
        skus: &["ELX-002"],
    },
    Supply {
        id: "SUP-036",
        name: "ironbark sap",
        cost: 11,
        volatile: true,
        origin_region: "Thornwall",
        skus: &["ELX-002"],
    },
    Supply {
        id: "SUP-037",
        name: "basilisk bone powder",
        cost: 36,
        volatile: true,
        origin_region: "Starfen",
        skus: &["ELX-002"],
    },
    Supply {
        id: "SUP-038",
        name: "shadowbean grounds",
        cost: 52,
        volatile: true,
        origin_region: "Duskmarsh",
        skus: &["ELX-003", "ELX-004"],
    },
    Supply {
        id: "SUP-039",
        name: "vanilla frost crystal",
        cost: 72,
        volatile: true,
        origin_region: "Misthollow",
        skus: &["ELX-003"],
    },
    Supply {
        id: "SUP-040",
        name: "emerald kiwi extract",
        cost: 20,
        volatile: true,
        origin_region: "Thornwall",
        skus: &["ELX-005"],
    },
    Supply {
        id: "SUP-041",
        name: "serpent lime juice",
        cost: 13,
        volatile: true,
        origin_region: "Duskmarsh",
        skus: &["ELX-005"],
    },
];

#[derive(Debug, Clone, Copy)]
pub(super) struct Store {
    pub name: &'static str,
    pub popularity: f64,
    pub opens: usize,
    pub tam: usize,
    pub tax_rate: f64,
}

pub(super) const STORES: [Store; 6] = [
    Store {
        name: "Thornwall",
        popularity: 0.85,
        opens: 0,
        tam: 9,
        tax_rate: 0.0600,
    },
    Store {
        name: "Misthollow",
        popularity: 0.95,
        opens: 192,
        tam: 14,
        tax_rate: 0.0400,
    },
    Store {
        name: "Ironvale",
        popularity: 0.92,
        opens: 605,
        tam: 12,
        tax_rate: 0.0625,
    },
    Store {
        name: "Starfen",
        popularity: 0.87,
        opens: 615,
        tam: 11,
        tax_rate: 0.0750,
    },
    Store {
        name: "Duskmarsh",
        popularity: 0.92,
        opens: 920,
        tam: 8,
        tax_rate: 0.0400,
    },
    Store {
        name: "Sunspire",
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
    fn each_product_type_covers_every_rarity_once() {
        let rarities = BTreeSet::from(["common", "uncommon", "rare", "epic", "legendary"]);
        for (kind, prefix) in [("weapon", "WEP"), ("armor", "ARM"), ("elixir", "ELX")] {
            let products: Vec<_> = PRODUCTS
                .iter()
                .filter(|product| product.kind == kind)
                .collect();
            assert_eq!(products.len(), 5);
            assert_eq!(
                products
                    .iter()
                    .map(|product| product.power_level)
                    .collect::<BTreeSet<_>>(),
                rarities
            );
            for (index, product) in products.iter().enumerate() {
                assert_eq!(product.sku, format!("{prefix}-{:03}", index + 1));
                assert!(product.price > 0);
                assert!(!product.name.is_empty());
                assert!(!product.description.is_empty());
            }
        }
    }

    #[test]
    fn supplies_form_92_unique_valid_product_relations() {
        let mut relations = BTreeSet::new();
        for (index, supply) in SUPPLIES.iter().enumerate() {
            assert_eq!(supply.id, format!("SUP-{:03}", index + 1));
            assert!(supply.cost > 0);
            assert!(!supply.name.is_empty());
            assert!(!supply.skus.is_empty());
            assert_eq!(supply.volatile, index >= 10);
            assert!(
                STORES
                    .iter()
                    .any(|store| store.name == supply.origin_region)
            );
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
            STORES.map(|store| store.name),
            [
                "Thornwall",
                "Misthollow",
                "Ironvale",
                "Starfen",
                "Duskmarsh",
                "Sunspire"
            ]
        );
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
