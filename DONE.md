# Rowing Machine — DONE

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
