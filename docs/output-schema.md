# Output schema

The ecommerce scenario declares seven entities in [`src/scenario/ecommerce/mod.rs`](../src/scenario/ecommerce/mod.rs). The SaaS scenario declares six in [`src/scenario/saas/schema.rs`](../src/scenario/saas/schema.rs). The [output sink](../src/output/mod.rs) writes populated entities to `{output-dir}/{prefix}_{entity}.{ext}` in schema column order. Choose `csv` (default), `jsonl`, or `parquet` with `--format`. Default: `./factory-output/raw_{entity}.csv`. An entity with no rows creates no file.

CSV and JSONL timestamps are ISO 8601 (`YYYY-MM-DDTHH:MM:SS`) without a zone suffix. Parquet timestamps use UTC `TIMESTAMP_MICROS`. All monetary values are integer cents, stored as int64 in Parquet. UUIDs are lowercase v4 strings, generated deterministically from named PCG streams.

JSONL emits one object per line with native numbers, booleans, and nulls. Parquet uses typed nullable columns. CSV booleans are `True` or `False`. `--compress` writes gzip JSONL with the `.jsonl.gz` extension or zstd Parquet with the `.parquet` extension. Every format, including compressed output, is byte-identical for repeated runs with the same seed and flags.

## stores

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Store UUID |
| name | string | Store name (settlement) |
| opened_at | timestamp | Epoch + opened_day |
| tax_rate | float | e.g. 0.06 |

6 rows (fixed).

## customers

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Customer UUID |
| name | string | "{Guild hall} patron {customer index}" |
| guild_rank | string | Order-frequency quartile: "initiate", "journeyman", "adept", or "master" |

Only customers who placed at least one order. Guild rank cohorts differ in size by at most one customer and progress from the lowest to highest lifetime order counts.

## orders

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Order UUID |
| customer | uuid | FK to customers.id |
| ordered_at | timestamp | Date + sampled minute |
| store_id | uuid | FK to stores.id |
| subtotal | int | Sum of item prices (cents) |
| tax_paid | int | round(subtotal * tax_rate) |
| order_total | int | subtotal + tax_paid |

## items

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Item UUID (generated per item in order) |
| order_id | uuid | FK to orders.id |
| sku | string | FK to products.sku |

Normalized join table. One row per item per order.

## products

| Column | Type | Notes |
| --- | --- | --- |
| sku | string | e.g. WEP-001, ARM-003, ELX-002 |
| name | string | Product name |
| type | string | "weapon", "armor", or "elixir" |
| price | int | Price in cents |
| description | string | Product description |
| power_level | string | "common", "uncommon", "rare", "epic", or "legendary" |

15 rows (fixed).

## supplies

| Column | Type | Notes |
| --- | --- | --- |
| id | string | Supply identifier, e.g. SUP-001 |
| name | string | Supply/reagent name |
| cost | int | Cost in cents |
| volatile | boolean | `True` or `False` in CSV; native boolean in JSONL and Parquet |
| origin_region | string | Region of origin (store settlement name) |
| sku | string | Associated product SKU |

Denormalized: one row per `(id, sku)` pair. The composite pair is the primary key. 92 rows (fixed).

## sparrows

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Sparrow UUID |
| user_id | uuid | FK to customers.id |
| sent_at | timestamp | Order time + 0-19 min delay |
| content | string | Fan-level template and the sender's final guild-rank vocabulary |

## accounts

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Primary key |
| name | string | Generated organization name |
| industry | string | Theme industry label |
| employee_band | string | `small`, `medium`, or `large` |
| region | string | Theme region label |
| created_at | timestamp | Arrival day at midnight UTC |
| acquisition_channel | string | `direct` |
| first_touch_id | nullable uuid | Reserved FK to `touches.id`; null for direct arrivals |

## users

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Primary key |
| account_id | uuid | FK to accounts.id |
| name | string | Generated person name |
| email | string | Person and organization slugs under the reserved `.example` domain |
| role | string | Theme role label |
| created_at | timestamp | On or after account arrival |
| activated_at | nullable timestamp | Within seven days of user creation, when activated |

User rows record creation and activation. Departures affect seat counts but have no exported timestamp.

## plans

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Primary key; three static rows |
| name | string | Generated plan name |
| tier | string | Theme tier label, ordered lowest to highest |
| seat_price_monthly | int | Monthly price per billable seat in cents |
| seat_price_annual | int | Annual price per billable seat in cents |
| included_seats | int | Reference bundle size; billable seats are not reduced by it |

## subscriptions

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Primary key for an immutable paid interval |
| account_id | uuid | FK to accounts.id |
| plan_id | uuid | FK to plans.id |
| billing_interval | string | `monthly` or `annual` |
| seats | int | Billable active users during this interval |
| mrr | int | Seats × interval seat price, normalized to monthly cents |
| started_at | timestamp | Inclusive interval start |
| ended_at | nullable timestamp | Exclusive interval end; null when still open |
| status | string | `canceled` for ended intervals; otherwise `past_due` or `active` at the run boundary |

Trials create users but no paid subscription rows. A seat change ends one interval and starts another at the same timestamp.

## mrr_movements

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Primary key |
| account_id | uuid | FK to accounts.id |
| subscription_id | uuid | FK to subscriptions.id; ended interval for churn, starting interval otherwise |
| movement_type | string | `new`, `expansion`, `contraction`, `churn`, or `reactivation` |
| occurred_at | timestamp | Subscription transition timestamp |
| mrr_delta | int | Signed change in monthly cents |
| mrr_after | int | Account MRR after this transition |

## invoices

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Primary key |
| account_id | uuid | FK to accounts.id |
| subscription_id | uuid | FK to subscriptions.id |
| issued_at | timestamp | Start of the billed period |
| period_start | timestamp | Inclusive coverage start |
| period_end | timestamp | Exclusive coverage end |
| amount | int | Cents, prorated by covered days for partial periods |
| paid_at | nullable timestamp | Payment date observed during the run; null for unpaid or not-yet-paid invoices |

Periods tile each subscription through its end or the exclusive simulation boundary. See [the SaaS model](saas-model.md) for lifecycle rules and SQL examples.
