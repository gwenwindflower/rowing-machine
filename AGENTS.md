# Rowing Machine

Synthetic data generator for Queria, a retro RPG-inspired SQL training app. Simulates the Arcanum Collective — a mage guild running 6 guild halls across fantasy towns — producing relational data across 7 CSV files: stores, customers, orders, items, products, supplies, and sparrows. Behavioral personas, seasonality curves, and market penetration dynamics make the output realistic enough to teach SQL on.

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
    inventory.go                15 products (5 weapons, 5 armor, 5 elixirs), RandomItems()
    stock.go                    Reagent supplies, DenormalizedSupplyRows() with origin_region
    names.go                    Fantasy name pool (JSON-sourced) for deterministic generation
  market/
    market.go                   Market struct, customer pool, penetration curve, SimDay()
  models/
    customer.go                 Customer struct + Persona interface + 6 implementations
    order.go                    Order struct, NewOrder() with tax calc
    sparrow.go                  Sparrow struct, NewSparrow() with guild_rank templates
    item.go                     Item struct, ItemType enum (Weapon/Armor/Elixir)
    supply.go                   Supply struct with origin_region, volatile flag
    store.go                    Store struct, StoreConfigs() (6 guild halls), PBuy/IsOpen methods
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
  static-data.md                Guild hall configs, product catalog, reagents, persona mix
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

Orders/items/sparrows collected during simulation, written at end via buffered CSV writers. Customers tracked in a map for deduplication. Products and supplies are static — written once from catalog.

## Domain Model

### Guild halls (stores)

6 locations: Thornwall, Misthollow, Ironvale, Starfen, Duskmarsh, Sunspire.

### Products

15 items across 3 categories, prefixed by type: `WEP-*` (weapons), `ARM-*` (armor), `ELX-*` (elixirs). Each product has a `power_level` (common/uncommon/rare/epic/legendary).

### Customers

Guild members with a `guild_rank` (initiate/journeyman/adept/master). Names are full fantasy names drawn from a JSON pool — not first+last combinations.

### Supplies (reagents)

Each supply has an `origin_region` column and a `volatile` flag (replaces perishable).

### Sparrows (magical message birds)

Replace tweets. Sparrows are sent by customers based on guild rank, using rank-appropriate message templates.

### Personas

6 behavioral personas: Courier, Artificer, FeastReveler, Apprentice, Wanderer, Herbalist.

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
