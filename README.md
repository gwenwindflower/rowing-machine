# Rowing Machine

`rowing-machine` generates deterministic synthetic ecommerce and SaaS data for SQL training, analytics engineering demos, and evaluating data tools. Pick a seed and duration, and it writes relational CSV, JSONL, or Parquet files you can load anywhere.

The shop has stores, customers, orders, items, products, supplies, and customer messages, driven by personas, seasonality, growth, and store ramp-up. The default `plain` theme uses retail vocabulary; `fantasy_rpg` supplies the Arcanum Collective mage-guild vocabulary for Queria, a retro-RPG SQL trainer.

Runs are byte-deterministic from one seed, so a lesson can pin a dataset forever or sweep seeds for fresh data of the same shape.

## Installation

With [Cargo Binstall](https://github.com/cargo-bins/cargo-binstall#installation):

```bash
cargo binstall rowing-machine
```

Build from crates.io with Cargo:

```bash
cargo install rowing-machine --locked
```

Or download a [release archive](https://github.com/gwenwindflower/rowing-machine/releases) for Linux or macOS on Intel or ARM. Verify its SHA-256 checksum, extract it, and place `rowing-machine` on your `PATH`.

## Quick start

```bash
rowing-machine --seed 42 --years 1
rowing-machine themes
rowing-machine --seed 42 --theme fantasy_rpg
rowing-machine --scenario saas --seed 42 --years 4
rowing-machine --help
```

Defaults simulate four 365-day years starting on 2023-01-01 at scale 100 and write `raw_*.csv` under `factory-output/`. A random seed is printed when `--seed` is omitted; pass that seed to reproduce the files. Use `--quiet` to suppress console output.

Generation uses available cores by default. Set `--workers 1` for serial generation or choose another positive worker count; the output files remain byte-identical.

Select `--format jsonl` or `--format parquet` for typed output. Add `--compress` for `.jsonl.gz` files or zstd-compressed Parquet; CSV does not support compression.

```bash
rowing-machine --seed 42 --target-rows 100000 --format parquet --compress
```

`--target-rows` samples the simulation to choose a duration producing about that many orders (ecommerce) or accounts (SaaS), within 5% or the nearest whole day. Other entities retain their relationships and natural row counts. Calibration runs before generation and cannot be combined with `--years`.

`--scenario saas` uses `plain` business vocabulary and writes marketing, account, revenue, and product usage entities. `--scale` controls visitor volume; account conversion and active user-days determine session and event volume. See [the SaaS model](docs/saas-model.md) for lifecycle rules and attribution, revenue, retention, and engagement SQL.

Use `--theme ./shop.toml` for a custom naming pack. [Theme authoring](docs/themes.md) explains the schema and name assignment rules.

## Documentation

- `SPEC.md` and `specs/` — what the tool does, with stable requirement IDs
- `docs/` — how it works: architecture, formulas, catalog, output schema

## About the project

Rowing Machine descends from the Jaffle Shop Generator ([jafgen](https://pypi.org/project/jafgen/), [dbt-labs/jaffle-shop-generator](https://github.com/dbt-labs/jaffle-shop-generator)), which produced semi-realistic trends and patterns without simulating a full world of agents.
