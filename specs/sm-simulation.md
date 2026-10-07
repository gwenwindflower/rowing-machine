# Simulation engine and ecommerce scenario

## Goals

The engine turns a seed, a date range, and a scenario into rows. It owns the calendar, the shared curves, stream derivation, and the day loop that every scenario runs on. This spec also holds the ecommerce scenario's observable behavior; its formulas live in `docs/scenarios/ecommerce.md` and its catalog tables in `docs/scenarios/ecommerce.md`. SaaS behavior lives in `sp-saas-product.md` and `gm-go-to-market.md`, and travel behavior in `tr-travel.md`.

## Vocabulary

- **Market** — the ecommerce per-store simulation unit with its own customer pool.

## Requirements

### Streams

- **sm-R010** — All randomness flows from streams derived from `--seed`; no simulation code reads system entropy or the clock.
- **sm-R011** — Each stream's seed is a fixed mix of `--seed`, a stream name, and its indices (market, day, entity index), so no stream's values depend on the order in which others are consumed.

### Engine

- **sm-R025** — A scenario declares its entities and generates each day's rows from the shared day state and its own streams; the engine runs any scenario without scenario-specific code.
- **sm-R026** — When a run finishes, the engine reports rows written per entity and elapsed time unless `--quiet` is set.

### Parallelism contract

- **sm-R030** — A scenario splits generation into work units (ecommerce: one market-day), and each unit's rows are a pure function of the seed, the run configuration, the unit's indices, and state fixed before the unit starts.
- **sm-R031** — Customer pool membership, persona assignment, and activation day are a pure function of the seed, the market index, and the customer index.
- **sm-R032** — Cross-day facts (customer dedup, order counts for loyalty tiers) are computed after generation from emitted rows, never by threading mutable state through the day loop.
- **sm-R033** — Rows within each entity file follow the scenario's declared unit order (ecommerce: day index, then market index), then generation order within the unit, regardless of how work was scheduled.
- **sm-R034** — A scenario may run generation as ordered stages (SaaS: marketing and funnel by day, then each account's lifecycle), where a stage reads only the finished output of earlier stages.
- **sm-R035** — On a machine with at least 8 cores, an all-core ecommerce CSV run at scale 100 finishes in under half the wall time of a one-worker run.
- **sm-R036** — A one-worker ecommerce CSV run at scale 100 writes at least 2 million rows per second on the benchmark machine recorded in `docs/performance.md`.

### Ecommerce orders

- **sm-R009** — Every order falls within store hours: 07:00–20:00 on weekdays and 08:00–15:00 on weekends.
- **sm-R017** — Every order has at least one item.
- **sm-R018** — `tax_paid = round(subtotal * store.tax_rate)` in cents, and `order_total = subtotal + tax_paid`.

### Tweets

- **sm-R020** — A tweet's `tweeted_at` is its order's `ordered_at` plus a uniform 0–19 minutes.
- **sm-R021** — Tweet text uses one of three templates by `fan_level` (1–5): positive adjective and items above 3, negative adjective and items below 3, neutral adjective at 3.
- **sm-R022** — Tweet vocabulary matches the sender's loyalty tier.

### Customers

- **sm-R023** — Only customers who placed at least one order appear in the customer output.
- **sm-R024** — Ordering customers split into four order-count cohorts whose sizes differ by at most one; more orders never means a lower rank, and ties break by customer UUID.

Retired: sm-R001–sm-R008, sm-R012–sm-R016, sm-R019.
