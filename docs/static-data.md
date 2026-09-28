# Static data reference

Hardcoded simulation data. Authoritative source: `src/scenario/ecommerce/catalog.rs`.

## Store configs

| Index | Name | Popularity | Opens (day) | TAM (base) | Tax Rate |
| --- | --- | --- | --- | --- | --- |
| 0 | Thornwall | 0.85 | 0 | 9 | 0.0600 |
| 1 | Misthollow | 0.95 | 192 | 14 | 0.0400 |
| 2 | Ironvale | 0.92 | 605 | 12 | 0.0625 |
| 3 | Starfen | 0.87 | 615 | 11 | 0.0750 |
| 4 | Duskmarsh | 0.92 | 920 | 8 | 0.0400 |
| 5 | Sunspire | 0.87 | 1107 | 8 | 0.0800 |

TAM = base * scale (default scale=100). Index is critical — used as PRNG seed offset.

## Catalog (15 items)

Weapons: WEP-001 to WEP-005. Armor: ARM-001 to ARM-005. Elixirs: ELX-001 to ELX-005.

| SKU | Name | Cents | Type | Power Level |
| --- | --- | --- | --- | --- |
| WEP-001 | wyrmfang edge | 1100 | weapon | common |
| WEP-002 | stormcaller bow | 1100 | weapon | uncommon |
| WEP-003 | emberveil dagger | 1200 | weapon | rare |
| WEP-004 | inferno maul | 1400 | weapon | epic |
| WEP-005 | void sigil staff | 1200 | weapon | legendary |
| ARM-001 | ironbark buckler | 800 | armor | common |
| ARM-002 | glacial bulwark | 1200 | armor | uncommon |
| ARM-003 | drake scale cuirass | 1500 | armor | rare |
| ARM-004 | phoenix ward mantle | 1800 | armor | epic |
| ARM-005 | voidweave vestments | 2000 | armor | legendary |
| ELX-001 | sunfire tonic | 600 | elixir | common |
| ELX-002 | ironbark draught | 500 | elixir | uncommon |
| ELX-003 | frostmint vial | 600 | elixir | rare |
| ELX-004 | oracle's brew | 700 | elixir | epic |
| ELX-005 | serpent's kiss | 400 | elixir | legendary |

## Supplies / reagents (41 items)

`SUP-001` through `SUP-041` map shared components and product-specific reagents to one or more SKUs. Output is denormalized to 92 rows with `(id, sku)` as its composite primary key. The authoritative roster lives in `src/scenario/ecommerce/catalog.rs`.

Supply origins name the guild halls: Thornwall supplies workshop materials and forest reagents, Ironvale supplies mined materials, Sunspire supplies plateau reagents, Starfen supplies void reagents, Misthollow supplies frost reagents, and Duskmarsh supplies wetland reagents.

## Persona mix

| Persona | Weight | Weekday | Weekend | Sparrow | Notes |
| --- | --- | --- | --- | --- | --- |
| Courier | 0.25 | high | 0.001 | 0.20 | Order time: N(450, 30) |
| Artificer | 0.25 | low-med | 0.001 | 0.01 | Order time: N(420, 180) |
| FeastReveler | 0.10 | 0 | med | 0.80 | Weekend-only, largest orders |
| Apprentice | 0.20 | med | med | 0.80 | Absent in summer |
| Wanderer | 0.10 | 0.1 | 0.1 | 0.10 | Constant probability |
| Herbalist | 0.10 | 0.2 | varies | 0.60 | Higher in summer |
