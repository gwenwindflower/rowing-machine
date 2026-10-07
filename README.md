# Rowing Machine

`rowing-machine` generates deterministic synthetic business data for SQL training, analytics engineering demos, and evaluating data tools. Pick a scenario, a seed, and a duration, and it writes relational CSV, JSONL, or Parquet files you can load anywhere. The same seed and flags always produce byte-identical files, so a lesson can pin a dataset forever or sweep seeds for fresh data of the same shape.

| Scenario | Simulates | Default theme |
| --- | --- | --- |
| `ecommerce` (default) | Six stores selling to growing customer pools: orders, items, products, supplies, customer posts, loyalty tiers | `plain` |
| `saas` | A B2B software company: marketing, a sales pipeline, trials, subscriptions, invoices, and product usage | `plain` |
| `travel` | A transport network: locations, routes, vehicles, scheduled trips with delays, bookings, tickets, and add-ons | `airline` |

## Installation

With [Cargo Binstall](https://github.com/cargo-bins/cargo-binstall#installation):

```bash
cargo binstall rowing-machine
```

Build from crates.io with Cargo:

```bash
cargo install rowing-machine --locked
```

Or download a [release archive](https://github.com/gwenwindflower/rowing-machine/releases) for Linux or macOS on Intel or ARM, verify its SHA-256 checksum, and put `rowing-machine` on your `PATH`.

## Quick start

```bash
rowing-machine --seed 42
rowing-machine --scenario saas --seed 42 --years 2
rowing-machine --scenario travel --seed 42
rowing-machine --help
```

Defaults simulate four 365-day years from 2023-01-01 at scale 100 and write `raw_<table>.csv` files under `./factory-output/`. Without `--seed`, a random seed is chosen and printed; pass it back to reproduce the files.

| Flag | Default | Effect |
| --- | --- | --- |
| `--scenario` | `ecommerce` | `ecommerce`, `saas`, or `travel` |
| `--seed` | `0` (random, printed) | Same seed and flags give identical files |
| `--years` | `4` | Run length in 365-day years |
| `--target-rows` | Unset | Picks a run length for about this many orders, accounts, or tickets (within 5%); conflicts with `--years` |
| `--scale` | `100` | Population multiplier |
| `--start-date` | `2023-01-01` | First simulated day |
| `--format` | `csv` | `csv`, `jsonl`, or `parquet` |
| `--compress` | Off | gzip JSONL (`.jsonl.gz`) or zstd Parquet; not for CSV |
| `--output-dir`, `--pre` | `./factory-output`, `raw` | Where files go and their filename prefix |
| `--workers` | Available cores | Worker threads; output is identical at any count |
| `--quiet` | Off | No seed, progress, or row summary |

```bash
rowing-machine --seed 42 --target-rows 100000 --format parquet --compress
```

## Themes

A theme renames tables and columns, generates names, and supplies labels, catalogs, and parameter values. It never changes which tables exist or how they join.

| Theme | Scenarios | Skin |
| --- | --- | --- |
| [`plain`](docs/themes/plain.md) | ecommerce, saas | Neutral shop and software vocabulary |
| [`fantasy_rpg`](docs/themes/fantasy_rpg.md) | ecommerce | The Arcanum Collective mage guild, for the Queria SQL trainer |
| [`sneakers`](docs/themes/sneakers.md) | ecommerce | Starcloud Sneakers, a running shoe brand |
| [`airline`](docs/themes/airline.md) | travel | SuperAir, a low-cost airline with six UK bases |

```bash
rowing-machine themes
rowing-machine --theme fantasy_rpg --seed 42
rowing-machine --theme ./shop.toml
rowing-machine --theme sneakers --param price_scale=10
rowing-machine --scenario travel --param route_density=0.2 --param daily_frequency=2
```

`--param name=value` overrides one scenario parameter for a run and is repeatable. Each scenario reference lists its parameters and ranges.

## Documentation

| Doc | Covers |
| --- | --- |
| [Scenarios and themes](docs/scenarios/README.md) | How scenarios and themes fit together, shared run controls |
| [Ecommerce](docs/scenarios/ecommerce.md), [SaaS](docs/scenarios/saas.md), [Travel](docs/scenarios/travel.md) | Each scenario's entities, volume, rules, parameters, and example queries |
| [Themes](docs/themes/README.md) | Bundled theme pages, theme TOML schema, name assignment, renames, validation |
| [Output schema](docs/output-schema.md) | Every file, column, and type encoding |
| [Architecture](docs/architecture.md) | Modules, the run pipeline, and contracts between them |
| [Performance](docs/performance.md) | Benchmark method and recorded results |
| [SPEC.md](SPEC.md) and [specs/](specs/) | Requirements with stable IDs |

## About the project

Rowing Machine descends from the Jaffle Shop Generator ([jafgen](https://pypi.org/project/jafgen/), [dbt-labs/jaffle-shop-generator](https://github.com/dbt-labs/jaffle-shop-generator)), which produced semi-realistic trends and patterns without simulating a full world of agents.
