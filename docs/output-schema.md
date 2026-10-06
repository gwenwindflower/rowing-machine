# Output schema

The ecommerce scenario declares seven entities in [`src/scenario/ecommerce/mod.rs`](../src/scenario/ecommerce/mod.rs). The SaaS scenario declares sixteen through [`src/scenario/saas/schema.rs`](../src/scenario/saas/schema.rs). The [output sink](../src/output/mod.rs) writes populated entities to `{output-dir}/{prefix}_{entity}.{ext}` in schema column order. Choose `csv` (default), `jsonl`, or `parquet` with `--format`. Default: `./factory-output/raw_{entity}.csv`. An entity with no rows creates no file.

CSV and JSONL timestamps are ISO 8601 (`YYYY-MM-DDTHH:MM:SS`) without a zone suffix. Parquet timestamps use UTC `TIMESTAMP_MICROS`. Dates are `YYYY-MM-DD` strings in CSV and JSONL and native dates in Parquet. All monetary values are integer cents, stored as int64 in Parquet. UUIDs are lowercase v4 strings, generated deterministically from named PCG streams.

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
| name | string | Generated person name from the theme |
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

## campaigns

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Primary key |
| channel | string | `paid_search`, `paid_social`, or `display` |
| name | string | Generated campaign name |
| started_at | timestamp | Inclusive start of the 90-day flight |
| ended_at | nullable timestamp | Exclusive end of the flight |
| daily_budget | int | Daily budget in cents |

## ad_spend

| Column | Type | Notes |
| --- | --- | --- |
| date | date | Day of spend |
| campaign_id | uuid | FK to campaigns.id |
| impressions | int | At least the number of clicks |
| clicks | int | Number of paid touches for this campaign and day |
| spend | int | Daily spend in cents |

One row per active campaign per day. The composite primary key is `(date, campaign_id)`.

## touches

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Primary key |
| visitor_id | uuid | Anonymous visitor identifier; shared across that visitor's touches |
| campaign_id | nullable uuid | FK to campaigns.id; null for unpaid touches |
| channel | string | `paid_search`, `paid_social`, `display`, `content`, `referral`, or `direct` |
| occurred_at | timestamp | Interaction time |
| landing_page | string | Relative page path |

## leads

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Primary key |
| visitor_id | uuid | Visitor identifier shared with touches |
| name | string | Generated person name |
| email | string | Address under the reserved `.example` domain |
| created_at | timestamp | Identification time, after the visitor's touches |
| lead_source | string | First-touch channel |
| first_touch_id | uuid | FK to touches.id |
| last_touch_id | uuid | FK to touches.id; may equal first_touch_id |
| status | string | `qualified`, `demo_requested`, or `converted` |
| account_id | nullable uuid | FK to accounts.id; populated for self-serve conversions and admitted sales opportunities |

Self-serve converted leads start trials; sales leads become converted when their opportunity is won within the run. Admitted demo requests retain an account link while open or lost. Demo requests without rep capacity have no account or opportunity.

## accounts

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Primary key |
| name | string | Generated organization name |
| industry | string | Theme industry label |
| employee_band | string | `small`, `medium`, or `large` |
| region | string | Theme region label |
| created_at | timestamp | Trial or sales prospect arrival at midnight UTC, the day after lead creation |
| acquisition_channel | string | Lead's first-touch channel |
| first_touch_id | nullable uuid | FK to touches.id; matches the converting lead's first_touch_id |

## sales_reps

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Primary key |
| name | string | Generated person name |
| segment | string | `small`, `medium`, or `large`; matches account employee band |
| hired_at | timestamp | Inclusive employment start |
| departed_at | nullable timestamp | Exclusive employment end; null for retained reps |
| annual_cost | int | Annual compensation cost in cents |

## opportunities

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Primary key |
| account_id | uuid | FK to accounts.id |
| lead_id | uuid | FK to leads.id |
| owner_id | uuid | FK to sales_reps.id |
| created_at | timestamp | Sales prospect arrival |
| stage | string | Latest observed stage: `discovery`, `demo`, `proposal`, `negotiation`, `closed_won`, or `closed_lost` |
| amount | int | Quoted annual recurring revenue in cents; twelve times initial subscription MRR for a win |
| closed_at | nullable timestamp | Observed close; null while open at the run boundary |
| outcome | nullable string | `won` or `lost`; null while open |

## opportunity_stages

| Column | Type | Notes |
| --- | --- | --- |
| opportunity_id | uuid | FK to opportunities.id |
| stage | string | Entered stage, using the opportunity stage vocabulary |
| entered_at | timestamp | Inclusive stage entry time |

The composite primary key is `(opportunity_id, stage)`. Only entries before the exclusive run boundary are emitted.

## sales_activities

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Primary key |
| rep_id | uuid | FK to sales_reps.id |
| opportunity_id | uuid | FK to opportunities.id |
| activity_type | string | `email`, `call`, or `meeting` |
| occurred_at | timestamp | Activity time during the open opportunity and rep employment |

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

## sessions

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Primary key |
| user_id | uuid | FK to users.id |
| account_id | uuid | FK to accounts.id; matches the user's account |
| started_at | timestamp | Inclusive session start in UTC |
| ended_at | timestamp | Exclusive session end in UTC |
| device | string | `desktop` or `mobile` |

Regular sessions follow regional business hours. Activation markers use a dedicated one-minute desktop session at `users.activated_at`. See [product usage](saas-model.md#product-usage) for scheduling rules.

## events

| Column | Type | Notes |
| --- | --- | --- |
| id | uuid | Primary key |
| session_id | uuid | FK to sessions.id |
| user_id | uuid | FK to users.id; matches the session's user |
| account_id | uuid | FK to accounts.id; matches the session's account |
| occurred_at | timestamp | Within the session's inclusive start and exclusive end |
| event_name | string | `<feature>:used` or `<feature>:activated` |
| feature | string | One of 16 theme-generated feature names |

Every non-null `users.activated_at` has exactly one matching activation event. Regular sessions contain 4–12 events; activation sessions contain one.

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
