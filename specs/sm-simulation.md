# Simulation Engine — Spec

Owns the math, RNG discipline, and per-day generation flow that turn static catalog data into rows. Lives in `internal/simulation/`, `internal/market/`, and the persona logic in `internal/models/customer.go`.

## Vocabulary

- **`p_buy`** — probability that a given customer purchases on a given day. Combines store seasonality and persona disposition.
- **`day_effect`** — pre-computed scalar applied to a store's base popularity: `annual × weekend × growth`.
- **Penetration** — fraction of a store's total addressable market (TAM) that has been reached, ramps up over the first year a store is open.
- **Day state** — pre-computed `{annual, weekend, growth, season}` tuple for each simulated day.

## Requirements

### Probability formulas

- **sm-R001 Purchase probability.** Per-customer-per-day purchase probability is `p_buy = sqrt(p_buy_season * p_buy_persona)` where `p_buy_season = store.base_popularity * day_effect`. The square root smooths the combined probability into the 0–1 range used by the persona roll.
- **sm-R002 Day effect composition.** `day_effect = annual_curve(day) * weekend_curve(day) * growth_curve(day)`. Multiplicative, not additive — each curve is a multiplier on the others.
- **sm-R003 Annual curve.** `annual_curve(day) = (cos(x) + 1) / 10 + 0.8` where `x` is day-of-year mapped to `[0, 2π]`. Range: `[0.8, 1.0]`. Peaks at year start, troughs mid-year.
- **sm-R004 Weekend curve.** Weekday returns `1.0`, weekend (Sat/Sun) returns `0.6`.
- **sm-R005 Growth curve.** `growth_curve(month_offset) = 1 + (month_offset / 12) * 0.2` where `month_offset = (year - 2016) * 12 + month`. Compounds slowly across the simulation duration.
- **sm-R006 Market penetration.** Per-store penetration ramps up over the first year open: `pct = min(days_since_open / 365, 1)`; `penetration = min(ln(1 + pct * (e - 1)), 1)`. Gives day 0 → 0%, day 30 → ~14%, day 180 → ~62%, day 365 → 100%.

### Calendar

- **sm-R007 Epoch.** Simulation day index 0 corresponds to `--start-date` (default `2023-01-01`).
- **sm-R008 Seasons.** Day-of-year ranges:
    - WINTER: Jan 1 – Mar 20, Dec 21 – Dec 31
    - SPRING: Mar 21 – Jun 20
    - SUMMER: Jun 21 – Sep 20
    - FALL: Sep 21 – Dec 20
- **sm-R009 Hours of operation.** Weekday: opens 07:00 (minute 420), closes 20:00 (minute 1200). Weekend: opens 08:00 (minute 480), closes 15:00 (minute 900). Orders sampled outside these windows are discarded.

### RNG discipline

- **sm-R010 PCG-based generation.** All randomness MUST flow through `math/rand/v2` PCG streams. No `math/rand` v1 calls.
- **sm-R011 Per-market seeding.** Each market gets its own PRNG: `rand.New(rand.NewPCG(seed + uint64(marketIndex) + 1, 0))`. The `+1` offset ensures market 0 doesn't collide with the bare-seed store RNG.
- **sm-R012 Store RNG isolation.** Store UUIDs use a separate PRNG seeded with the bare `--seed`. Decouples store ID stability from market simulation order.

### Day loop

- **sm-R013 Pre-computed day state.** All `(annual, weekend, growth, season)` tuples MUST be computed into a `[]DayState` slice before the simulation loop starts. The per-day hot loop allocates no curve state.
- **sm-R014 Import-cycle isolation.** `market` defines its own `DayInfo` struct mirroring `simulation.DayState`; the orchestrator converts between them. Prevents `simulation → market → simulation` cycles.

### Personas

- **sm-R015 Persona roster.** Six personas implement the `Persona` interface in `internal/models/customer.go`: Courier, Artificer, FeastReveler, Apprentice, Wanderer, Herbalist. Weights and disposition documented in `dt-catalog.md` (the catalog domain owns the *data*; this domain owns the *behavior contract*).
- **sm-R016 Order time sampling.** Each persona samples its order minute from its own normal distribution; samples below 0 clamp to 0. Samples that fall outside the store's hours-of-operation window are discarded (the customer doesn't order that day).
- **sm-R017 Item selection.** Each persona selects items per its own rules (item count distribution, type preference). Item count MUST be ≥ 1 when an order is produced.

### Order construction

- **sm-R018 Tax and total.** `tax_paid = int64(math.Round(float64(subtotal) * store.tax_rate))`. `order_total = subtotal + tax_paid`. All three are `int64` cents.
- **sm-R019 Order generation flow.**
    1. Roll `p_buy` — miss skips the customer for the day.
    2. Sample order minute from the persona's distribution.
    3. Check store hours — outside-window orders discarded.
    4. Select items per persona rules.
    5. Construct order with subtotal, tax, total.
    6. Roll `p_sparrow` — on hit, attach a sparrow with a 0–19 minute delay after the order.

### Sparrows

- **sm-R020 Sparrow timing.** `sent_at = ordered_at + uniform(0, 19)` minutes.
- **sm-R021 Sparrow templates.** Three message templates parametrized by `fan_level` (1–5):
    - `fan_level > 3`: positive adjective + items sentence
    - `fan_level < 3`: negative adjective + items sentence
    - `fan_level == 3`: neutral adjective
- **sm-R022 Sparrow rank gating.** Sparrows MUST use rank-appropriate vocabulary based on the sender's `guild_rank`.

### Customer dedup

- **sm-R023 Output gating.** Only customers who placed at least one order during the simulation MUST appear in the customer output. Tracked in a map during simulation; emitted at end.

## Out of scope for this domain

- File formatting and on-disk layout → `op-output.md`.
- Hardcoded store, product, and supply data → `dt-catalog.md`.
- CLI flag surface and validation → `cl-cli.md`.
- Future parallelism via `--workers` — adds the requirement that any per-day simulation step be reproducible from `(seed, day_index)` alone, without depending on prior days' RNG state. Will be added under `sm-R030+` when Phase 2 lands.
