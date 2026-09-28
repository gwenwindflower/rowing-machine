# Simulation engine and ecommerce scenario

## Goals

The engine turns a seed, a date range, and a scenario into rows. It owns the calendar, the shared curves, stream derivation, and the day loop that every scenario runs on. This spec also holds the ecommerce scenario's behavior: purchase probability, personas, orders, and sparrows. Catalog data lives in `dt-catalog.md`, SaaS behavior in `sp-saas-product.md` and `gm-go-to-market.md`.

## Vocabulary

- **`p_buy`** — the probability that a customer buys on a given day, combining store seasonality and persona disposition.
- **`day_effect`** — the pre-computed multiplier on a store's base popularity: `annual × weekend × growth`.
- **Penetration** — the fraction of a store's addressable market reached, ramping over the store's first year.

## Requirements

### Probability formulas

- **sm-R001** — Per-customer-per-day purchase probability is `p_buy = sqrt(p_buy_season * p_buy_persona)`, where `p_buy_season = store.base_popularity * day_effect`.
- **sm-R002** — `day_effect = annual_curve(day) * weekend_curve(day) * growth_curve(day)`; the curves multiply, never add.
- **sm-R003** — `annual_curve(day) = (cos(x) + 1) / 10 + 0.8`, where `x` is day-of-year mapped to `[0, 2π]`, giving a range of `[0.8, 1.0]` that peaks at year start.
- **sm-R004** — `weekend_curve` returns `1.0` on weekdays and `0.6` on Saturday and Sunday.
- **sm-R005** — `growth_curve(month_offset) = 1 + (month_offset / 12) * 0.2`, where `month_offset = (year - 2016) * 12 + month`.
- **sm-R006** — Store penetration is `min(ln(1 + pct * (e - 1)), 1)` with `pct = min(days_since_open / 365, 1)`: day 0 is 0%, day 30 about 14%, day 180 about 62%, day 365 100%.

### Calendar

- **sm-R007** — Simulation day index 0 is `--start-date` (default `2023-01-01`).
- **sm-R008** — Seasons by day of year: winter Jan 1–Mar 20 and Dec 21–31, spring Mar 21–Jun 20, summer Jun 21–Sep 20, fall Sep 21–Dec 20.
- **sm-R009** — Stores open 07:00–20:00 on weekdays and 08:00–15:00 on weekends; orders sampled outside those hours are discarded.

### Streams

- **sm-R010** — All randomness flows from PCG streams derived from `--seed`; no simulation code reads system entropy or the clock.
- **sm-R011** — Each stream's seed is a fixed mix of `--seed`, a stream name, and its indices (market, day, entity index), so no stream's values depend on the order in which others are consumed.
- **sm-R012** — Store IDs come from their own stream, so store identity is independent of market simulation.

### Day loop

- **sm-R013** — Day state for every simulated day is computed before generation starts; the per-day loop allocates no curve state.
- ~~sm-R014~~ — retired: import-cycle isolation was a Go package constraint (see `docs/adr/2026-09-27-rust-rewrite.md`).
- **sm-R025** — A scenario declares its entities and generates each day's rows from the shared day state and its own streams; the engine runs any scenario without scenario-specific code.
- **sm-R026** — When a run finishes, the engine reports rows written per entity and elapsed time unless `--quiet` is set.

### Parallelism contract

- **sm-R030** — A scenario splits generation into work units (ecommerce: one market-day), and each unit's rows are a pure function of the seed, the run configuration, the unit's indices, and state fixed before the unit starts.
- **sm-R031** — Customer pool membership, persona assignment, and activation day are a pure function of the seed, the market index, and the customer index.
- **sm-R032** — Cross-day facts (customer dedup, order counts for guild ranks) are computed after generation from emitted rows, never by threading mutable state through the day loop.
- **sm-R033** — Rows within each entity file follow the scenario's declared unit order (ecommerce: day index, then market index), then generation order within the unit, regardless of how work was scheduled.
- **sm-R034** — A scenario may run generation as ordered stages (SaaS: marketing and funnel by day, then each account's lifecycle), where a stage reads only the finished output of earlier stages.

### Personas

- **sm-R015** — Six ecommerce personas share one behavior contract: Courier, Artificer, FeastReveler, Apprentice, Wanderer, Herbalist; weights and dispositions live in `dt-catalog.md`.
- **sm-R016** — Each persona samples its order minute from its own normal distribution; samples below 0 clamp to 0, and samples outside store hours mean no order that day.
- **sm-R017** — Each persona selects items by its own count distribution and type preference, and every order has at least one item.

### Order construction

- **sm-R018** — `tax_paid = round(subtotal * store.tax_rate)` in cents, and `order_total = subtotal + tax_paid`.
- **sm-R019** — An order is generated in this order: roll `p_buy` (a miss skips the customer for the day), sample the order minute, check store hours, select items, compute subtotal, tax, and total, then roll `p_sparrow`.

### Sparrows

- **sm-R020** — A sparrow's `sent_at` is its order's `ordered_at` plus a uniform 0–19 minutes.
- **sm-R021** — Sparrow text uses one of three templates by `fan_level` (1–5): positive adjective and items above 3, negative adjective and items below 3, neutral adjective at 3.
- **sm-R022** — Sparrow vocabulary matches the sender's guild rank.

### Customers

- **sm-R023** — Only customers who placed at least one order appear in the customer output.
- **sm-R024** — Ordering customers split into four order-count cohorts (initiate, journeyman, adept, master) whose sizes differ by at most one; more orders never means a lower rank, and ties break by customer UUID.
