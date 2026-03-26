# Rowing Machine - Synthetic Data Generator

Go rewrite of the original Python jaffle-shop-generator, with a new spin. Instead of a restaurant, it simulates a chain of outdoors supply stores called Rowing Outfitters, producing synthetic relational data (customers, orders, items, stores, products, supplies, tweets).

Full spec: @MIGRATION.md. Static data and simulation formulas: @.claude/rules/static-data.md, @.claude/rules/simulation.md.

## Commands

```bash
go build -o rowing-machine .                    # build
go run .                                # run (once CLI is wired up)
go test ./...                           # all tests
go test ./internal/simulation/ -run X   # single package/test
go vet ./...                            # static analysis
```

## Project Structure

```text
cmd/rowing-machine/main.go              CLI entry (cobra or similar)
internal/
  simulation/
    simulation.go                Orchestrator: creates markets, runs day loop, writes output
    day.go                       Day, Season, DayState, pre-computed effects
    curves.go                    AnnualCurve, WeekendCurve, GrowthCurve
  models/
    store.go                     Store struct
    customer.go                  Customer struct + Persona interface + 6 implementations
    order.go                     Order struct, tax calc
    tweet.go                     Tweet struct, content generation
    item.go                      Item struct, ItemType enum
    supply.go                    Supply struct
  market/
    market.go                    Market: customer pool, penetration curve, daily sim
  catalog/
    inventory.go                 Menu items (static data, 10 items)
    stock.go                     Supplies (static data, 29 items)
    names.go                     First/last name pools for deterministic name generation
  output/
    writer.go                    OutputWriter interface
    csv.go                       CSV implementation
```

## Architecture Decisions

### Money as int64 cents

All monetary values stored as `int64` cents. Menu prices defined in cents. Tax computed with
`math.Round`. Never use float64 for money storage.

### Deterministic PRNG

All randomness from `math/rand/v2`. Single `--seed` controls everything.
Each market gets its own PRNG: `rand.New(rand.NewPCG(seed+uint64(marketIndex), 0))`.
UUIDs generated from market PRNG, not `crypto/rand`. Same seed = byte-identical output.

### Parallel markets, deterministic merge

Markets simulate independently via goroutines. Results merged in fixed index order (0-5).
No shared mutable state between goroutines — catalog and day effects are read-only.

### Pre-computed day effects

Compute all day effects (annual *weekend* growth) into a `[]float64` slice at startup.
No per-day object allocation in the hot loop.

### Streaming output

Write orders/items/tweets as generated (buffered writers). Track seen customers in a map,
write customers CSV at end. Products and supplies are static — write once.

## Conventions

- Module path: `rowing-machine` (match go.mod)
- Package names: lowercase, single word where possible
- Errors: return `error`, wrap with `fmt.Errorf("context: %w", err)`
- Tests: table-driven, in `_test.go` files alongside source
- No `init()` functions — explicit initialization
- Enums: `type ItemType int` with `iota` constants and `String()` method

## Python Bugs Fixed in This Rewrite

1. **WeekendCurve**: Python always returned 1.0. Go returns 0.6 on weekends.
2. **Commuter order time**: Python used N(60, 30) = 1 AM. Go uses N(450, 30) = 7:30 AM.
3. **Market penetration**: Python had discontinuity at day 7. Go uses single smooth curve.
4. **total_minutes**: Python used `second * 60 + minute`. Go uses `hour * 60 + minute`.
5. **Non-deterministic output**: Python had no seed control. Go has `--seed` flag.

## Output Schema

Seven CSV files: `{prefix}_{entity}.csv` in `{output_dir}/` (default `./factory-output/`).
Default prefix: `raw`. Monetary columns are integer cents. Timestamps are ISO 8601.
See MIGRATION.md Section 13 for column-level detail.

## CLI Flags

| Flag | Type | Default | Notes |
| --- | --- | --- | --- |
| `--years` | int | 3 | Mutually exclusive with --target-rows |
| `--target-rows` | int | — | All stores open day 0 |
| `--scale` | int | 100 | Customer pool multiplier |
| `--start-date` | string | 2018-09-01 | Epoch |
| `--seed` | int64 | 0 (random) | 0 = random, print chosen seed |
| `--format` | enum | csv | csv, jsonl, parquet |
| `--messy` | bool | false | Inject data quality issues |
| `--output-dir` | string | ./factory-output | Output directory |
| `--pre` | string | raw | Filename prefix |
| `--compress` | bool | false | Gzip output |
| `--workers` | int | NumCPU() | Parallel workers |
| `--quiet` | bool | false | Suppress progress |
