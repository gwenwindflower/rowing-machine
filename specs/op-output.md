# Output

## Goals

Output owns where files land, how rows serialize in each format, and the column shape of every ecommerce entity. Writers are format-specific and scenario-agnostic: a scenario hands them entity schemas and rows, and they never know which business produced them. SaaS entity schemas live in `sp-saas-product.md` and `gm-go-to-market.md`; `docs/output-schema.md` holds every column list.

## Vocabulary

- **Entity schema** — an entity's name, ordered columns, column types, nullability, and primary key, all declared by the scenario.
- **Prefix** — the `--pre` value prepended to each entity file name.

## Requirements

### File layout

- **op-R001** — Each entity writes to `{output-dir}/{prefix}_{entity}.{ext}`, where `ext` is `csv`, `jsonl`, or `parquet`, plus `.gz` for compressed JSONL.
- **op-R002** — A run writes exactly one file per entity its scenario declares: seven for `ecommerce`, and the entity set in the SaaS specs for `saas`.
- **op-R003** — A writer never creates an empty file for an entity with zero rows in the run.

### Formatting

- **op-R004** — Timestamps render as ISO 8601 `YYYY-MM-DDTHH:MM:SS` with no zone suffix in CSV and JSONL.
- **op-R005** — Monetary columns serialize as integer cents with no decimal point.
- **op-R006** — UUIDs render as lowercase v4 layout, `xxxxxxxx-xxxx-4xxx-xxxx-xxxxxxxxxxxx`.
- **op-R007** — Boolean columns render as `True` / `False` in CSV.

### Ecommerce schemas

- **op-R008** — `stores`: `id, name, opened_at, tax_rate`; 6 rows; PK `id`.
- **op-R009** — `customers`: `id, name, guild_rank`; one row per ordering customer; PK `id`.
- **op-R010** — `orders`: `id, customer, ordered_at, store_id, subtotal, tax_paid, order_total`; PK `id`; FKs `customer → customers.id`, `store_id → stores.id`.
- **op-R011** — `items`: `id, order_id, sku`; PK `id`; FKs `order_id → orders.id`, `sku → products.sku`.
- **op-R012** — `products`: `sku, name, type, price, description, power_level`; 15 rows; PK `sku`.
- **op-R013** — `supplies`: `id, name, cost, volatile, origin_region, sku`; 92 rows; composite PK `(id, sku)`; FK `sku → products.sku`.
- **op-R014** — `sparrows`: `id, user_id, sent_at, content`; PK `id`; FK `user_id → customers.id`.

### Streaming and integrity

- **op-R015** — Entities without a fixed row count write through buffered writers and are never collected whole in memory.
- **op-R016** — Static entities (catalogs, rosters, plans) are written once per run.
- **op-R017** — Every row has exactly its entity schema's field count; a writer rejects a malformed row with an error instead of writing a ragged file.
- **op-R018** — Every primary key is non-empty and unique within its entity, and every other field is non-empty unless its schema marks it nullable.
- **op-R024** — Columns appear in the order their entity schema declares, in every format.

### Formats and compression

- **op-R020** — With `--format jsonl`, each file holds one JSON object per line, keyed by the schema's column names, with numbers as numbers and booleans as `true` / `false`.
- **op-R021** — With `--format parquet`, integer cents stay int64, timestamps are `TIMESTAMP_MICROS` UTC, and row group size is derived from the estimated row count through a single code-level constant.
- **op-R022** — `--compress` gzips JSONL files (`.jsonl.gz`) and switches Parquet column compression from uncompressed to zstd, keeping the `.parquet` extension.
- **op-R023** — Two runs with the same seed and flags byte-match within each format.
