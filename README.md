# Rowing Machine

`rowing-machine` is a command-line synthetic data generator powering **Queria**, a retro-RPG-inspired SQL trainer. It simulates the **Arcanum Collective** — a mage guild running six guild halls across the fantasy land of Queria — and produces seven CSVs covering stores, customers, orders, items, products, supplies, and sparrows.

The simulation drives realistic temporal patterns from behavioral personas, seasonality curves, growth, market penetration, and store hours of operation. Runs are byte-deterministic from a single seed, which makes the output equally useful as fixed fixtures (pin a seed in a lesson and the data never moves) or as variable training corpora (sweep seeds to evaluate AI data tools against unseen but valid-shaped data).

Built in Go for speed, scale, and single-binary distribution.

## Quick start

```bash
go run ./cmd/rowing-machine                       # 2023–2026 × scale 100, random seed, ./factory-output/
go run ./cmd/rowing-machine --seed 42 --years 1   # reproducible 1-year run
go run ./cmd/rowing-machine --help                # full flag reference
```

## Documentation

- `SPEC.md` — project contract and domain spec index
- `specs/` — durable per-domain requirements (simulation, catalog, output, CLI)
- `docs/` — implementation references (formulas, static data, output schema)
- `TODO.md` / `DONE.md` — active and shipped work

## About the project

`rowing-machine` is a Go rewrite of the original **Jaffle Shop Generator** (`[jafgen](https://pypi.org/project/jafgen/)` on PyPI, [dbt-labs/jaffle-shop-generator](https://github.com/dbt-labs/jaffle-shop-generator) on GitHub) for generating simulated data for the Jaffle Shop project, outputting semi-realistic trends and patterns without building a full blown simulated world of agents and interactions.
