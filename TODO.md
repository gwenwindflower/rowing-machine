# Rowing Machine — TODO

Phases are scoped to one Manager-session worth of work each. Numbers are stable IDs, not execution order — `**Dependencies**:` lines (when present) drive sequencing.

## Phase 6: Conference dataset hardening

**Requirements**: cl-R001, cl-R004, cl-R012, sm-R007, dt-R007, dt-R009, dt-R010, op-R013, op-R017, op-R018, R004

### Current default dataset

- [x] Set the default simulation window to 2023–2026 so a no-argument run produces current conference data and includes every guild hall
- [x] Reject non-positive `--years` and `--scale` values before creating output
- [x] Cover defaults and validation through the CLI boundary

### Clean relational output

- [ ] Reject output rows whose field count differs from the entity schema
- [ ] Verify every generated primary key is non-empty and unique, every persisted field is non-empty, and supplies use the `(id, sku)` composite key
- [ ] Ensure each product type covers every power level exactly once
- [ ] Pin the current 41-supply roster and 92-row denormalized output
- [ ] Update user-facing schema and quick-start documentation for the current defaults and supply key

## Phase 1: Flexible output controls

**Requirements**: cl-R020, cl-R021, cl-R022, op-R020, op-R021, op-R022, op-R023

### `--target-rows` calibration mode

- [ ] Design and implement the pre-calculation flow that samples output density at the chosen scale and derives a `--years` value to hit the target `orders` row count
- [ ] Wire the `--target-rows` flag and mutual-exclusion with `--years` (error on both)
- [ ] Surface a pre-calculation progress indicator distinct from the main generation progress; switch over once calibration completes
- [ ] Cover via integration test: target → resulting row count within an acceptable tolerance

### JSONL output format

- [ ] Add JSONL writer behind the `OutputWriter` interface
- [ ] Native JSON types for numbers and booleans (no `"True"`/`"False"` string-wrapping in JSONL)
- [ ] Wire `--format jsonl`
- [ ] Integration test: byte-identical JSONL across two runs with the same seed

### Parquet output format

- [ ] Add Parquet writer behind the `OutputWriter` interface, using a maintained Go Parquet library (evaluate `parquet-go` vs. `apache/arrow-go`)
- [ ] Auto-tune row group size from estimated row count; expose the tuning knob as a code-level constant for later flag-based override
- [ ] Map types: int64 cents → int64; timestamps → TIMESTAMP_MICROS UTC; UUIDs → string for now (revisit binary later)
- [ ] Wire `--format parquet`
- [ ] Integration test: byte-identical Parquet across two runs with the same seed

### Compression

- [ ] Add `--compress` flag; route to gzip for JSONL, zstd for Parquet
- [ ] Error clearly when `--compress` is combined with `--format csv`
- [ ] File extensions reflect compression: `.jsonl.gz`, `.parquet.zst`

## Phase 2: Worker-parallel simulation

**Dependencies**: 1
**Requirements**: cl-R030, R001 (re-affirmed under parallelism)

Refactor the simulation hot path so per-day work is reproducible from `(seed, day_index)` alone — no dependence on prior days' RNG state — then fan out across worker goroutines. Phase 1 ships first so the output writers can absorb concurrent-producer patterns cleanly.

### Audit and refactor for day-independence

- [ ] Audit each subsystem that touches the per-day RNG (markets, personas, sparrows) for cross-day state
- [ ] Refactor each finding into pure-per-day generation seeded by `(seed, day_index, market_index)`
- [ ] Add unit tests pinning the new per-day determinism contract
- [ ] Update `sm-simulation.md` with `sm-R030+` requirements capturing the parallelism contract

### Worker fan-out

- [ ] Implement worker pool that pulls day-index slices from a shared queue
- [ ] Aggregate per-worker output into the shared writers without violating dedup or order constraints
- [ ] Wire `--workers <int>` (default 1, 0 also treated as 1)
- [ ] Integration test: serial vs. `--workers 4` MUST byte-match for the same seed

## Phase 3: Theming system

**Dependencies**: 1
**Requirements**: cl-R040

Refactor the catalog and output vocabulary as theme-driven. The current fantasy roster becomes one bundled theme; the baseline becomes plain ecommerce. Phase 1 ships first so themes don't have to track multiple in-flight format changes simultaneously.

### Theme contract

- [ ] Design the TOML schema for table/column mappings and value vocabulary swaps
- [ ] Add a `th-themes.md` spec capturing the theme contract and bundled theme rules

### Theme loader and registry

- [ ] Implement theme parsing and validation
- [ ] Register `default` (plain ecommerce) and `fantasy_rpg` (current data) as bundled themes
- [ ] Refactor `internal/catalog/` and writers to consume the active theme rather than hardcoded fantasy data
- [ ] Wire `--theme <name>` (default `default` once the plain ecom theme is in place)
- [ ] Update `dt-catalog.md` requirements: the fantasy roster becomes a theme example, not the contract

## Phase 4: Messy mode

**Dependencies**: 3
**Requirements**: cl-R050

Inject realistic mess: bad formatting, inconsistent column patches, negative amounts for absolute-value columns, etc. Phase 3 ships first so the mess-injection logic can plug into the theme pipeline cleanly. Parameters need tuning — expect this Phase to grow Tasks during execution.

### Mess catalog

- [ ] Enumerate the categories of mess: format violations, value anomalies, schema drift, missing values, encoding glitches
- [ ] Pick a flag shape: single boolean vs. graded levels (`--messy=light|medium|heavy`) vs. independent toggles
- [ ] Refine `cl-R050` based on the chosen shape; flow back to Planner

### Implementation pass 1

- [ ] Implement chosen mess categories with seed-pinned determinism
- [ ] Integration test: byte-identical output across runs with the same seed + same mess setting

## Phase 5: Expanded schema entities

**Dependencies**: 3

New entities to consider: payments, promotions, staff, loyalty program, social-source reviews, event logs. Requires planning before execution — the exact roster, schemas, and FK relationships need to be mapped first. This Phase is a placeholder; promote to active when the planning lands.

- [ ] Planner pass: enumerate entity roster, draft schemas, identify FK relationships and theme implications
- [ ] Split into per-entity Objectives once the roster is settled; this Phase may split into multiple Phases

## Backlog

- **TUI form (Charm / Bubble Tea)** — once the flag surface gets dense enough that picking a configuration from CLI args becomes painful, launch a TUI that lays out the options and produces an invocation. Defer until Phases 1–4 have shaken out the actual flag surface.
- **Verbose-for-agents mode** — a `--verbose` or `--explain` mode that narrates the simulation as it runs, in a style optimized for agents reading CLI output rather than humans (per the Supermodel Labs DX bar in `~/.claude/supermodellabs.md`).
- **Distribution wiring** — Homebrew tap entry, Linux package manifests, GitHub Actions release workflow. Hold until the binary stabilizes through Phase 2.
