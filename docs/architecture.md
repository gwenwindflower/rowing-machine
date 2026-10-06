# Architecture

Rowing Machine is one Cargo package with a library (`src/lib.rs`) and a thin binary (`src/main.rs`). The library holds everything testable; the binary parses flags and calls it.

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
    ecommerce/         markets, personas, orders, tweets, loyalty tiers
    saas/              accounts, subscriptions, usage, marketing, sales
    travel/            network, fleets, schedules, bookings, loyalty tiers
themes/                bundled theme TOML, compiled in with include_str!
tests/                 integration tests that drive the binary and read its files
benches/               throughput benchmarks
```

## Contracts between modules

These seams keep modules independent, so a change to one rarely touches another's files.

- **Streams.** `engine::stream::Stream::derive(seed, name, indices)` returns a PCG stream whose seed mixes all three with SplitMix64. Simulation draws go through named streams; nothing holds a stream across work units. Stream names are string constants owned by the module that uses them. The CLI alone obtains entropy when `--seed 0` requests a random seed.
- **Scenario.** A scenario declares its entity schemas and ordered work units, each tagged with a stage. The engine queues units for output and calls `observe` on their rows in declared order. After every queued write succeeds, `complete_stage` fixes accumulated facts before the next stage begins. A scenario never knows about threads or formats.
- **Entity schemas and rows.** `output::EntitySchema` carries the entity name, ordered columns with types and nullability, and the primary key, all declared by the scenario. Rows are `Vec<Value>` in schema order.
- **Writers.** Generation workers validate row shapes and values, extract typed primary keys, and encode CSV/JSONL bytes or Parquet Arrow batches per entity. `output::EntityWriter` appends those payloads without branching on scenario or theme. The output thread checks entity-wide key uniqueness and owns file buffering, compression, and Parquet row groups.
- **Themes.** A theme is data only: a name generator per name kind, a value list per label set, catalogs of typed records, and parameter values under `[params.<scenario>]`. Each scenario declares the name kinds, label sets, catalogs, and parameters (with defaults and ranges) it reads, and a theme is usable with the scenarios whose declarations it covers. Keep a scenario's declarations to what its simulation needs, since every one is something each theme must supply. `--param` overrides become theme parameter values before validation. The theme module never imports a scenario. Schemas never come from a theme.

Ecommerce stages emit static catalogs, orders/items, tweets, and customers. Order observation retains customer counts. The order stage's completion assigns loyalty tiers; the tweet stage regenerates market-day decisions from indexed streams to include final ranks without retaining orders. Customer rows follow market/customer-index order. The sink retains primary keys for duplicate detection, so memory grows with key count even though full entity rows are streamed.

`--workers` defaults to available cores. One worker generates and encodes serially; larger counts use a dedicated Rayon pool. Each stage runs in batches of at most four units per worker (one unit with one worker). Indexed collection preserves declared unit order even when generation finishes out of order. A bounded queue hands encoded units to one output thread while the engine observes rows and generates the next batch. The queue holds at most one batch, in addition to the generating batch and the unit being written. Payload memory depends on unit volume and worker count, not run duration; retained primary keys still grow with the dataset. UUID keys occupy `u128` hash sets, and composite keys retain typed values.

Every unit in a stage is observed and successfully written before `complete_stage` prepares state for the next stage. Generation errors are reported in declared unit order. Output errors stop the queue, survive channel shutdown, and prevent stage completion; the engine joins the writer before returning. Gzip compression and Parquet row groups stay on the output thread, so worker scheduling cannot change their byte stream.

SaaS stage zero emits day-indexed campaigns, spend, and touches. Observed touches identify sales prospects; stage completion assigns opportunities to reps with available capacity. Stage one emits plans, the rep roster, leads, accounts, opportunities, stage entries, and activities. Observation records account arrivals and won close dates. Its completion runs a lifecycle count pass to fix user-name offsets after all lead and rep names. Stage two generates each arrived account independently, emitting users, subscription intervals, MRR movements, invoices, sessions, and events. Sales accounts enter paid service on an observed won close; self-serve accounts use trial conversion. The simulation and count pass share the same lifecycle logic; the count pass skips usage generation. Usage derives from the completed account lifecycle, using separate personal, session, and event streams keyed by account, user, day, and session indices; each unit retains one account's dynamic rows, and the scheduler bounds the number of units held concurrently. See [the SaaS model](saas-model.md) for rates and interval semantics.

Parquet estimates entity volume from the first nonempty unit's row count and the number of units remaining in that stage. Row groups use `ceil(estimated_rows / TARGET_ROW_GROUPS)`, bounded to 1,024–65,536 rows. The estimate affects buffering only; writers preserve every row in its declared order. Gzip headers use a fixed zero modification time and omit filenames.

Target-row calibration starts with a 30-day sample and counts rows through the same generation, observation, and stage-completion contract without creating files. Each sample uses fresh scenario state. The search expands and refines a duration bracket until the calibration entity is within 5% of the target or the nearest whole day is known. Ecommerce calibrates on orders; the engine accepts the calibration entity and scenario factory as inputs. Calibration respects calendar bounds, prints a separate indicator before generation, and is silent under `--quiet`.

`ScenarioKind` registers the CLI scenario names and their theme requirements. `Theme` validates those requirements without importing a scenario. Name assignment uses a seeded weighted permutation of distinct whole-token expansions, and person indices span a scenario's population. `run_scenario` accepts a scenario kind and loaded pack; `run_with_theme` defaults to ecommerce and `run` selects `plain`. SaaS target-row calibration counts accounts. See [theme authoring](themes.md) for the file schema and exhaustion behavior.

## Dependencies

Keep the list short; a new crate needs a job no existing one does.

| Need | Crate |
| --- | --- |
| Errors, CLI | `anyhow`, `clap` |
| PCG streams, distributions | `rand`, `rand_core`, `rand_pcg`, `rand_distr` |
| Dates and times | `jiff` |
| CSV | `csv` |
| JSONL, theme files | `serde`, `serde_json`, `toml` |
| Parquet | `arrow`, `parquet` (zstd feature) |
| Gzip | `flate2` |
| Worker pool | `rayon` |
| Progress | `indicatif` |
| Tests and benchmarks | `assert_cmd`, `tempfile`, `criterion` (dev) |
