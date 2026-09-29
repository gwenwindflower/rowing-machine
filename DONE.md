# Rowing Machine — DONE

## Phase 15: Parallel output throughput ✅

**Dependencies**: 2
**Requirements**: sm-R035, sm-R036, sm-R033, op-R017, op-R018, op-R023, R001, dev-R016, dev-R017, dev-R020, dev-R031, dev-R032

### Profiling harness

- [x] Add `mise run profile` that builds with symbols into a scratch target dir and records a profile (`samply` on macOS and Linux) of a given invocation
- [x] Add the scale-100 run to `mise run bench` and record one-worker and all-core baselines in `docs/performance.md`
- [x] Bisect the serial regression between the Phase 9 baseline and Phase 2, and note the cause in `docs/performance.md`

### Row path off the serial thread

- [x] Move row validation and format serialization into the worker that generated the unit, handing the writer finished bytes per entity
- [x] Replace string key sets with typed keys (UUIDs as `u128`) in a hash set, or prove key uniqueness by construction and check it in tests only, keeping `op-R018`
- [x] Format UUIDs, timestamps, and integers into reusable buffers without `core::fmt`

### Overlapped ordered writing

- [x] Stream finished units to writers through a bounded, ordered handoff so workers keep generating while earlier units are written
- [x] Write each entity's file on its own thread, or show with the profile that one writer thread keeps up
- [x] Keep Parquet and compressed JSONL byte-identical across worker counts, with encoding in parallel where the format allows it

### Proof

- [x] Show `sm-R035` and `sm-R036` in `mise run bench`, keep every byte-identity test passing at 1 and all workers, and update `docs/performance.md`

Generation workers validate rows and prepare CSV/JSONL bytes or Arrow batches. A bounded ordered queue overlaps generation with one output thread; stage barriers wait for writing before scenario state advances. UUID primary keys use compact `u128` hash sets, composite keys retain typed values, and failed writes release key reservations. Full entity rows remain bounded by worker count, while key storage grows with the dataset.

Adjacent-revision measurements isolate the serial regression to `c58606e`, which formatted CSV fields twice. The symbolized samply profile shows the output thread waiting for work in 46.7% of weighted samples, supporting one writer rather than a thread per entity. Compression and Parquet row-group assembly stay ordered; Arrow conversion runs in workers.

The full ten-sample benchmark writes 4,158,194 ecommerce rows in 1.7075 seconds with one worker and 628.72 milliseconds with ten: 2.4352 million serial rows/sec and a 2.7159× speedup. Both performance requirements pass. The benchmark also covers ecommerce scale 10 and SaaS scale 10, with per-case peak memory; measurements and intervals live in `docs/performance.md`.

Validation: `mise run check` passed all 103 Rust tests, Clippy, hooks, workflow-task checks, optimized build, and crate packaging. Byte comparisons cover both scenarios, compatible bundled themes, every format, and compression at one and all available workers. Four Objective commits are left for the user's Worktrunk merge.

## Phase 2: Worker-parallel generation ✅

**Dependencies**: 1
**Requirements**: cl-R030, sm-R030, sm-R033, R001, dev-R016, dev-R020

### Parallel scheduler

- [x] Generate work units on a `rayon` pool and reorder finished units so the sink receives them in declared unit order with bounded memory
- [x] Wire `--workers`, defaulting to available cores and rejecting `0`
- [x] Test that `--workers 1` and `--workers 8` byte-match for every scenario and format

### Throughput

- [x] Extend `mise run bench` to worker counts 1 and all cores, and record the results in `docs/performance.md`

Generation uses a dedicated Rayon pool with ordered batches of at most four work units per worker. Batches bound pending row payloads without channels or a separate reorder queue; stage completion follows all writes and observations. One worker uses the serial path. Primary-key retention and scenario state still scale with the dataset.

Worker comparisons cover ecommerce, both bundled themes, every format, and supported compression modes. Ecommerce was the only registered scenario when this Phase closed; SaaS is developed separately in Phase 5. Scheduler tests force out-of-order completion, verify stage barriers and bounded pending units, and exercise generation and observation failures.

`mise run bench` measured 456,850 rows/sec with one worker and 449,130 with 10 available cores, at 70.81 and 74.27 MiB peak process RSS respectively. Overlapping intervals show no demonstrated speedup for this end-to-end CSV workload; see `docs/performance.md` for the measurement limits.

Validation: `mise run check` passed all 78 Rust tests, Clippy, hooks, task-selection checks, the optimized build, and packaged-crate compilation. Left as two Objective commits for the user's merge.

## Phase 1: Flexible output controls ✅

**Dependencies**: 9
**Requirements**: cl-R020, cl-R021, cl-R022, cl-R023, op-R001, op-R020, op-R021, op-R022, op-R023, R001

### JSONL output

- [x] Add the JSONL writer with native numbers, booleans, and nulls, and wire `--format jsonl`
- [x] Test byte-identical JSONL across two runs with the same seed

### Parquet output

- [x] Add the Parquet writer with `arrow` and `parquet`, mapping cents to int64 and timestamps to `TIMESTAMP_MICROS` UTC
- [x] Derive row group size from estimated row count through one named constant, and wire `--format parquet`
- [x] Test byte-identical Parquet across two runs, and read a file back to check types

### Compression

- [x] Add `--compress`: gzip for JSONL (`.jsonl.gz`), zstd column compression for Parquet
- [x] Reject `--compress` with CSV, suggesting `jsonl` or `parquet`

### Target-row calibration

- [x] Estimate the duration that yields `--target-rows` rows of the scenario's calibration entity by sampling a short run
- [x] Show a calibration indicator distinct from generation progress, and reject `--target-rows` with `--years`
- [x] Test that a calibrated run lands within a stated tolerance of the target

The sink dispatches validated rows to CSV, JSONL, or Parquet writers and retains lazy file creation. JSONL preserves schema order and native types. Arrow/Parquet 60 stores int64 cents and UTC microsecond timestamps; row groups target eight groups from the first nonempty unit's estimated volume, bounded to 1,024–65,536 rows. Gzip headers omit filenames and use zero modification time for reproducibility.

Calibration samples fresh scenarios without output, starting with 30 days and refining a duration bracket rather than assuming linear growth. The tolerance is 5%, with the nearest attainable whole day used for targets finer than day granularity. The engine accepts a scenario factory and calibration entity; ecommerce selects orders. The separate calibration indicator prints before generation and respects quiet mode.

Validation: `mise run check` passed with 54 Rust tests, Clippy, hooks, versioning checks, and Go statistical parity. Coverage includes compressed and uncompressed byte determinism, Parquet type/null/value readback, compression metadata, JSONL escaping and large integers, target tolerance, and conflicting flags. No scenario or theme implementation changed; shared CLI, library wiring, docs, and Cargo dependencies may need reconciliation with Phases 3 and 14. Four Objective commits are left for the user's merge.

## Phase 0: Go port + fantasy retheme

Pre-SPOT work captured retroactively. This Phase covers the rewrite of the original Python `jaffle-shop-generator` into Go, plus the subsequent re-theming from generic outdoor / sandwich-shop vocabulary to the Arcanum Collective mage-guild theme that ships today.

Commits: `fbc8dca` (initial pre-migration), `9076457` (feat: completed migration), `a1ddd6d` (refactor: shift to fantasy theme), `44815df` (fix: utilize full name pool randomly).

### Why Go

- **Speed and scale.** The Python version was slow enough that interesting simulation knobs (millions of rows, many years, dense persona behavior) weren't viable. A Go port unblocks the actual goal — generating useful training data at a meaningful scale.
- **Single-binary distribution.** Aligns with the Supermodel Labs DX model: Homebrew + Linux package managers, no runtime dependencies.
- **Determinism story.** Go's `math/rand/v2` PCG streams give a clean per-subsystem seeding model; the Python version's NumPy + Faker stack was harder to pin precisely.

### Why the fantasy retheme

- **Queria fit.** The downstream consumer (Queria — a retro-RPG SQL trainer) wanted vocabulary that matches its UI, not generic sandwich-shop or outdoors lingo.
- **More distinctive teaching surface.** Guild ranks, power levels, reagents, and sparrows give learners memorable joins to practice on — the fantasy nouns stick better than `customers` / `orders` / `items` alone.

### What landed

- **Architecture.** `cmd/rowing-machine/main.go` Cobra entrypoint; `internal/{catalog,market,models,simulation,output}` packages; `internal/simulation/integration_test.go` E2E suite covering determinism and referential integrity.
- **Domain model.** Six guild halls, fifteen products (5 weapons / 5 armor / 5 elixirs across common→legendary), 29 supplies denormalized to 65 rows, six personas (Courier, Artificer, FeastReveler, Apprentice, Wanderer, Herbalist), sparrows replacing tweets.
- **Output.** Seven CSV files (`stores`, `customers`, `orders`, `items`, `products`, `supplies`, `sparrows`) at `{output-dir}/{prefix}_{entity}.csv`.
- **RNG discipline.** Single `--seed` controls all randomness; per-market PRNG seeded `seed + marketIndex + 1`; store UUIDs use a bare-seed PRNG so store IDs are stable regardless of market simulation order.
- **Money as cents.** All currency persisted as `int64`; tax computed once via `int64(math.Round(float64(subtotal) * taxRate))`.
- **Determinism guarantee.** Same seed in → byte-identical CSV out, verified in integration tests.

### Lessons rolled forward

- **Pre-compute day state.** Annual × weekend × growth into `[]DayState` before the loop. No per-day allocations. Captured as `sm-R013`.
- **Per-market PRNG isolation.** Adopted to avoid cross-market contamination when scale or persona weights change. Captured as `sm-R011`.
- **Streaming output where size is unbounded.** Orders, items, sparrows write through buffered writers; only customers stay in memory (dedup map). Captured as `op-R015`.
- **Full-pool name draws.** Sequential name draws clustered around the start of the pool. `44815df` fixed the draw to use full-pool random sampling. Captured as `dt-R015`.

### Reference material retired

- The original 1834-line `MIGRATION.md` port reference was deleted at the close of Phase 0. The Python source remains accessible at [dbt-labs/jaffle-shop-generator](https://github.com/dbt-labs/jaffle-shop-generator) if anyone needs to revisit it.
- `SCHEMA.md` (top-level CSV schema reference) was deleted in favor of `docs/output-schema.md`, which is the canonical schema doc and now backed by `op-R008`–`op-R014`.

Closes Phase 0.

## Phase 6: Conference dataset hardening ✅

**Requirements**: cl-R001, cl-R004, cl-R012, sm-R007, dt-R007, dt-R009, dt-R010, op-R013, op-R017, op-R018, R004

The default run spans 2023–2026, activates all six guild halls, and produces current data without extra flags. Validation rejects invalid generation sizes before output work begins, while writer and integration checks enforce rectangular CSV rows, populated fields, and unique relational keys.

The catalog contract reflects its 41 supplies and 92 `(id, sku)` relationships. Each product type covers all five power levels exactly once, giving demo queries a complete categorical dimension.

### Current default dataset

- [x] Set the default simulation window to 2023–2026 so a no-argument run produces current conference data and includes every guild hall
- [x] Reject non-positive `--years` and `--scale` values before creating output
- [x] Cover defaults and validation through the CLI boundary

### Clean relational output

- [x] Reject output rows whose field count differs from the entity schema
- [x] Verify every generated primary key is non-empty and unique, every persisted field is non-empty, and supplies use the `(id, sku)` composite key
- [x] Ensure each product type covers every power level exactly once
- [x] Pin the current 41-supply roster and 92-row denormalized output
- [x] Update user-facing schema and quick-start documentation for the current defaults and supply key

## Phase 7: Balanced guild rank cohorts ✅

**Requirements**: sm-R024, R001

Guild ranks are relative order-frequency cohorts rather than fixed lifetime-order thresholds. Ordering customers are sorted by order count with UUID tie-breaking, then divided into four cohorts whose sizes differ by at most one. This preserves deterministic output and the progression from lower-activity initiates to higher-activity masters at every simulation duration and scale.

A seeded four-year, scale-10 validation run produced 152 customers in each rank. Order counts progressed from 1–61 for initiates, 61–185 for journeymen, 187–410 for adepts, and 411–890 for masters.

### Order-count quartiles

- [x] Replace fixed lifetime-order thresholds with deterministic order-count quartiles
- [x] Prove cohort sizes differ by at most one and rank never decreases as order count rises
- [x] Cover deterministic UUID tie-breaking for customers with equal order counts
- [x] Verify the default-style simulation produces balanced guild ranks without changing the customer schema
- [x] Document guild ranks as relative order-frequency cohorts

## Phase 8: Rust foundation ✅

**Requirements**: dev-R013, dev-R014, dev-R015, dev-R021

The Rust rewrite is planned and wired, but not yet implemented. `docs/adr/2026-09-27-rust-rewrite.md` records why the project moves to Rust, why parity with Go is statistical rather than byte-for-byte, and which shipped requirements changed. The work runs on the `feat/rust-rewrite` integration branch with the Go code moved to `go-reference/` as read-only reference.

Planning changes: the old Phase 4 (messy mode) and `cl-R050` moved to the Backlog. The old TODO Phase 6 (theme-native names) folded into Phase 3; its number had also collided with the shipped Phase 6 below. Phase 5 became the SaaS scenario, split across Phases 5, 10, 11, and 12 by entity group. Scenario and theme are separate concepts: a scenario is the business model, a theme its vocabulary. Parallel safety is designed into the engine in Phase 9 (`sm-R030`–`sm-R034`), so Phase 2 only adds the scheduler.

The template's bootstrap script rejected the worktree because it checks for a `.git` directory rather than a `.git` file, and its `pinact run -update` call uses a flag pinact 5 dropped; both were worked around here and are worth fixing in the skill.

### Template alignment

- [x] Move the Go implementation to `go-reference/`
- [x] Install the `_tool` template and Rust kit: mise tasks, prek hooks, CI and release workflows, worktrunk merge gates
- [x] Reconcile `.gitignore`, `README.md`, and `AGENTS.md` with the template

### Rewrite plan

- [x] Rewrite `SPEC.md` and the domain specs for scenarios, themes, and the Rust engine, and add the SaaS and engineering specs
- [x] Record the rewrite decision in an ADR
- [x] Plan Phases 9–13 with dependencies, lanes, and requirement coverage

### Crate skeleton

- [x] Split the crate into a library and binary, and declare the stream, output, and scenario contracts
- [x] Pass `mise run check`

## Phase 9: Ecommerce engine port ✅

**Dependencies**: 8
**Requirements**: R001, R002, R004, R005, sm-R001, sm-R002, sm-R003, sm-R004, sm-R005, sm-R006, sm-R007, sm-R008, sm-R009, sm-R010, sm-R011, sm-R012, sm-R013, sm-R015, sm-R016, sm-R017, sm-R018, sm-R019, sm-R020, sm-R021, sm-R022, sm-R023, sm-R024, sm-R025, sm-R026, sm-R030, sm-R031, sm-R032, sm-R033, dt-R001, dt-R002, dt-R003, dt-R004, dt-R005, dt-R006, dt-R007, dt-R008, dt-R009, dt-R010, dt-R011, dt-R012, dt-R013, dt-R016, dt-R017, op-R001, op-R002, op-R003, op-R004, op-R005, op-R006, op-R007, op-R008, op-R009, op-R010, op-R011, op-R012, op-R013, op-R014, op-R015, op-R016, op-R017, op-R018, op-R024, cl-R001, cl-R002, cl-R003, cl-R004, cl-R005, cl-R006, cl-R007, cl-R010, cl-R011, cl-R012, dev-R016, dev-R017, dev-R018, dev-R019, dev-R020

Port the Go ecommerce simulation onto the scenario engine, serially, with per-unit streams from the start. Read `go-reference/` for behavior and `docs/architecture.md` for where each piece lands. Catalog data lives in Rust constants until Phase 3 moves it into themes.

### Go reference parity fixture

- [x] Add a test-support module that reads an ecommerce output directory and computes the `dev-R019` statistics as JSON
- [x] Build the Go reference, run it at seed 42, scale 10, four years, and capture `tests/fixtures/go-reference-stats.json` with a tolerance per statistic
- [x] Document the capture command beside the fixture so it can be rerun

### Streams and calendar

- [x] Wrap `stream_seed` in a PCG stream type using `rand_pcg` and `rand_distr`, with stream-name constants per consumer
- [x] Port the annual, weekend, growth, and penetration curves with table tests against the Go values
- [x] Pre-compute day state (curves, season, weekday, hours) for the whole run with `jiff` dates

### Output sink and CSV writer

- [x] Implement the sink that validates rows against their entity schema and routes them to per-entity writers in unit order
- [x] Implement the buffered CSV writer with lazy file creation, ISO timestamps, cents, UUIDs, and `True`/`False` booleans
- [x] Test ragged rows, empty keys, duplicate keys, nullable columns, and zero-row entities

### Ecommerce scenario

- [x] Port guild halls, products, supplies, personas, and sparrow vocabulary as Rust constants under `src/scenario/ecommerce/`
- [x] Port markets with customer pools derived per customer index, and day generation per market-day unit
- [x] Port personas, order construction, tax, items, and sparrows
- [x] Compute customer dedup and balanced guild rank cohorts after generation from emitted rows

### CLI and run loop

- [x] Define the clap CLI with every `cl-R001`–`cl-R012` flag, verbose help, and actionable validation errors
- [x] Implement the serial run loop with `indicatif` progress, the seed print, and the per-entity row summary
- [x] Cover defaults and validation through the binary with `assert_cmd`

### End-to-end proof

- [x] Add integration tests for byte-identical reruns, primary and foreign keys, and parity against the Go fixture
- [x] Add `benches/` with a criterion throughput benchmark and a `mise run bench` task that reports rows per second and peak memory
- [x] Update `docs/simulation.md`, `docs/static-data.md`, and `docs/output-schema.md` to point at the Rust code

The Rust CLI generates all seven ecommerce entities with indexed PCG streams, a precomputed calendar, validated CSV output, and serial stage execution. Orders feed customer counts; final ranks feed a second deterministic pass for sparrows. Primary-key sets grow with row count, while full rows stream by market-day.

Go parity is captured through a reproducible, capture-only overlay for persona metadata, leaving reference sources untouched. Independent persona draws initially underrepresented Herbalists, so indexed shuffled blocks preserve the reference's fixed mixture without weakening fixture tolerances. Catalog origins follow guild-hall requirements; product columns follow the specified schema. Customer names remain deterministic hall/index labels pending Phase 3 themes. `rand` supports the PCG distributions and CLI seed selection; `serde` and `serde_json` are dev dependencies for fixture capture ahead of JSONL output.

Validation: `mise run check` passed, including 36 Rust tests, statistical parity, binary determinism, relational integrity, Clippy, hooks, and versioning checks. The release binary smoke test wrote 417,590 rows at seed 42, scale 10, four years. `mise run bench` measured 717,380 rows/sec and 71.75 MiB maximum resident memory for the Criterion process; details are in `docs/performance.md`. Worker-count comparisons remain with Phase 2. The branch is left for the user's merge into `feat/rust-rewrite`.

## Phase 14: Release and crates.io plumbing ✅

**Requirements**: dev-R023, dev-R024, dev-R025, dev-R026, dev-R027, dev-R028, dev-R030, R003

Brings the release pipeline up to the heraldr pattern (`~/dev/herdr/heraldr`): crates.io publishing, asset recovery, and the everyday dev and dependency tasks. Port from heraldr, keeping Homebrew, which heraldr does not use. Generic template tasks (labels, rulesets, publishing, recovery, versioning) are tested in the `_tool` template, not here; this project tests only its own task config. Touches `mise.toml`, `mise-tasks/`, `tests/*.sh`, `.github/workflows/`, and `Cargo.toml` package metadata, so it can run alongside Phases 1 and 3; expect a small `Cargo.toml` rebase.

### Crate publishing

- [x] Add `include`, `keywords`, and `categories` to `Cargo.toml`, and a `test:crate` task running `cargo package --locked --allow-dirty`
- [x] Port `release:crate-preflight`, `release:publish-crate`, and the confirmed `release:bootstrap-crate` task
- [x] Add the `crate` job to `release-build.yml` behind `CRATES_IO_PUBLISHING`, in the `release` environment with `id-token: write` and `rust-lang/crates-io-auth-action`, after the asset upload

### Release recovery and test tasks

- [x] Port `release:recover-assets`
- [x] Port heraldr's task-workflow test as `test:workflows`, checking that `check` and CI never select `dev:` or other interactive tasks
- [x] Remove `tests/versioning.sh` and `test:versioning`, which test template-generic tasks
- [x] Add `test:build` and an aggregate `test` task

### Everyday tasks and CI hygiene

- [x] Add `dev:build` and `dev:test` (`cargo pretty`, `raw = true`) and `deps:check`, `deps:update`, and `deps:audit`, keeping every `dev:` task out of `check`
- [x] Match heraldr's CI and hook settings: `MISE_TASK_OUTPUT` and `MISE_JOBS` in `ci.yml`, `GH_REPO` on the asset upload, `default_stages` in `prek.toml`, zizmor cache `allow_write`, and `jq` in `[tools]`
- [x] Derive ruleset status checks from the latest completed push run of `ci.yml` on `main`
- [x] Order the README install section: Homebrew, `cargo binstall`, `cargo install --locked`, release archive

Crate publishing follows heraldr's source/tag and eight-asset preflight, confirmed local bootstrap, and OIDC CI publication after binary uploads. Homebrew remains a separate downstream job. Recovery checks the completed run, release tag commit, and archive checksums before uploading. Ruleset provisioning selects only the latest completed CI push run on main, satisfying dev-R028 without requiring release jobs.

The crate allowlist includes Rust sources, benchmarks, examples, test support and fixtures, and bundled themes; it excludes Go reference sources and repository automation. Generic release-task tests remain the template's responsibility. The project test checks dry-run selections for local gates and CI, rejecting development, installation, and publication tasks and requiring each suite once. The prek stage setting landed with crate publishing because prek requires its config changes to be staged at commit time.

Validation: `mise run check` passed all 36 Rust tests, task-selection checks, the optimized build, packaged-crate compilation, Clippy, and hooks. `mise run ci-audit` passed zizmor and action-pin verification. Publishing, asset uploads, and repository provisioning were not executed; first publication and trusted-publisher setup remain in Phase 13. Left as three Objective commits for the user's merge.

## Phase 3: Theming system and native names ✅

**Dependencies**: 9
**Requirements**: th-R001, th-R002, th-R003, th-R004, th-R005, th-R006, th-R007, th-R008, th-R009, th-R010, th-R011, th-R012, th-R013, th-R014, th-R015, th-R016, th-R017, th-R018, cl-R040, cl-R042, R001

Owns `src/theme/`, `themes/`, and swapping hardcoded names in `src/scenario/ecommerce/` for theme lookups. Can run in parallel with Phase 1. Themes only generate names and labels; scenarios keep every number.

### Theme contract and loader

- [x] Define the theme TOML schema: name, description, a name generator per name kind, and a value list per label set
- [x] Let each scenario declare the name kinds and label sets it needs, and check theme compatibility against the selected scenario
- [x] Parse and validate themes with `toml` and `serde`, rejecting bad files before simulation with the file and field named
- [x] Compile bundled themes into the binary and load path themes through `--theme`

### Native name generation

- [x] Expand weighted name formats over whole-token component pools, per name kind
- [x] Map entity index to a unique combination with a seeded bijective permutation, and define reuse after exhaustion
- [x] Test traceability, exhaustion, run-wide uniqueness, and that name config changes leave every other field unchanged

### Bundled themes

- [x] Move ecommerce names and labels (guild halls, products, product types, power levels, ranks, sparrow vocabulary) into `themes/fantasy_rpg.toml`, leaving numbers in Rust, and write `themes/plain.toml`
- [x] Review and check in component pools for both themes, sized for the default population
- [x] Add `rowing-machine themes` and make `plain` the default
- [x] Pin a seeded name snapshot per theme and update `docs/static-data.md`

Themes use strict TOML parsing, scenario-declared coverage, and ordered label lengths. Ecommerce needs only the `person` generator; static catalog names, descriptions, ranks, and message vocabulary use ordered label sets to preserve catalog relationships. Both bundles cover ecommerce; Phase 5 adds SaaS declarations and the matching `plain` entries.

Whole-token expansions are deduplicated before a seeded weighted permutation assigns names without replacement. Each format contributes its weight across its distinct combinations, including overlap with other formats. The sequence repeats after exhaustion. Materializing combinations keeps the implementation simple for the checked-in pools: 7,426 plain names and 7,047 fantasy names, each above the default 6,200 customers. Custom pack memory use grows with its combination count.

Customer name indices are assigned after observing orders, contiguously across emitted customers in market/customer order. This avoids early repetition from gaps left by non-ordering customers when a custom pool is small. Sparrow wording has a dedicated stream; IDs and all non-text fields match across themes. The Go parity test selects `fantasy_rpg` explicitly because its fixture groups by fantasy labels.

Validation: `mise run check` passed with 54 Rust tests, including seeded snapshots, malformed-path diagnostics before output, bundled/path byte identity, default-population uniqueness, sparse-customer exhaustion, theme and name-config invariance, statistical parity, Clippy, hooks, and versioning checks. The three Objective commits remain on `feat/themes` for the user's Worktrunk merge.

## Phase 5: SaaS accounts and revenue ✅

**Dependencies**: 3
**Requirements**: sp-R001, sp-R002, sp-R003, sp-R004, sp-R005, sp-R006, sp-R010, sp-R011, sp-R012, sp-R013, sp-R014, sp-R015, sp-R016, sp-R020, sp-R021, sp-R022, sp-R023, sp-R024, th-R006, th-R010, th-R013, cl-R041, op-R002, sm-R034, dev-R017, dev-R022, R001, R002, R004

Stands up the `saas` scenario with accounts, users, plans, subscriptions, MRR movements, and invoices. Accounts arrive through a simple arrival stage with `direct` attribution; Phase 11 replaces that stage with the marketing funnel without changing the account lifecycle.

### Scenario scaffold

- [x] Register `saas` in the scenario registry and wire `--scenario`
- [x] Declare the SaaS name kinds and label sets (organizations, plans, features, campaigns, industries, roles, regions) and add generators for them to `plain`
- [x] Implement staged generation: an account arrival stage by day, then one lifecycle unit per account

### Account lifecycle

- [x] Generate accounts, users, and seat growth scaled by employee band
- [x] Generate trials, conversion driven by user activation, plan and interval choice, and subscriptions
- [x] Generate expansion, contraction, involuntary churn from unpaid invoices, voluntary churn by tenure and engagement, and reactivation

### Revenue ledger

- [x] Derive MRR movements from subscription changes and classify each movement type
- [x] Generate invoices that tile each subscription's active period, with late and unpaid payments
- [x] Test the `sp-R010`–`sp-R016` invariants, MRR by date, and signup-cohort retention shape from the output files

### Docs

- [x] Add the SaaS entities to `docs/output-schema.md` and write `docs/saas-model.md` with the lifecycle model and example MRR and cohort SQL

The direct arrival stage uses 20 addressable accounts per scale unit across 1,460 days. Target-row calibration counts accounts and rejects targets beyond that population. Stage completion observes arrivals and counts each lifecycle to allocate contiguous person-name indices; generation then repeats the account-local simulation without retaining all accounts' dynamic rows. Scheduling remains unchanged for the parallel worker-pool Phase.

Trials emit users and begin paid subscriptions only on conversion, keeping subscription MRR equal to seats times plan pricing. Membership changes create immutable subscription intervals. Their invoices cover calendar cycles, clipped and prorated at interval and run boundaries; account-wide unpaid debt triggers churn after 30 days. The ledger and invoices ship with the lifecycle because payment outcomes drive its transitions; the revenue Objective adds independent file-level reconciliation and signup-cohort checks.

The plain theme expands to 126,242 people and 5,120 organizations, covering the default population without repetition. Its expanded person pool changes the seeded plain-name snapshot, including ecommerce names; IDs, dates, counts, and money remain invariant. Subscription status, employee band, and billing intervals remain scenario values. Usage and go-to-market metric assertions remain with their respective entity Phases.

Validation: `mise run check` passed all 87 Rust tests, Clippy, hooks, workflow task checks, version checks, the optimized build, and packaged-crate compilation. SaaS tests cover every primary and foreign key, activation and employee-band behavior, all five movement types, MRR/ARR reconciliation, calendar invoice tiling, late and unpaid payments, status, signup-cohort retention, theme invariance, generation-order independence, and byte identity across CSV, JSONL, Parquet, gzip, and zstd. Four Objective commits are ready for the user's Worktrunk merge.

## Phase 11: SaaS marketing and funnel ✅

**Dependencies**: 5
**Requirements**: gm-R001, gm-R002, gm-R003, gm-R004, gm-R010, gm-R011, gm-R012, gm-R013, gm-R014, gm-R020, gm-R021, gm-R022, gm-R023, gm-R040, sp-R001, sm-R034, dev-R022, R001, R004

Replaces Phase 5's direct arrival stage with campaigns, spend, touches, and leads that convert into accounts.

### Marketing

- [x] Generate campaigns per channel with budgets and flights, and daily `ad_spend` with impressions, clicks, and spend
- [x] Generate paid touches from clicks and organic, referral, and direct touches with steady growth
- [x] Give visitors multi-touch paths so first-touch and last-touch attribution disagree

### Funnel

- [x] Convert touches to leads by channel quality, and route leads to trials or demo requests by employee band
- [x] Feed converted leads into the account lifecycle as its arrival stage, setting `acquisition_channel` and `first_touch_id`
- [x] Test funnel monotonicity, lead-to-account tracing, and paid CAC per channel from the output files

### Docs

- [x] Document the funnel model with example attribution and CAC SQL

Paid channels run consecutive 90-day campaign flights with distinct CPC and conversion rates. Each paid click emits a touch; unpaid visitor volume grows toward twice its baseline, and 40% of visitors return through another channel. Leads progress by channel quality, with employee bands routing progressing leads toward self-serve trials or demo requests. Demo requests remain leads for Phase 12's sales pipeline. Converted leads create trial accounts the following day; the existing lifecycle controls paid conversion after 14 days.

Account slots are reserved from deterministic visitor decisions, but lifecycle generation reads only arrivals observed from emitted account rows. Lead names occupy a contiguous person-name sequence, and observed lead counts place users after that sequence to preserve theme uniqueness. Funnel-derived arrivals replace the fixed addressable population, so account calibration searches the supported calendar. Engine scheduling and output code remain outside this Phase.

Validation: `mise run check` passed all 97 Rust tests, Clippy, hooks, workflow and version checks, the optimized build, and packaged-crate compilation. File-level tests cover keys, campaign flights, click/touch reconciliation, attribution, funnel monotonicity, channel differences, monthly paid CAC, revenue metrics, theme invariance, person-name uniqueness, and byte identity across one and four workers in every format and compression mode. Both attribution and CAC documentation queries executed successfully in DuckDB against default-scale Parquet output. Seed 42 produced 10,292 leads and 1,845 trial accounts; 4,125 leads had different first-touch and last-touch channels. Three Objective commits are ready for the user's Worktrunk merge.
