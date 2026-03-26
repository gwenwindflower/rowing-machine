# Phase 1: Core MVP

Get from zero to a working CLI that generates all 7 CSV files with correct simulation logic,
deterministic output, and bug fixes applied. No messy mode, no JSONL/parquet, no expanded schema.

## Milestone: "It runs and produces correct CSVs"

---

## Task List

### 1. Project scaffolding

- [ ] Move `main.go` to `cmd/rowing-machine/main.go` (standard Go project layout)
- [ ] Create `internal/` package directories: `simulation`, `models`, `market`, `catalog`, `output`
- [ ] Add CLI framework (Cobra recommended — already common in Go CLIs)
- [ ] Wire up flags: `--years`, `--scale`, `--seed`, `--output-dir`, `--pre`, `--quiet`
- [ ] Defer `--target-rows`, `--format`, `--messy`, `--compress`, `--workers` to later phases
- [ ] `--seed 0` picks a random seed and prints it; nonzero uses the given seed

### 2. Static data / catalog

- [ ] `internal/catalog/inventory.go` — 10 menu items as Go structs, prices in cents
- [ ] `internal/catalog/stock.go` — 29 supplies with SKU associations
- [ ] `internal/catalog/names.go` — first name + last name pools (~500 each)
- [ ] `internal/models/item.go` — `Item` struct, `ItemType` enum (`Jaffle`, `Beverage`)
- [ ] `internal/models/supply.go` — `Supply` struct
- [ ] Unit tests: verify inventory counts, supply-SKU mappings, item type filtering

### 3. Temporal system

- [ ] `internal/simulation/day.go` — `Season` enum, `DayState` struct, `season_from_date()`
- [ ] `internal/simulation/curves.go` — `AnnualCurve`, `WeekendCurve`, `GrowthCurve`
- [ ] Pre-compute `[]DayState` for full simulation at startup
- [ ] Hours of operation: weekday 07:00-20:00, weekend 08:00-15:00
- [ ] Unit tests: verify season boundaries, curve output ranges, weekend detection
- [ ] Verify: WeekendCurve returns 0.6 on weekends (Python bug #1 fixed)

### 4. Store model

- [ ] `internal/models/store.go` — `Store` struct (id, name, popularity, tax_rate, opened_day)
- [ ] Store config array (6 stores with hardcoded values from static-data reference)
- [ ] Methods: `PBuy(dayEffect)`, `IsOpen(dayIndex)`, `IsOpenAt(minute, isWeekend)`, `DaysSinceOpen(dayIndex)`
- [ ] UUID generation from PRNG (not crypto/rand)
- [ ] Unit tests: verify tax calc, open/closed checks, p_buy ranges

### 5. Customer model + personas

- [ ] `internal/models/customer.go` — `Customer` struct, `Persona` interface
- [ ] Persona interface: `PBuyPersona(isWeekend, season, favNum)`, `PTweet()`, `OrderMinute(rng)`, `OrderItems(rng, inventory)`
- [ ] Implement all 6 personas: Commuter, RemoteWorker, BrunchCrowd, Student, Casuals, HealthNut
- [ ] Verify: Commuter uses N(450, 30) not N(60, 30) (Python bug #2 fixed)
- [ ] Name generation from deterministic pool using market PRNG
- [ ] Unit tests: verify persona probability ranges, item counts per persona, order time distributions

### 6. Order + Tweet models

- [ ] `internal/models/order.go` — `Order` struct, subtotal/tax/total computation in cents
- [ ] `internal/models/tweet.go` — `Tweet` struct, content generation with fan_level templates
- [ ] Monetary: `tax = int64(math.Round(float64(subtotal) * taxRate))`, `total = subtotal + tax`
- [ ] Tweet delay: 0-19 minutes from order time
- [ ] Unit tests: verify cent arithmetic, tweet content templates, items sentence formatting

### 7. Market simulation

- [ ] `internal/market/market.go` — `Market` struct with customer pool
- [ ] Market init: create customers per persona weights, shuffle with market PRNG
- [ ] Penetration curve: `min(ln(1 + pct*(e-1)), 1)` — single smooth curve (Python bug #4 fixed)
- [ ] `SimDay()`: activate customers, roll p_buy, generate orders, roll p_tweet, generate tweets
- [ ] Core loop: `p_buy = sqrt(p_buy_season * p_buy_persona)`
- [ ] Order generation: sample minute → check hours → select items → create order
- [ ] Unit tests: verify penetration curve values, customer activation counts, order/tweet generation

### 8. Simulation orchestrator

- [ ] `internal/simulation/simulation.go` — ties everything together
- [ ] Create 6 stores and 6 markets with per-market PRNGs
- [ ] Pre-compute day effects
- [ ] Main loop: iterate days, iterate markets, call `SimDay()`, collect results
- [ ] Single-threaded first (defer goroutine parallelism to optimization pass)
- [ ] Track seen customers (map by UUID) for deduplication in output

### 9. CSV output

- [ ] `internal/output/csv.go` — write 7 CSV files with buffered writers
- [ ] File naming: `{prefix}_{entity}.csv` in `{output_dir}/`
- [ ] Create output directory if missing
- [ ] Column schemas match MIGRATION.md Section 13 exactly:
  - `stores.csv`: id, name, opened_at, tax_rate
  - `customers.csv`: id, name (only customers who placed orders)
  - `orders.csv`: id, customer, ordered_at, store_id, subtotal, tax_paid, order_total (cents)
  - `items.csv`: id, order_id, sku (one row per item in each order)
  - `products.csv`: sku, name, type, price, description
  - `supplies.csv`: id, name, cost, perishable, sku (denormalized)
  - `tweets.csv`: id, user_id, tweeted_at, content
- [ ] Timestamps: ISO 8601 (`2006-01-02T15:04:05` in Go format)

### 10. Integration + determinism verification

- [ ] End-to-end test: run with `--seed 42 --years 1 --scale 10`, verify non-empty output
- [ ] Determinism test: run twice with same seed, diff output — must be byte-identical
- [ ] Sanity checks on output:
  - All customer IDs in orders.csv exist in customers.csv
  - All store IDs in orders.csv exist in stores.csv
  - Order totals = subtotal + tax_paid
  - No orders before store opening dates
  - No orders outside operating hours
  - Item SKUs all exist in products.csv
- [ ] Row count smoke test: 1-year scale-10 should produce reasonable order counts

### 11. Progress display

- [ ] Progress bar or spinner during simulation (respect `--quiet`)
- [ ] Print seed value at start (so user can reproduce)
- [ ] Print summary at end: row counts per entity, output directory, elapsed time

---

## Out of Scope for Phase 1

These are documented in MIGRATION.md but deferred to later phases:

- `--target-rows` mode (auto-calculate simulation duration)
- `--format jsonl` / `--format parquet` output
- `--messy` mode (data quality injection)
- `--compress` (gzip output)
- Goroutine parallelism (`--workers`) — build single-threaded first, optimize later
- Expanded schema entities (payments, promotions, staff, loyalty, etc.)
- TUI form (Charm/bubbletea) — CLI flags are sufficient for MVP

## Definition of Done

1. `go build` succeeds with no warnings
2. `go test ./...` passes
3. `go vet ./...` clean
4. Running `rowing-machine --seed 42 --years 1` produces 7 CSV files in `./factory-output/`
5. Running the same command twice produces byte-identical output
6. Output passes all sanity checks from task 10
7. All 5 Python bugs are fixed (weekend curve, commuter time, penetration curve, total_minutes, determinism)
