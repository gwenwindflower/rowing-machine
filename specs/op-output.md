# Output — Spec

Owns on-disk layout, file formats, and column schemas. Lives in `internal/output/`.

The current implementation writes CSV only. JSONL, Parquet, and compression land in Phase 1.

## Vocabulary

- **Entity** — one of the seven tables: `stores`, `customers`, `orders`, `items`, `products`, `supplies`, `sparrows`.
- **Writer** — implementation of the `OutputWriter` interface that handles all seven entities for a given format.
- **Prefix** — `--pre` flag value (default `raw`); prepended to each entity file name.

## Requirements

### File layout

- **op-R001 File path.** Each entity writes to `{output-dir}/{prefix}_{entity}.{ext}` where `ext` is the format-appropriate extension (`csv`, `jsonl`, `parquet`). Defaults: `--output-dir ./factory-output`, `--pre raw`, format `csv`.
- **op-R002 Entity set.** Exactly seven files MUST be produced per run: `stores`, `customers`, `orders`, `items`, `products`, `supplies`, `sparrows`. No more, no fewer.
- **op-R003 Lazy file creation.** A writer MUST NOT create empty entity files for entities with zero rows in the run.

### Formatting conventions

- **op-R004 Timestamps.** All timestamp columns formatted as ISO 8601 `YYYY-MM-DDTHH:MM:SS` (no zone suffix). Applies to `stores.opened_at`, `orders.ordered_at`, and `sparrows.sent_at`.
- **op-R005 Currency.** All monetary columns serialized as integer cents (per `R002`). No decimal points.
- **op-R006 UUIDs.** UUIDs formatted as `xxxxxxxx-xxxx-4xxx-xxxx-xxxxxxxxxxxx` (v4 layout, lowercase hex).
- **op-R007 Boolean rendering.** Boolean columns render as the strings `"True"` / `"False"` for parity with the upstream Python schema (currently only `supplies.volatile`).

### Schemas

The canonical column reference lives in `docs/output-schema.md` to keep this spec readable. Requirement IDs here pin the *shape* — file `docs/output-schema.md` carries the table-by-table column lists.

- **op-R008 `stores` schema.** Columns: `id, name, opened_at, tax_rate`. 6 rows fixed. PK: `id`.
- **op-R009 `customers` schema.** Columns: `id, name, guild_rank`. Variable rows — only ordering customers. PK: `id`.
- **op-R010 `orders` schema.** Columns: `id, customer, ordered_at, store_id, subtotal, tax_paid, order_total`. PK: `id`. FKs: `customer → customers.id`, `store_id → stores.id`.
- **op-R011 `items` schema.** Columns: `id, order_id, sku`. PK: `id`. FKs: `order_id → orders.id`, `sku → products.sku`.
- **op-R012 `products` schema.** Columns: `sku, name, type, price, description, power_level`. 15 rows fixed. PK: `sku`.
- **op-R013 `supplies` schema.** Columns: `id, name, cost, volatile, origin_region, sku`. 92 rows fixed (denormalized — see `dt-R010`). Composite PK: `(id, sku)`. FK: `sku → products.sku`.
- **op-R014 `sparrows` schema.** Columns: `id, user_id, sent_at, content`. PK: `id`. FK: `user_id → customers.id`.

### Streaming and dedup

- **op-R015 Buffered writes.** Order, item, and sparrow rows write through buffered writers — no full collection in memory of unbounded streams.
- **op-R016 Static-table emission.** `products` and `supplies` are emitted once at end-of-run from the catalog. `stores` is emitted once after store generation.
- **op-R017 Rectangular rows.** Every row MUST contain exactly the number of fields declared by its entity schema. Writers MUST reject malformed rows instead of emitting ragged files.
- **op-R018 Clean relational keys.** Every primary-key field MUST be non-empty and unique within its entity. All other persisted fields MUST be non-empty. `supplies` uniqueness is evaluated on its composite `(id, sku)` key.

### Format and compression (planned — Phase 1)

- **op-R020 JSONL output format.** When `--format jsonl` is selected, each entity file is newline-delimited JSON with one row per line. Column names match the CSV column names exactly; types preserve native JSON types (numbers as numbers, booleans as `true`/`false`, not `"True"`/`"False"`).
- **op-R021 Parquet output format.** When `--format parquet` is selected, each entity file is Parquet with column types matching the natural domain types (integer cents stay int64, timestamps materialize as `TIMESTAMP_MICROS` UTC). Row group size auto-tuned to a sensible medium based on input size; the auto-tune knob MUST be exposed in code for later flag-based override.
- **op-R022 Compression.** `--compress` selects format-appropriate compression: gzip for `jsonl`, zstd for `parquet`. CSV does not support `--compress` in the first cut (revisit if needed). File extensions add the suffix: `.jsonl.gz`, `.parquet.zst`.
- **op-R023 Format determinism.** `R001` (byte-identical output per seed) MUST hold for each supported format independently. The same seed in CSV and JSONL won't byte-match (different formats) but two CSV runs of the same seed MUST byte-match, and likewise for JSONL and Parquet.

## Out of scope for this domain

- CLI flag parsing and validation → `cl-cli.md`.
- Per-entity data generation → `sm-simulation.md` and `dt-catalog.md`.
- Theme-driven entity remapping (Phase 3) — when themes land, the file naming pattern stays but column vocabulary becomes theme-controlled.
