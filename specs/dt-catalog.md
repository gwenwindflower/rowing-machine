# Static Catalog — Spec

Hardcoded simulation data: guild halls, products, supplies, persona roster, name pool. Authoritative implementation lives in `internal/catalog/` and `internal/models/store.go`; this spec documents the contract those values must satisfy.

The fantasy vocabulary here is the bundled `fantasy_rpg` flavoring of the underlying ecommerce schema. When Phase 3 lands the theming system, this domain becomes one bundled theme rather than the only one — but for now it is the canonical catalog.

## Requirements

### Guild halls (stores)

- **dt-R001 Fixed guild hall count.** Exactly 6 guild halls. Open in a fixed sequence over the simulation timeline.
- **dt-R002 Guild hall roster.** Names, base popularity, opened-day offset (from epoch), TAM base, and tax rate as below. The implementation in `internal/models/store.go` is authoritative.

    | Index | Name | Popularity | Opens (day) | TAM (base) | Tax Rate |
    | --- | --- | --- | --- | --- | --- |
    | 0 | Thornwall | 0.85 | 0 | 9 | 0.0600 |
    | 1 | Misthollow | 0.95 | 192 | 14 | 0.0400 |
    | 2 | Ironvale | 0.92 | 605 | 12 | 0.0625 |
    | 3 | Starfen | 0.87 | 615 | 11 | 0.0750 |
    | 4 | Duskmarsh | 0.92 | 920 | 8 | 0.0400 |
    | 5 | Sunspire | 0.87 | 1107 | 8 | 0.0800 |

- **dt-R003 TAM scaling.** Effective TAM is `TAM_base * --scale` (default scale 100). Drives the customer pool size per market.
- **dt-R004 Index stability.** Guild hall index is the PRNG offset for per-market seeding (see `sm-R011`) — order MUST NOT change without a corresponding seed-stability migration.

### Products

- **dt-R005 Fixed product count.** Exactly 15 products: 5 weapons, 5 armor, 5 elixirs.
- **dt-R006 SKU format.** SKUs use the prefix-NNN pattern: `WEP-001`–`WEP-005`, `ARM-001`–`ARM-005`, `ELX-001`–`ELX-005`.
- **dt-R007 Power levels.** Each product has a `power_level` from the closed set `{common, uncommon, rare, epic, legendary}`. Within each type, the five SKUs cover the five levels exactly once.
- **dt-R008 Product roster.** Names, prices (cents), and power levels per the implementation in `internal/catalog/inventory.go`. Documented in `docs/static-data.md` for reference.

### Supplies (reagents)

- **dt-R009 Reagent roster.** 29 supplies (`SUP-001`–`SUP-029`). Non-volatile components (`SUP-001`–`SUP-007`) are shared across product types; volatile reagents (`SUP-008`–`SUP-029`) map to specific SKUs.
- **dt-R010 Denormalized output.** Output is denormalized to one row per `(supply, sku)` pair, producing exactly 65 rows.
- **dt-R011 Origin region.** Every supply has an `origin_region` field set to one of the six guild hall settlement names.
- **dt-R012 Volatile flag.** Every supply has a `volatile` boolean. Serialized as the strings `"True"` / `"False"` in CSV output for parity with the original schema.

### Customers and guild ranks

- **dt-R013 Guild rank set.** `guild_rank` is one of `{initiate, journeyman, adept, master}`.
- **dt-R014 Name pool.** Customer names are drawn as **full fantasy names from a JSON pool** in `internal/catalog/names.go` — not assembled from separate first/last name lists. Each draw consumes a name from the pool.
- **dt-R015 Full pool utilization.** Names MUST be drawn randomly across the entire pool, not sequentially or weighted toward the start. (See `fix: utilize full name pool randomly` — `44815df`.)

### Persona roster

The behavior contract lives in `sm-simulation.md`; this domain pins the roster and weights.

- **dt-R016 Persona mix.** Six personas with the weights below. Weights sum to 1.0.

    | Persona | Weight | Weekday | Weekend | Sparrow |
    | --- | --- | --- | --- | --- |
    | Courier | 0.25 | high | 0.001 | 0.20 |
    | Artificer | 0.25 | low–med | 0.001 | 0.01 |
    | FeastReveler | 0.10 | 0 | med | 0.80 |
    | Apprentice | 0.20 | med | med | 0.80 |
    | Wanderer | 0.10 | 0.1 | 0.1 | 0.10 |
    | Herbalist | 0.10 | 0.2 | varies | 0.60 |

- **dt-R017 Seasonal modulation.** Apprentice is absent in summer; Herbalist is higher in summer. Other personas hold steady across seasons.

## Out of scope for this domain

- Persona *behavior* (order time distribution, item selection logic) → `sm-simulation.md`.
- File output format and schemas → `op-output.md`.
- Future TOML-driven theme mapping (Phase 3) — when it lands, this domain's roster will be re-cast as the `fantasy_rpg` bundled theme; new requirements will be added in a follow-up `dt-` block (or a new `th-themes.md` domain if the surface grows).
