# Architecture

Rowing Machine is one Cargo package with a library (`src/lib.rs`) and a thin binary (`src/main.rs`). The library holds everything testable; the binary parses flags and calls it. The Go implementation in `go-reference/` is read-only reference for the ecommerce port and is deleted once the port is proven.

## Module layout

```text
src/
  main.rs              binary: parse the CLI, run, map errors to exit codes
  lib.rs               module tree and the public run entrypoint
  cli.rs               clap definitions, validation, help text
  engine/
    mod.rs             RunConfig, the run loop, stage and unit scheduling, progress
    stream.rs          seed derivation and PCG streams
    calendar.rs        day state: curves, seasons, weekday, hours
    calibration.rs     sample row counts and refine target duration
  output/
    mod.rs             EntitySchema, Value, Row, EntityWriter trait, OutputSink
    csv.rs             CSV writer
    jsonl.rs           JSONL writer, optionally gzip-compressed
    parquet.rs         Parquet writer, optionally zstd-compressed
  theme/
    mod.rs             Theme, bundled registry, path loading, validation
    names.rs           format expansion and without-replacement traversal
  scenario/
    mod.rs             Scenario trait and the scenario registry
    ecommerce/         markets, personas, orders, sparrows, guild ranks
    saas/              accounts, subscriptions, usage, marketing, sales
themes/                bundled theme TOML, compiled in with include_str!
tests/                 integration tests that drive the binary and read its files
benches/               throughput benchmarks
```

## Contracts between modules

These seams let Phases run in parallel without editing each other's files.

- **Streams.** `engine::stream::Stream::derive(seed, name, indices)` returns a PCG stream whose seed mixes all three with SplitMix64. Simulation draws go through named streams; nothing holds a stream across work units. Stream names are string constants owned by the module that uses them. The CLI alone obtains entropy when `--seed 0` requests a random seed.
- **Scenario.** A scenario declares its entity schemas and ordered work units, each tagged with a stage. The engine queues units for output and calls `observe` on their rows in declared order. After every queued write succeeds, `complete_stage` fixes accumulated facts before the next stage begins. A scenario never knows about threads or formats.
- **Entity schemas and rows.** `output::EntitySchema` carries the entity name, ordered columns with types and nullability, and the primary key, all declared by the scenario. Rows are `Vec<Value>` in schema order.
- **Writers.** Generation workers validate row shapes and values, extract typed primary keys, and encode CSV/JSONL bytes or Parquet Arrow batches per entity. `output::EntityWriter` appends those payloads without branching on scenario or theme. The output thread checks entity-wide key uniqueness and owns file buffering, compression, and Parquet row groups.
- **Themes.** A theme is data only: a name generator per name kind and a value list per label set. Each scenario declares the name kinds and label sets it needs, and a theme is usable with the scenarios whose declarations it covers. Keep a scenario's declared kinds to the columns that need them, since every one is a generator each theme must supply. The theme module never imports a scenario. Numbers, counts, and schemas never come from a theme.

Ecommerce stages emit static catalogs, orders/items, sparrows, and customers. Order observation retains customer counts. The order stage's completion assigns guild ranks; the sparrow stage regenerates market-day decisions from indexed streams to include final ranks without retaining orders. Customer rows follow market/customer-index order. The sink retains primary keys for duplicate detection, so memory grows with key count even though full entity rows are streamed.

`--workers` defaults to available cores. One worker generates and encodes serially; larger counts use a dedicated Rayon pool. Each stage runs in batches of at most four units per worker (one unit with one worker). Indexed collection preserves declared unit order even when generation finishes out of order. A bounded queue hands encoded units to one output thread while the engine observes rows and generates the next batch. The queue holds at most one batch, in addition to the generating batch and the unit being written. Payload memory depends on unit volume and worker count, not run duration; retained primary keys still grow with the dataset. UUID keys occupy `u128` hash sets, and composite keys retain typed values.

Every unit in a stage is observed and successfully written before `complete_stage` prepares state for the next stage. Generation errors are reported in declared unit order. Output errors stop the queue, survive channel shutdown, and prevent stage completion; the engine joins the writer before returning. Gzip compression and Parquet row groups stay on the output thread, so worker scheduling cannot change their byte stream.

SaaS emits plans and day-indexed account arrivals, observes those arrivals, and fixes user-name offsets from a lifecycle count pass at the stage boundary. Each account's lifecycle is then an independent work unit emitting users, subscription intervals, MRR movements, and invoices. The simulation and count pass share the same logic; each unit retains one account's dynamic rows, and the scheduler bounds the number of units held concurrently. See [the SaaS model](saas-model.md) for rates and interval semantics.

Parquet estimates entity volume from the first nonempty unit's row count and the number of units remaining in that stage. Row groups use `ceil(estimated_rows / TARGET_ROW_GROUPS)`, bounded to 1,024–65,536 rows. The estimate affects buffering only; writers preserve every row in its declared order. Gzip headers use a fixed zero modification time and omit filenames.

Target-row calibration starts with a 30-day sample and counts rows through the same generation, observation, and stage-completion contract without creating files. Each sample uses fresh scenario state. The search expands and refines a duration bracket until the calibration entity is within 5% of the target or the nearest whole day is known. Ecommerce calibrates on orders; the engine accepts the calibration entity and scenario factory as inputs. Calibration respects calendar bounds, prints a separate indicator before generation, and is silent under `--quiet`.

`ScenarioKind` registers the CLI scenario names and their theme requirements. `Theme` validates those requirements without importing a scenario. Name assignment uses a seeded weighted permutation of distinct whole-token expansions, and person indices span a scenario's population. `run_scenario` accepts a scenario kind and loaded pack; `run_with_theme` defaults to ecommerce and `run` selects `plain`. SaaS target-row calibration counts accounts. See [theme authoring](themes.md) for the file schema and exhaustion behavior.

## Dependencies

Add each crate in the Phase that first needs it, and keep the list short. These are the intended picks; a Phase may swap one with a line in its DONE narrative.

| Need | Crate | Phase |
| --- | --- | --- |
| Errors, CLI | `anyhow`, `clap` | 8 |
| PCG streams, distributions | `rand`, `rand_core`, `rand_pcg`, `rand_distr` | 9 |
| Dates and times | `jiff` | 9 |
| CSV | `csv` | 9 |
| Progress | `indicatif` | 9 |
| CLI integration tests | `assert_cmd`, `tempfile` (dev) | 9 |
| Parity fixture JSON | `serde`, `serde_json` (dev) | 9 |
| JSONL | `serde`, `serde_json` | 1 |
| Parquet | `arrow`, `parquet` (zstd feature) | 1 |
| Gzip | `flate2` | 1 |
| Worker pool | `rayon` | 2 |
| Theme files | `toml`, `serde` | 3 |
| Benchmarks | `criterion` (dev) | 9 |

## Porting map

| Go reference | Rust home |
| --- | --- |
| `go-reference/cmd/rowing-machine/main.go` | `src/cli.rs`, `src/main.rs` |
| `go-reference/internal/simulation/curves.go`, `day.go` | `src/engine/calendar.rs` |
| `go-reference/internal/simulation/simulation.go` | `src/engine/mod.rs`, `src/scenario/ecommerce/` |
| `go-reference/internal/simulation/ranks.go` | `src/scenario/ecommerce/mod.rs` stage completion |
| `go-reference/internal/simulation/progress.go` | `src/engine/mod.rs` progress |
| `go-reference/internal/market/market.go` | `src/scenario/ecommerce/mod.rs` indexed pools and market-day generation |
| `go-reference/internal/models/customer.go` | `src/scenario/ecommerce/persona.rs` |
| `go-reference/internal/models/order.go`, `item.go`, `sparrow.go` | `src/scenario/ecommerce/` |
| `go-reference/internal/models/store.go`, `internal/catalog/` | numbers in `src/scenario/ecommerce/catalog.rs`; names and labels in `themes/fantasy_rpg.toml` from Phase 3 |
| `go-reference/internal/output/` | `src/output/` |

The Go code threads one RNG per market through every day, so its output depends on draw order. The port replaces that with per-unit streams (`sm-R030`); match the Go behavior statistically (`dev-R018`), not draw for draw.

## Build lanes

After the engine port (Phase 9), work splits into lanes whose files barely overlap:

```text
8 ─ 9 ─┬─ 1 ─ 2 ──────┬─ 13
       └─ 3 ─┬────────┘
             └─ 5 ─┬─ 10
                   └─ 11 ─ 12
```

- Phase 1 owns `src/output/`; Phase 3 owns `src/theme/`, `themes/`, and swapping names in `src/scenario/ecommerce/` for theme lookups. They can run as parallel sessions.
- Phase 2 touches `src/engine/` scheduling and the sink; it follows Phase 1 because both change the sink.
- Phases 5, 10, 11, and 12 build `src/scenario/saas/`. They never touch scheduling, so they run alongside Phase 2. Phases 10 and 11 can run in parallel.
- Phase 15 (parallel output throughput) owns engine scheduling and `src/output/`; it can run alongside the SaaS Phases.
- Phase 14 (release plumbing) touches only tasks, tests, workflows, and `Cargo.toml` metadata, so it runs alongside any lane.
- Phase 13 retires the Go reference and cuts the first Rust release; SaaS Phases can still be open when it lands.
