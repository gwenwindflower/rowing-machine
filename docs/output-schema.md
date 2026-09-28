# Output schema

The ecommerce scenario declares seven entities in [`src/scenario/ecommerce/mod.rs`](../src/scenario/ecommerce/mod.rs). The [CSV sink](../src/output/mod.rs) writes populated entities to `{output-dir}/{prefix}_{entity}.csv` in schema column order. Default: `./factory-output/raw_{entity}.csv`. An entity with no rows creates no file.

All timestamps are ISO 8601 (`YYYY-MM-DDTHH:MM:SS`). All monetary values are integer cents. UUIDs are v4, generated deterministically from named PCG streams.

## stores.csv

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Store UUID |
| name | string | Store name (settlement) |
| opened_at | timestamp | Epoch + opened_day |
| tax_rate | float | e.g. 0.06 |

6 rows (fixed).

## customers.csv

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Customer UUID |
| name | string | "{Guild hall} patron {customer index}" |
| guild_rank | string | Order-frequency quartile: "initiate", "journeyman", "adept", or "master" |

Only customers who placed at least one order. Guild rank cohorts differ in size by at most one customer and progress from the lowest to highest lifetime order counts.

## orders.csv

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Order UUID |
| customer | uuid | FK to customers.id |
| ordered_at | timestamp | Date + sampled minute |
| store_id | uuid | FK to stores.id |
| subtotal | int | Sum of item prices (cents) |
| tax_paid | int | round(subtotal * tax_rate) |
| order_total | int | subtotal + tax_paid |

## items.csv

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Item UUID (generated per item in order) |
| order_id | uuid | FK to orders.id |
| sku | string | FK to products.sku |

Normalized join table. One row per item per order.

## products.csv

| Column | Type | Notes |
| --- | --- | --- |
| sku | string | e.g. WEP-001, ARM-003, ELX-002 |
| name | string | Product name |
| type | string | "weapon", "armor", or "elixir" |
| price | int | Price in cents |
| description | string | Product description |
| power_level | string | "common", "uncommon", "rare", "epic", or "legendary" |

15 rows (fixed).

## supplies.csv

| Column | Type | Notes |
| --- | --- | --- |
| id | string | Supply identifier, e.g. SUP-001 |
| name | string | Supply/reagent name |
| cost | int | Cost in cents |
| volatile | string | "True" or "False" |
| origin_region | string | Region of origin (store settlement name) |
| sku | string | Associated product SKU |

Denormalized: one row per `(id, sku)` pair. The composite pair is the primary key. 92 rows (fixed).

## sparrows.csv

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Sparrow UUID |
| user_id | uuid | FK to customers.id |
| sent_at | timestamp | Order time + 0-19 min delay |
| content | string | Fan-level template and the sender's final guild-rank vocabulary |
