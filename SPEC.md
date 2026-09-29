# Rowing Machine

## Goals

Rowing Machine is a deterministic synthetic data generator for SQL training, analytics engineering demos, and evaluating data tools. It simulates a business over time and writes relational files a learner or agent can query: realistic enough to teach joins, funnels, cohorts, and time series on, and reproducible enough that a lesson can pin a seed and trust the data never moves. It ships two scenarios: an ecommerce shop (the upstream data factory for Queria, a retro-RPG SQL trainer) and a B2B SaaS company with product usage, sales, and marketing. It is a single portable binary, fast enough to produce tens of millions of rows in seconds and to use every core.

Non-goals: it is not an ETL tool, a database client, or a real-world faker. Output is files; loading them is the consumer's job. Distributions are designed for teachable patterns, not statistical fidelity to any real industry. Themes change generated names and labels, never a scenario's structure or numbers.

## Vocabulary

- **Scenario** — the simulated business model: its entities, relationships, and behavior. `ecommerce` and `saas` ship.
- **Theme** — a naming pack: generators for people, organizations, locations, products, and the other names and labels scenarios need. A theme works with every scenario whose name kinds it covers; `fantasy_rpg` renders the shop as the Arcanum Collective mage guild.
- **Entity** — one output table of a scenario (`orders`, `subscriptions`, `events`).
- **Stream** — a named PRNG derived from the seed plus fixed indices, such as a market and a day.
- **Day state** — the pre-computed curves and calendar facts for one simulated day.
- **Market** — the ecommerce per-store simulation unit with its own customer pool.
- **Persona** — a behavioral archetype that drives an actor's timing and choices.
- **Funnel** — the SaaS path from anonymous visit to lead, opportunity or trial, and paying account.

Ecommerce terms (guild hall, sparrow, power level, guild rank) live in `specs/dt-catalog.md`; SaaS terms live in their domain specs.

## Domain specs

- @specs/sm-simulation.md
- @specs/dt-catalog.md
- @specs/op-output.md
- @specs/cl-cli.md
- @specs/th-themes.md
- @specs/sp-saas-product.md
- @specs/gm-go-to-market.md
- @specs/dev-engineering.md
- @specs/dev-release.md

## Requirements

- **R001** — Running the binary twice with the same seed and the same flags produces byte-identical output files, for every scenario, theme, format, and worker count.
- **R002** — Every persisted monetary value is integer cents; floats appear only inside a rate multiplication and are rounded back to cents immediately.
- **R003** — The release build is a single binary with no runtime dependencies, installable through Homebrew, `cargo binstall`, and release archives.
- **R004** — Every foreign key in every output file resolves to a primary key of the entity it references, for every scenario.
- **R005** — When `--seed 0` is passed, the binary prints the seed it chose to stdout unless `--quiet` is set, so the run can be reproduced.

## Backlog

- Messy mode: opt-in injection of formatting violations, value anomalies, schema drift, missing values, and encoding glitches, with the same determinism contract as clean output. The flag shape (boolean, graded levels, or independent toggles) is still open.
- More ecommerce entities: payments, promotions, staff, loyalty program.
- A TUI form that builds an invocation once the flag surface is dense enough to be painful as CLI arguments.
- An agent-oriented `--explain` mode that narrates the simulation as it runs.
- Bounded memory: primary-key sets grow with every row written, so a default SaaS run (about 20M rows) peaks near 2 GB. Proving key uniqueness by construction would let large runs stream in constant memory.
- Open question: is the default SaaS volume (about 20M rows, 6 s on one core) the right first run on a laptop, or should the SaaS scenario default to a smaller scale?
