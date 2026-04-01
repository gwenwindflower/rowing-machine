# Rowing Machine

Synthetic data generator for the Rowing Outfitters SQL trainer. Simulates a chain of 6 fictional stores producing relational data across 7 CSV files: customers, orders, items, stores, products, supplies, and tweets. Behavioral personas, seasonality curves, and market penetration dynamics make the output realistic enough to teach SQL on.

## Commands

```bash
go build ./cmd/rowing-machine          # build
go run ./cmd/rowing-machine             # run
go test ./...                           # all tests
go test ./internal/simulation/ -run X   # single package/test
go vet ./...                            # static analysis
```

## Project Structure

```text
cmd/rowing-machine/
  main.go                       CLI entry (Cobra), flag parsing, seed handling
internal/
  catalog/
    inventory.go                10 menu items (5 jaffles, 5 beverages), RandomItems()
    stock.go                    29 supplies, DenormalizedSupplyRows() (65 rows)
    names.go                    ~510 first/last names for deterministic generation
  market/
    market.go                   Market struct, customer pool, penetration curve, SimDay()
  models/
    customer.go                 Customer struct + Persona interface + 6 implementations
    order.go                    Order struct, NewOrder() with tax calc
    tweet.go                    Tweet struct, NewTweet() with fan_level templates
    item.go                     Item struct, ItemType enum (Jaffle/Beverage)
    supply.go                   Supply struct
    store.go                    Store struct, StoreConfigs() (6 stores), PBuy/IsOpen methods
    season.go                   Season enum (Winter/Spring/Summer/Fall)
    uuid.go                     UUIDFromRNG() deterministic v4 UUID, FormatUUID()
  simulation/
    simulation.go               Config struct, Run() orchestrator, row conversion helpers
    day.go                      SeasonFromDate(), DayState struct, PrecomputeDays()
    curves.go                   AnnualCurve, WeekendCurve, GrowthCurve
    progress.go                 Progress bar (respects --quiet)
    integration_test.go         E2E, determinism, referential integrity, arithmetic checks
  output/
    writer.go                   OutputWriter interface
    csv.go                      CSVWriter with lazy file creation, 7 CSV files
docs/
  simulation.md                 Formulas, curves, order generation flow
  static-data.md                Store configs, menu items, supplies, persona mix
  output-schema.md              CSV column schemas for all 7 output files
```

## Architecture

### Money as int64 cents

All monetary values stored as `int64` cents. Tax: `int64(math.Round(float64(subtotal) * taxRate))`. Never use float64 for money storage.

### Deterministic PRNG

All randomness from `math/rand/v2`. Single `--seed` controls everything. Each market gets its own PRNG: `rand.New(rand.NewPCG(seed+uint64(marketIndex)+1, 0))`. Store UUIDs use a separate store RNG seeded with just `seed`. Same seed = byte-identical output.

### Import cycle avoidance

`market` defines its own `DayInfo` struct mirroring `simulation.DayState`. The orchestrator converts between them. This avoids `simulation` -> `market` -> `simulation` cycles.

### Pre-computed day effects

All day effects (annual * weekend * growth) computed into `[]DayState` at startup. No per-day allocation in the hot loop.

### Streaming output

Orders/items/tweets collected during simulation, written at end via buffered CSV writers. Customers tracked in a map for deduplication. Products and supplies are static — written once from catalog.

## Conventions

- Module path: `rowing-machine` (match go.mod)
- Package names: lowercase, single word
- Errors: return `error`, wrap with `fmt.Errorf("context: %w", err)`
- Tests: table-driven, in `_test.go` files alongside source
- No `init()` functions — explicit initialization
- Enums: `type X int` with `iota` constants and `String()` method

## CLI Flags (implemented)

| Flag | Type | Default | Notes |
| --- | --- | --- | --- |
| `--years` | int | 3 | Simulation duration (365 days each) |
| `--scale` | int | 100 | Customer pool multiplier |
| `--seed` | int64 | 0 | 0 = random (prints chosen seed) |
| `--start-date` | string | 2018-09-01 | Simulation epoch |
| `--output-dir` | string | ./factory-output | Output directory |
| `--pre` | string | raw | Filename prefix |
| `--quiet` | bool | false | Suppress progress |

Detailed reference docs (simulation formulas, static data, output schemas) in `docs/`.
