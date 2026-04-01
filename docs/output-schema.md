# Output Schema

Seven CSV files written to `{output-dir}/{prefix}_{entity}.csv`. Default: `./factory-output/raw_{entity}.csv`.

All timestamps are ISO 8601 (`2006-01-02T15:04:05`). All monetary values are integer cents. UUIDs are v4, generated deterministically from PRNG.

## stores.csv

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Store UUID |
| name | string | Store name (city) |
| opened_at | timestamp | Epoch + opened_day |
| tax_rate | float | e.g. 0.06 |

6 rows (fixed).

## customers.csv

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Customer UUID |
| name | string | "{FirstName} {LastName}" |

Only customers who placed at least one order.

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
| sku | string | e.g. JAF-001, BEV-003 |
| name | string | Product name |
| type | string | "jaffle" or "beverage" |
| price | int | Price in cents |
| description | string | Product description |

10 rows (fixed).

## supplies.csv

| Column | Type | Notes |
| --- | --- | --- |
| id | string | e.g. SUP-001 |
| name | string | Supply name |
| cost | int | Cost in cents |
| perishable | string | "True" or "False" |
| sku | string | Associated product SKU |

Denormalized: one row per (supply, SKU) pair. 65 rows (fixed).

## tweets.csv

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Tweet UUID |
| user_id | uuid | FK to customers.id |
| tweeted_at | timestamp | Order time + 0-19 min delay |
| content | string | Generated from fan_level templates |
