# Rowing Machine TODO

Phase numbers are stable IDs, not order; `**Dependencies**:` lines drive sequencing. `docs/architecture.md` shows the build lanes and which Phases can run as parallel sessions.

## Phase 1: Flexible output controls

**Dependencies**: 9
**Requirements**: cl-R020, cl-R021, cl-R022, cl-R023, op-R001, op-R020, op-R021, op-R022, op-R023, R001

Owns `src/output/` and the calibration path in `src/engine/`. Can run in parallel with Phase 3.

### JSONL output

- [ ] Add the JSONL writer with native numbers, booleans, and nulls, and wire `--format jsonl`
- [ ] Test byte-identical JSONL across two runs with the same seed

### Parquet output

- [ ] Add the Parquet writer with `arrow` and `parquet`, mapping cents to int64 and timestamps to `TIMESTAMP_MICROS` UTC
- [ ] Derive row group size from estimated row count through one named constant, and wire `--format parquet`
- [ ] Test byte-identical Parquet across two runs, and read a file back to check types

### Compression

- [ ] Add `--compress`: gzip for JSONL (`.jsonl.gz`), zstd column compression for Parquet
- [ ] Reject `--compress` with CSV, suggesting `jsonl` or `parquet`

### Target-row calibration

- [ ] Estimate the duration that yields `--target-rows` rows of the scenario's calibration entity by sampling a short run
- [ ] Show a calibration indicator distinct from generation progress, and reject `--target-rows` with `--years`
- [ ] Test that a calibrated run lands within a stated tolerance of the target

## Phase 2: Worker-parallel generation

**Dependencies**: 1
**Requirements**: cl-R030, sm-R030, sm-R033, R001, dev-R016, dev-R020

The engine is already unit-pure from Phase 9; this Phase adds the scheduler. Can run alongside the SaaS Phases, which never touch scheduling.

### Parallel scheduler

- [ ] Generate work units on a `rayon` pool and reorder finished units so the sink receives them in declared unit order with bounded memory
- [ ] Wire `--workers`, defaulting to available cores and rejecting `0`
- [ ] Test that `--workers 1` and `--workers 8` byte-match for every scenario and format

### Throughput

- [ ] Extend `mise run bench` to worker counts 1 and all cores, and record the results in `docs/performance.md`

## Phase 3: Theming system and native names

**Dependencies**: 9
**Requirements**: th-R001, th-R002, th-R003, th-R004, th-R005, th-R006, th-R007, th-R008, th-R009, th-R010, th-R011, th-R012, th-R013, th-R014, th-R015, th-R016, th-R017, th-R018, cl-R040, cl-R042, R001

Owns `src/theme/`, `themes/`, and swapping hardcoded names in `src/scenario/ecommerce/` for theme lookups. Can run in parallel with Phase 1. Themes only generate names and labels; scenarios keep every number.

### Theme contract and loader

- [ ] Define the theme TOML schema: name, description, a name generator per name kind, and a value list per label set
- [ ] Let each scenario declare the name kinds and label sets it needs, and check theme compatibility against the selected scenario
- [ ] Parse and validate themes with `toml` and `serde`, rejecting bad files before simulation with the file and field named
- [ ] Compile bundled themes into the binary and load path themes through `--theme`

### Native name generation

- [ ] Expand weighted name formats over whole-token component pools, per name kind
- [ ] Map entity index to a unique combination with a seeded bijective permutation, and define reuse after exhaustion
- [ ] Test traceability, exhaustion, run-wide uniqueness, and that name config changes leave every other field unchanged

### Bundled themes

- [ ] Move ecommerce names and labels (guild halls, products, product types, power levels, ranks, sparrow vocabulary) into `themes/fantasy_rpg.toml`, leaving numbers in Rust, and write `themes/plain.toml`
- [ ] Review and check in component pools for both themes, sized for the default population
- [ ] Add `rowing-machine themes` and make `plain` the default
- [ ] Pin a seeded name snapshot per theme and update `docs/static-data.md`

## Phase 5: SaaS accounts and revenue

**Dependencies**: 3
**Requirements**: sp-R001, sp-R002, sp-R003, sp-R004, sp-R005, sp-R006, sp-R010, sp-R011, sp-R012, sp-R013, sp-R014, sp-R015, sp-R016, sp-R020, sp-R021, sp-R022, sp-R023, sp-R024, th-R006, th-R010, th-R013, cl-R041, op-R002, sm-R034, dev-R017, dev-R022, R001, R002, R004

Stands up the `saas` scenario with accounts, users, plans, subscriptions, MRR movements, and invoices. Accounts arrive through a simple arrival stage with `direct` attribution; Phase 11 replaces that stage with the marketing funnel without changing the account lifecycle.

### Scenario scaffold

- [ ] Register `saas` in the scenario registry and wire `--scenario`
- [ ] Declare the SaaS name kinds and label sets (organizations, plans, features, campaigns, industries, roles, regions) and add generators for them to `plain`
- [ ] Implement staged generation: an account arrival stage by day, then one lifecycle unit per account

### Account lifecycle

- [ ] Generate accounts, users, and seat growth scaled by employee band
- [ ] Generate trials, conversion driven by user activation, plan and interval choice, and subscriptions
- [ ] Generate expansion, contraction, involuntary churn from unpaid invoices, voluntary churn by tenure and engagement, and reactivation

### Revenue ledger

- [ ] Derive MRR movements from subscription changes and classify each movement type
- [ ] Generate invoices that tile each subscription's active period, with late and unpaid payments
- [ ] Test the `sp-R010`–`sp-R016` invariants, MRR by date, and signup-cohort retention shape from the output files

### Docs

- [ ] Add the SaaS entities to `docs/output-schema.md` and write `docs/saas-model.md` with the lifecycle model and example MRR and cohort SQL

## Phase 10: SaaS product usage

**Dependencies**: 5
**Requirements**: sp-R007, sp-R008, sp-R030, sp-R031, sp-R032, sp-R033, sp-R034, dev-R022, R001, R004

Adds sessions and events, the highest-volume SaaS entities. Can run in parallel with Phase 11; both register entities in `src/scenario/saas/mod.rs`, so fold the second one with care.

### Sessions

- [ ] Generate sessions per user on a work-week rhythm in the account's region, with holiday dips
- [ ] Model onboarding decay to a personal rate and the pre-churn fade

### Events

- [ ] Generate events inside each session from the theme's feature catalog, varying adoption by tier and role
- [ ] Tie activation events to `users.activated_at`
- [ ] Test session and event bounds, engagement-to-churn correlation, and volume scaling with `--scale`

### Docs and performance

- [ ] Add usage entities and example engagement SQL to the SaaS docs, and benchmark the scenario at default scale

## Phase 11: SaaS marketing and funnel

**Dependencies**: 5
**Requirements**: gm-R001, gm-R002, gm-R003, gm-R004, gm-R010, gm-R011, gm-R012, gm-R013, gm-R014, gm-R020, gm-R021, gm-R022, gm-R023, gm-R040, sp-R001, sm-R034, dev-R022, R001, R004

Replaces Phase 5's direct arrival stage with campaigns, spend, touches, and leads that convert into accounts.

### Marketing

- [ ] Generate campaigns per channel with budgets and flights, and daily `ad_spend` with impressions, clicks, and spend
- [ ] Generate paid touches from clicks and organic, referral, and direct touches with steady growth
- [ ] Give visitors multi-touch paths so first-touch and last-touch attribution disagree

### Funnel

- [ ] Convert touches to leads by channel quality, and route leads to trials or demo requests by employee band
- [ ] Feed converted leads into the account lifecycle as its arrival stage, setting `acquisition_channel` and `first_touch_id`
- [ ] Test funnel monotonicity, lead-to-account tracing, and paid CAC per channel from the output files

### Docs

- [ ] Document the funnel model with example attribution and CAC SQL

## Phase 12: SaaS sales pipeline

**Dependencies**: 11
**Requirements**: gm-R005, gm-R006, gm-R007, gm-R008, gm-R030, gm-R031, gm-R032, gm-R033, gm-R034, gm-R035, gm-R041, gm-R042, dev-R022, R001, R004

### Sales team and opportunities

- [ ] Generate the rep roster by segment with hiring, departures, ramp, and annual cost
- [ ] Generate opportunities from demo leads with stage progression, band-driven cycle length and amount, and quarter-end close pressure
- [ ] Generate sales activities within each opportunity's open window

### Closing the loop

- [ ] Start the account's first paid subscription at each won opportunity's close, matching amount to ARR
- [ ] Test stage order, owner employment, activity windows, win rate, and blended CAC and payback bounds

### Docs

- [ ] Document the sales model with example pipeline, win-rate, and blended CAC SQL

## Phase 14: Release and crates.io plumbing

**Requirements**: dev-R023, dev-R024, dev-R025, dev-R026, dev-R027, dev-R028, dev-R030, R003

Brings the release pipeline up to the heraldr pattern (`~/dev/herdr/heraldr`): crates.io publishing, asset recovery, and the everyday dev and dependency tasks. Port from heraldr, keeping Homebrew, which heraldr does not use. Generic template tasks (labels, rulesets, publishing, recovery, versioning) are tested in the `_tool` template, not here; this project tests only its own task config. Touches `mise.toml`, `mise-tasks/`, `tests/*.sh`, `.github/workflows/`, and `Cargo.toml` package metadata, so it can run alongside Phases 1 and 3; expect a small `Cargo.toml` rebase.

### Crate publishing

- [x] Add `include`, `keywords`, and `categories` to `Cargo.toml`, and a `test:crate` task running `cargo package --locked --allow-dirty`
- [x] Port `release:crate-preflight`, `release:publish-crate`, and the confirmed `release:bootstrap-crate` task
- [x] Add the `crate` job to `release-build.yml` behind `CRATES_IO_PUBLISHING`, in the `release` environment with `id-token: write` and `rust-lang/crates-io-auth-action`, after the asset upload

### Release recovery and test tasks

- [ ] Port `release:recover-assets`
- [ ] Port heraldr's task-workflow test as `test:workflows`, checking that `check` and CI never select `dev:` or other interactive tasks
- [ ] Remove `tests/versioning.sh` and `test:versioning`, which test template-generic tasks
- [ ] Add `test:build` and an aggregate `test` task

### Everyday tasks and CI hygiene

- [ ] Add `dev:build` and `dev:test` (`cargo pretty`, `raw = true`) and `deps:check`, `deps:update`, and `deps:audit`, keeping every `dev:` task out of `check`
- [ ] Match heraldr's CI and hook settings: `MISE_TASK_OUTPUT` and `MISE_JOBS` in `ci.yml`, `GH_REPO` on the asset upload, `default_stages` in `prek.toml`, zizmor cache `allow_write`, and `jq` in `[tools]`
- [ ] Order the README install section: Homebrew, `cargo binstall`, `cargo install --locked`, release archive

## Phase 13: Go retirement and first Rust release

**Dependencies**: 1, 2, 3, 14
**Requirements**: R003, dev-R001, dev-R008, dev-R009, dev-R012, dev-R021, dev-R025, dev-R026

### Go retirement

- [ ] Confirm the parity test passes, then delete `go-reference/` and the Go ignore rules
- [ ] Drop Go references from `docs/architecture.md` and keep the parity fixture as a regression baseline

### Repository provisioning

- [ ] Run `mise run ci-audit:pinact` to refresh action pins and `mise run ci-audit`
- [x] Run `mise run repo:settings --homebrew`, `mise run repo:labels`, and `mise run repo:environments`
- [ ] Open a throwaway PR with a deliberate lint failure and confirm the annotation lands on the diff
- [ ] #user Confirm CONTRIBUTING and SECURITY resolve from the owner's `.github` repository
- [ ] #user Push `main` and run `mise run repo:rulesets` once CI reports on it

### Release

- [ ] Run `mise run release:rehearse`, resolve what it reports, and delete `docs/bootstrap.md`
- [ ] #user Create the Homebrew tap token secret, cut the release with `mise run release`, then run `mise run release:verify`
- [ ] #user Publish the first crate with `mise run release:bootstrap-crate`
- [ ] #user Add a crates.io trusted publisher for `release-build.yml` in the `release` environment, then set `CRATES_IO_PUBLISHING=true`
