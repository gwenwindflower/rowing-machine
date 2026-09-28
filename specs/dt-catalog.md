# Ecommerce catalog

## Goals

The ecommerce scenario's static data: guild halls, products, supplies, guild ranks, and the persona roster. The scenario owns the counts, SKUs, and every number below; names and labels shown are the `fantasy_rpg` theme's, and other themes swap only those (`th-R011`). Persona behavior lives in `sm-simulation.md`.

## Vocabulary

- **Guild hall** — a store location.
- **Sparrow** — a customer-sent message about an order, standing in for a review.
- **Supply** — a reagent or material associated with a product SKU.
- **Power level** — product rarity tier, common through legendary.
- **Guild rank** — customer order-frequency cohort, initiate through master.

## Requirements

### Guild halls

- **dt-R001** — There are exactly six guild halls, opening in a fixed sequence over the timeline.
- **dt-R002** — Guild halls carry these names, base popularity, opening day offset, addressable market base, and tax rate:

  | Index | Name | Popularity | Opens (day) | TAM (base) | Tax rate |
  | --- | --- | --- | --- | --- | --- |
  | 0 | Thornwall | 0.85 | 0 | 9 | 0.0600 |
  | 1 | Misthollow | 0.95 | 192 | 14 | 0.0400 |
  | 2 | Ironvale | 0.92 | 605 | 12 | 0.0625 |
  | 3 | Starfen | 0.87 | 615 | 11 | 0.0750 |
  | 4 | Duskmarsh | 0.92 | 920 | 8 | 0.0400 |
  | 5 | Sunspire | 0.87 | 1107 | 8 | 0.0800 |

- **dt-R003** — A market's customer pool size is its TAM base times `--scale`.
- **dt-R004** — Guild hall index is the market index in stream derivation (`sm-R011`); reordering the halls changes every seeded output.

### Products

- **dt-R005** — There are exactly 15 products: 5 weapons, 5 armor, 5 elixirs.
- **dt-R006** — SKUs follow `WEP-001`–`WEP-005`, `ARM-001`–`ARM-005`, and `ELX-001`–`ELX-005`.
- **dt-R007** — Each product has a `power_level` from `{common, uncommon, rare, epic, legendary}`, and each type covers all five levels exactly once.
- **dt-R008** — Product names, prices in cents, and power levels match `docs/static-data.md`.

### Supplies

- **dt-R009** — There are 41 supplies, `SUP-001`–`SUP-041`; shared components map across product types and product-specific reagents map to one or more SKUs.
- **dt-R010** — Supply output has one row per `(supply, sku)` pair, exactly 92 rows.
- **dt-R011** — Every supply's `origin_region` is one of the six guild hall names.
- **dt-R012** — Every supply has a `volatile` boolean.

### Customers

- **dt-R013** — `guild_rank` is one of `{initiate, journeyman, adept, master}`.
- ~~dt-R014~~ — retired: customer names are theme-owned (`th-R001`–`th-R008`).
- ~~dt-R015~~ — retired: superseded by run-wide unique name generation (`th-R005`).

### Personas

- **dt-R016** — Six personas with these weights, which sum to 1.0:

  | Persona | Weight | Weekday | Weekend | Sparrow |
  | --- | --- | --- | --- | --- |
  | Courier | 0.25 | high | 0.001 | 0.20 |
  | Artificer | 0.25 | low–med | 0.001 | 0.01 |
  | FeastReveler | 0.10 | 0 | med | 0.80 |
  | Apprentice | 0.20 | med | med | 0.80 |
  | Wanderer | 0.10 | 0.1 | 0.1 | 0.10 |
  | Herbalist | 0.10 | 0.2 | varies | 0.60 |

- **dt-R017** — Apprentices are absent in summer and Herbalists buy more in summer; other personas hold steady across seasons.
