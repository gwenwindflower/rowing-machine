# Output Schema

Rowing Machine writes seven CSV files to `{output-dir}/{prefix}_{entity}.csv`.
Default path: `./factory-output/raw_{entity}.csv`.

**Conventions across all tables:**

- Timestamps are ISO 8601 format: `YYYY-MM-DDThh:mm:ss`
- Monetary values are **integer cents** (e.g. `1100` = 11 gold)
- UUIDs are deterministic v4, formatted as `xxxxxxxx-xxxx-4xxx-xxxx-xxxxxxxxxxxx`

---

## stores

Six rows (fixed). One per settlement.

| Column | Type | Example | Description |
| --- | --- | --- | --- |
| id | uuid | `a1b2c3d4-...` | Primary key |
| name | string | `Misthollow` | Settlement name of the store |
| opened_at | timestamp | `2019-03-12T00:00:00` | Date the store opened (epoch + opened_day offset) |
| tax_rate | float | `0.0625` | Local tax rate as a decimal |

---

## customers

Variable row count. Only customers who placed at least one order appear.

| Column | Type | Example | Description |
| --- | --- | --- | --- |
| id | uuid | `e5f6a7b8-...` | Primary key |
| name | string | `Jordan Rivera` | Full name (`{FirstName} {LastName}`) |
| guild_rank | string | `journeyman` | Rank within guild: `initiate`, `journeyman`, `adept`, or `master` |

---

## orders

One row per order placed during the simulation.

| Column | Type | Example | Description |
| --- | --- | --- | --- |
| id | uuid | `c3d4e5f6-...` | Primary key |
| customer | uuid | `e5f6a7b8-...` | FK &rarr; `customers.id` |
| ordered_at | timestamp | `2019-06-15T08:42:00` | Date and time of the order (date + sampled minute) |
| store_id | uuid | `a1b2c3d4-...` | FK &rarr; `stores.id` |
| subtotal | int | `2300` | Sum of item prices in cents |
| tax_paid | int | `144` | `round(subtotal * tax_rate)` in cents |
| order_total | int | `2444` | `subtotal + tax_paid` in cents |

---

## items

Normalized join table. One row per line item per order.

| Column | Type | Example | Description |
| --- | --- | --- | --- |
| id | uuid | `d4e5f6a7-...` | Primary key (unique per item instance) |
| order_id | uuid | `c3d4e5f6-...` | FK &rarr; `orders.id` |
| sku | string | `WEP-003` | FK &rarr; `products.sku` |

---

## products

Fifteen rows (fixed). The full Arcanum Collective catalog.

| Column | Type | Example | Description |
| --- | --- | --- | --- |
| sku | string | `WEP-001` | Primary key. Format: `WEP-NNN`, `ARM-NNN`, or `ELX-NNN` |
| name | string | `Shadowfang Dagger` | Product display name |
| type | string | `weapon` | `"weapon"`, `"armor"`, or `"elixir"` |
| price | int | `1100` | Price in cents |
| description | string | `shadow-infused dagger` | Short product description |
| power_level | string | `rare` | `"common"`, `"uncommon"`, `"rare"`, `"epic"`, or `"legendary"` |

---

## supplies

Denormalized: one row per (supply, product SKU) pair. 65 rows (fixed).

| Column | Type | Example | Description |
| --- | --- | --- | --- |
| id | string | `SUP-001` | Supply identifier (not unique per row — repeats across SKUs) |
| name | string | `shadow essence` | Reagent name |
| cost | int | `400` | Cost in cents |
| volatile | string | `True` | `"True"` or `"False"` |
| origin_region | string | `Duskmarsh` | Region the reagent is sourced from |
| sku | string | `WEP-001` | Associated product SKU. FK &rarr; `products.sku` |

---

## sparrows

One row per customer sparrow. Generated probabilistically after orders.

| Column | Type | Example | Description |
| --- | --- | --- | --- |
| id | uuid | `f6a7b8c9-...` | Primary key |
| user_id | uuid | `e5f6a7b8-...` | FK &rarr; `customers.id` |
| sent_at | timestamp | `2019-06-15T08:55:00` | Order time + 0-19 minute random delay |
| content | string | `Wares from the Arcanum Collective are magnificent! Acquired a Frostbite Halberd.` | Generated from fan_level sentiment templates |

---

## Relationships

```text
stores.id        <--  orders.store_id
customers.id     <--  orders.customer
customers.id     <--  sparrows.user_id
orders.id        <--  items.order_id
products.sku     <--  items.sku
products.sku     <--  supplies.sku
```
