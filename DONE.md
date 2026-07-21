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
