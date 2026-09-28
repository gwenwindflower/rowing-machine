# Rowing Machine

`rowing-machine` generates deterministic synthetic ecommerce data for SQL training, analytics engineering demos, and evaluating data tools. Pick a seed and duration, and it writes relational CSV files you can load anywhere.

The shop has stores, customers, orders, items, products, supplies, and customer messages, driven by personas, seasonality, growth, and store ramp-up. Its Arcanum Collective mage-guild vocabulary serves Queria, a retro-RPG SQL trainer.

Runs are byte-deterministic from one seed, so a lesson can pin a dataset forever or sweep seeds for fresh data of the same shape.

> [!NOTE]
> The Rust CLI supports serial ecommerce CSV generation. Additional formats, parallel generation, selectable themes, and a SaaS scenario are planned in [TODO.md](TODO.md). The Go implementation is retained for statistical reference capture.

## Quick start

```bash
mise run build
./dist/bin/rowing-machine --seed 42 --years 1
./dist/bin/rowing-machine --help
```

Defaults simulate four 365-day years starting on 2023-01-01 at scale 100 and write `raw_*.csv` under `factory-output/`. A random seed is printed when `--seed` is omitted; pass that seed to reproduce the files. Use `--quiet` to suppress console output.

## Documentation

- `SPEC.md` and `specs/` — what the tool does, with stable requirement IDs
- `docs/` — how it works: architecture, formulas, catalog, output schema
- `TODO.md` and `DONE.md` — planned and shipped work

## About the project

Rowing Machine descends from the Jaffle Shop Generator ([jafgen](https://pypi.org/project/jafgen/), [dbt-labs/jaffle-shop-generator](https://github.com/dbt-labs/jaffle-shop-generator)), which produced semi-realistic trends and patterns without simulating a full world of agents.
