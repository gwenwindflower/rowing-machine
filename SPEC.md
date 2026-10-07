# Rowing Machine

## Goals

Rowing Machine is a deterministic synthetic data generator for SQL training, analytics engineering demos, and evaluating data tools. It simulates a business over time and writes relational files a learner or agent can query: realistic enough to teach joins, funnels, cohorts, and time series on, and reproducible enough that a lesson can pin a seed and trust the data never moves. It ships three scenarios: an ecommerce shop (the upstream data factory for Queria, a retro-RPG SQL trainer), a B2B SaaS company with product usage, sales, and marketing, and a travel network where people book seats on scheduled trips. It is a single portable binary, fast enough to produce tens of millions of rows in seconds and to use every core.

Non-goals: it is not an ETL tool, a database client, or a real-world faker. Output is files; loading them is the consumer's job. Distributions are designed for teachable patterns, not statistical fidelity to any real industry. Themes change table and column names, generated names, labels, catalogs, and parameter values, never which entities a scenario has or how they relate.

## Vocabulary

- **Scenario** — the simulated business model: its entities, relationships, and behavior. `ecommerce`, `saas`, and `travel` ship.
- **Theme** — the skin over a scenario: table and column names, name generators, label lists, catalogs, and parameter values. A theme works with every scenario whose declarations it covers; `fantasy_rpg` renders the shop as the Arcanum Collective mage guild.
- **Parameter** — a number a scenario declares with a default and an inclusive range, such as network density or a price multiplier; themes set parameters and `--param` overrides them.
- **Catalog** — an ordered list of records with the typed fields a scenario declares and a theme supplies, such as airports with coordinates or add-ons with prices.
- **Entity** — one output table of a scenario, declared under a generic name (`orders`, `subscriptions`, `events`) that a theme can rename.
- **Stream** — a named PRNG derived from the seed plus fixed indices, such as a market and a day.
- **Day state** — the pre-computed curves and calendar facts for one simulated day.
- **Persona** — a behavioral archetype that drives an actor's timing and choices.

Domain specs define their own terms; ecommerce catalog terms live in `docs/scenarios/ecommerce.md`.

## Domain specs

- @specs/sm-simulation.md
- @specs/op-output.md
- @specs/cl-cli.md
- @specs/th-themes.md
- @specs/sp-saas-product.md
- @specs/gm-go-to-market.md
- @specs/tr-travel.md

## Requirements

- **R001** — Running the binary twice with the same seed and the same flags produces byte-identical output files, for every scenario, theme, format, and worker count.
- **R002** — Every persisted monetary value is integer cents; floats appear only inside a rate multiplication and are rounded back to cents immediately.
- **R003** — The release build is a single binary with no runtime dependencies, installable through `cargo binstall`, `cargo install`, and release archives.
- **R004** — Every foreign key in every output file resolves to a primary key of the entity it references, for every scenario.
- **R005** — When `--seed 0` is passed, the binary prints the seed it chose to stdout unless `--quiet` is set.

## Open questions

- Is the default SaaS volume (about 37M rows, 32M of them events) the right first run on a laptop, or should the SaaS scenario default to a smaller scale?
