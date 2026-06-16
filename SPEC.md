# Rowing Machine — Project Spec

A flexible, deterministic synthetic data generator for **Queria**, a retro-RPG-inspired SQL trainer. Simulates the Arcanum Collective — a mage guild running guild halls across fantasy towns — producing realistic relational data across customers, orders, items, products, supplies, stores, and sparrows.

The tool's job is to be the data factory upstream of Queria's learning content: rich enough that a learner can practice meaningful joins, aggregations, and time-series queries; reproducible enough that lessons can pin to specific datasets.

## Goals

- **Deterministic by default.** Same seed in, byte-identical output across all formats. Lessons and tests can pin to a seed and trust the result.
- **Realistic enough to teach on.** Behavioral personas, seasonality, growth, market penetration, and store ramp-up dynamics — not random rows.
- **Flexible output shape.** Tune duration, scale, format, compression, and (eventually) theme and mess level from CLI flags.
- **Fast and parallelizable.** A Go binary that can churn out millions of rows in seconds and scale across cores.
- **Single portable binary.** Distributable via Homebrew and Linux package managers without runtime dependencies. Fits the Supermodel Labs CLI distribution model.

## Non-goals

- Not a general-purpose ETL tool. The simulation model is the product.
- Not a database client. Output is files; loading is the consumer's problem.
- Not a real-world data faker. We don't try to mimic any specific industry's real distributions — the personas and curves are designed for *teachable* patterns, not statistical fidelity.

## Vocabulary

- **Guild hall** — a store location (Thornwall, Misthollow, etc.).
- **Patron / customer** — a guild member who places orders.
- **Sparrow** — a customer-sent message about an order (the project's replacement for tweets/reviews).
- **Reagent / supply** — an ingredient or material associated with a product SKU.
- **Persona** — a behavioral archetype that drives a customer's order timing and content.
- **Power level** — product rarity tier (common → legendary).
- **Guild rank** — customer progression tier (initiate → master).
- **Market** — per-store simulation unit; each gets its own PRNG and customer pool.
- **Day effect** — pre-computed product of annual × weekend × growth curves for a given simulation day.
- **Theme** — *(planned)* a TOML-driven mapping from the baseline ecommerce schema to a flavored vocabulary (e.g. `fantasy_rpg`).
- **Messy mode** — *(planned)* deliberate injection of formatting and value anomalies for training learners against realistic dirty data.

## Design principles

- **Money is `int64` cents end-to-end.** Floats only appear in tax-rate multiplication, immediately rounded back to cents. Never store currency as `float64`.
- **All randomness flows from one `--seed`.** Subsystems derive their PRNGs deterministically from the seed and a known offset (per-market index, store RNG seeded with the bare seed, etc.).
- **Pre-compute per-day state.** Day effects (annual × weekend × growth) and season tags are computed up-front into a `[]DayState` slice; the hot loop allocates nothing.
- **Stream output where the size is unbounded.** Orders, items, and sparrows write through buffered writers. Customers stay in a dedup map. Products and supplies are static — written once from the catalog.
- **Specs hold *what*. Code holds *how*.** Hardcoded numbers in `internal/catalog/` and `internal/models/store.go` are the authoritative implementation; the matching spec rows describe the contract those values are required to satisfy.

## Project-scope requirements

- **R001 Deterministic seed contract.** Running the binary twice with the same `--seed`, `--years`, `--scale`, `--start-date`, and output format MUST produce byte-identical output files. Verified by `internal/simulation/integration_test.go`.
- **R002 Money as cents.** All persisted monetary values (price, cost, subtotal, tax_paid, order_total) MUST be `int64` cents. Tax is `int64(math.Round(float64(subtotal) * taxRate))`. No `float64` currency fields in any output row.
- **R003 Single portable binary.** `go build ./cmd/rowing-machine` MUST produce a single statically-linked binary with no runtime dependencies, suitable for Homebrew and Linux package distribution.
- **R004 Referential integrity.** Every `orders.customer` resolves to a `customers.id`; every `orders.store_id` to a `stores.id`; every `items.order_id` to an `orders.id`; every `items.sku` and `supplies.sku` to a `products.sku`; every `sparrows.user_id` to a `customers.id`. Verified end-to-end in integration tests.
- **R005 Self-printing seed.** When `--seed 0` is passed, the chosen random seed MUST be printed to stdout (unless `--quiet`) so the run can be reproduced.

## Domain specs

- @specs/sm-simulation.md — simulation engine: formulas, curves, RNG, day loop, personas, sparrow generation
- @specs/dt-catalog.md — static data: guild halls, products, supplies, name pool, persona roster
- @specs/op-output.md — output writers, file layout, schemas, planned formats and compression
- @specs/cl-cli.md — CLI surface: current flags and planned flag additions
