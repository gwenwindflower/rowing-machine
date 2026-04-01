# Output Schema

Rowing Machine writes seven CSV files to `{output-dir}/{prefix}_{entity}.csv`.
Default path: `./factory-output/raw_{entity}.csv`.

**Conventions across all tables:**

- Timestamps are ISO 8601 format: `YYYY-MM-DDThh:mm:ss`
- Monetary values are **integer cents** (e.g. `1100` = $11.00)
- UUIDs are deterministic v4, formatted as `xxxxxxxx-xxxx-4xxx-xxxx-xxxxxxxxxxxx`

---

## stores

Six rows (fixed). One per physical location.

| Column | Type | Example | Description |
| --- | --- | --- | --- |
| id | uuid | `a1b2c3d4-...` | Primary key |
| name | string | `Brooklyn` | City name of the store |
| opened_at | timestamp | `2019-03-12T00:00:00` | Date the store opened (epoch + opened_day offset) |
| tax_rate | float | `0.0625` | Local tax rate as a decimal |

---

## customers

Variable row count. Only customers who placed at least one order appear.

| Column | Type | Example | Description |
| --- | --- | --- | --- |
| id | uuid | `e5f6a7b8-...` | Primary key |
| name | string | `Jordan Rivera` | Full name (`{FirstName} {LastName}`) |

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
| sku | string | `JAF-003` | FK &rarr; `products.sku` |

---

## products

Ten rows (fixed). The full Jaffle Shop menu.

| Column | Type | Example | Description |
| --- | --- | --- | --- |
| sku | string | `JAF-001` | Primary key. Format: `JAF-NNN` or `BEV-NNN` |
| name | string | `nutellaphone who dis?` | Product display name |
| type | string | `jaffle` | `"jaffle"` or `"beverage"` |
| price | int | `1100` | Price in cents |
| description | string | `nutella and banana jaffle` | Short product description |

---

## supplies

Denormalized: one row per (supply, product SKU) pair. 65 rows (fixed).

| Column | Type | Example | Description |
| --- | --- | --- | --- |
| id | string | `SUP-001` | Supply identifier (not unique per row -- repeats across SKUs) |
| name | string | `Nutella` | Supply name |
| cost | int | `400` | Cost in cents |
| perishable | string | `True` | `"True"` or `"False"` |
| sku | string | `JAF-001` | Associated product SKU. FK &rarr; `products.sku` |

---

## tweets

One row per customer tweet. Generated probabilistically after orders.

| Column | Type | Example | Description |
| --- | --- | --- | --- |
| id | uuid | `f6a7b8c9-...` | Primary key |
| user_id | uuid | `e5f6a7b8-...` | FK &rarr; `customers.id` |
| tweeted_at | timestamp | `2019-06-15T08:55:00` | Order time + 0-19 minute random delay |
| content | string | `Jaffles from the Jaffle Shop are amazing! Ordered a mel-bun.` | Generated from fan_level sentiment templates |

---

## Relationships

```text
stores.id        <--  orders.store_id
customers.id     <--  orders.customer
customers.id     <--  tweets.user_id
orders.id        <--  items.order_id
products.sku     <--  items.sku
products.sku     <--  supplies.sku
```
