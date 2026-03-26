# Rowing Machine — Comprehensive Reference for Go Port of Jaffle Shop Generator

This document provides a complete specification of the Python jaffle-shop-generator,
sufficient to rewrite the tool in Go without constant reference back to the Python source.
It covers architecture, simulation logic, entity models, mathematical formulas, static data,
output schemas, and recommendations for the Go rewrite.

The new tool is called 'Rowing Machine' - and after the logic is ported, the items and documentation will be tailored to a new fictional business called 'Rowing Outfitters', with the documentation taking the angle of being the factory that supplies their goods (just for fun). 'Rowing Outfitters' is a real separate project which provides a SQL trainer web app, and this will supply the data for people to learn in that sandbox.

The documentation below describes the original Jaffle Shop project, and how to adapt it for Rowing Machine/Rowing Outfitters.

**IMPORTANT**: We are leaving the original schemas and items in place during migration. To avoid confusion and ensure the migration is consistent, the Rowing Machine will be outputting restaurant locations, sandwich purchases, etc. We will adapt this after migration.

---

## 1. Project Overview

The Jaffle Shop Generator is a synthetic data generator that simulates a fictional chain
of jaffle (Australian grilled sandwich) restaurants. Rather than generating random rows,
it takes a **simulation approach**: customers with distinct behavioral personas visit stores
over simulated calendar days, influenced by seasonality, day-of-week effects, growth curves,
and market penetration dynamics. This produces realistic-looking data with temporal patterns
that make it excellent for analytics training datasets (especially for dbt).

### What It Produces

Seven CSV files representing a normalized relational schema:

- `customers` — customer dimension
- `orders` — order fact table
- `items` — order line items (denormalized with product info)
- `stores` — store dimension
- `products` — menu item catalog
- `supplies` — ingredient/material catalog with product associations
- `tweets` — social media posts linked to orders

### Current Python Stack

- **CLI**: Typer (thin wrapper over Click)
- **Randomization**: NumPy (`np.random`) + Faker (names, UUIDs, shuffling)
- **Progress display**: Rich (via Typer integration)
- **Output**: Python stdlib `csv.DictWriter`

---

## 2. Architecture & Module Map

```text
cli.py                    CLI entry point (Typer app)
simulation.py             Orchestrator: creates markets, runs day loop, writes output
time.py                   Day, Season, hours-of-operation types
curves.py                 AnnualCurve, WeekendCurve, GrowthCurve
stores/
  store.py                Store entity with popularity, hours, tax rate
  market.py               Market: manages customer pool, activation, daily sim
  item.py                 Item entity (menu product) + ItemType enum
  inventory.py            Inventory registry: 10 hardcoded menu items
  supply.py               Supply entity (ingredient/material)
  stock.py                Stock registry: 29 hardcoded supplies
customers/
  customers.py            Abstract Customer + 6 persona subclasses
  order.py                Order entity with tax calculation
  tweet.py                Tweet entity with sentiment-based content generation
```

### Data Flow

```text
CLI(years, prefix)
  -> Simulation(years, prefix)
       creates 6 Store instances
       creates 6 Market instances (one per store), each with a customer pool
  -> run_simulation()
       for day_index in 0..(365*years):
         for each market:
           market.sim_day(day)
             -> activate new customers based on penetration curve
             -> for each active customer:
                  roll p_buy -> if buy, generate Order
                  roll p_tweet -> if tweet, generate Tweet
                  yield (order, tweet)
           collect orders and tweets
  -> save_results()
       serialize all entities to dicts
       write 7 CSV files to ./jaffle-data/
```

---

## 3. Temporal System

### Epoch

All simulation time is relative to a fixed epoch: **September 1, 2018**.

Day index 0 = 2018-09-01. Day index 365 = 2019-09-01. And so on.

### The Day Object

A `Day` combines a date index (days since epoch) with an optional intra-day minute offset.
On construction, it precomputes three multiplicative "effects" from the curve system.

```text
Day(date_index, minutes=0):
  date = EPOCH + timedelta(days=date_index, minutes=minutes)
  effects = [
    AnnualCurve.eval(date),
    WeekendCurve.eval(date),
    GrowthCurve.eval(date),
  ]

get_effect() -> product of all effects (multiplicative)
```

**Properties:**

- `day_of_week` — 0=Monday through 6=Sunday (Python weekday convention)
- `is_weekend` — True if weekday >= 5 (Saturday or Sunday)
- `season` — derived from month/day (see Season below)
- `total_minutes` — `hour * 60 + minute` of the datetime

**`at_minute(minutes)`** — creates a new Day with same date_index but different minute offset.
Used to stamp orders and tweets at specific times within a day.

### Seasons

Derived from calendar date with fixed cutoffs:

| Season | Date Range |
| --- | --- |
| WINTER | Jan 1 – Mar 20 |
| SPRING | Mar 21 – Jun 20 |
| SUMMER | Jun 21 – Sep 20 |
| FALL | Sep 21 – Dec 20 |

Note: Dec 21–31 falls through to WINTER (the `from_date` method returns WINTER as default).

### Hours of Operation

All stores share the same schedule:

| Day Type | Opens | Closes | Duration |
| --- | --- | --- | --- |
| Weekday (Mon–Fri) | 7:00 AM | 8:00 PM | 13 hours (780 min) |
| Weekend (Sat–Sun) | 8:00 AM | 3:00 PM | 7 hours (420 min) |

Orders are rejected if the sampled order time falls outside operating hours
(`is_open_at` check compares the order's time against the schedule for that day type).

---

## 4. Curve System (Temporal Effects)

Three curves modulate purchase probability. Each curve maps a date to a multiplier.
The combined effect is the product of all three.

### Curve Evaluation Pattern

All curves share this evaluation pattern:

```text
eval(date):
  domain_value = TranslateDomain(date)        // map date to domain index
  domain_index = domain_value % len(Domain)   // wrap into domain array bounds
  translated_value = Domain[domain_index]      // look up the x-value
  return Expr(translated_value)                // evaluate the function at x
```

### AnnualCurve (Seasonality)

- **Domain**: 365 evenly-spaced values from 0 to 2pi
- **TranslateDomain**: day-of-year (1–365) via `timetuple().tm_yday`
- **Expr**: `(cos(x) + 1) / 10 + 0.8`
- **Output range**: [0.8, 1.0]
  - Peak (1.0) at mid-year (~day 182)
  - Trough (0.8) at year boundaries (day 1 and ~365)
- **Effect**: ~20% seasonal swing in order volume

### WeekendCurve

> **BUG IN PYTHON**: The WeekendCurve is non-functional. Domain is `[0,1,2,3,4,5]`
> (6 values), TranslateDomain returns `weekday() - 1` (range -1 to 5), and after
> modulo 6 the domain values range 0–5. But `Expr` checks `x >= 6` to return 0.6,
> which is never true. **The curve always returns 1.0.** Weekend effects only come from
> individual persona `p_buy_persona` methods checking `day.is_weekend`.

**Intended behavior** (implement correctly in Go):

- Weekdays: 1.0
- Weekends: 0.6 (40% reduction)

### GrowthCurve

- **Domain**: integers 0–499 (500 months)
- **TranslateDomain**: `(year - 2016) * 12 + month`
- **Expr**: `1 + (x / 12) * 0.2`
- **Effect**: ~20% growth per year, linear in months
  - At simulation start (Sept 2018, month ~32): `1 + (32/12)*0.2 ≈ 1.53`
  - After 1 year (Sept 2019, month ~44): `1 + (44/12)*0.2 ≈ 1.73`

### Combined Effect

```text
day_effect = annual_curve * weekend_curve * growth_curve
```

This multiplicative effect feeds into the purchase probability:

```text
p_buy_season = store.base_popularity * day_effect
```

---

## 5. Entity Models

### Store

Immutable entity representing a physical restaurant location.

| Field | Type | Notes |
| --- | --- | --- |
| id | UUID | Generated once per instance via Faker |
| name | string | City name ("Philadelphia", "Brooklyn", etc.) |
| base_popularity | float | Range [0.85, 0.95] — baseline purchase likelihood multiplier |
| hours_of_operation | WeekHoursOfOperation | Weekday + weekend schedules |
| opened_day | Day | Day index when store opened |
| tax_rate | float | Local sales tax rate |

**Key methods:**

- `p_buy(day)` = `base_popularity * day.get_effect()`
- `is_open(day)` = `day.date >= opened_day.date` (has the store opened yet?)
- `is_open_at(day)` = is the store open at this specific time of day?
- `days_since_open(day)` = `day.date_index - opened_day.date_index`

**Output schema (`stores.csv`):**

| Column | Type | Notes |
| --- | --- | --- |
| id | string (UUID) | |
| name | string | |
| opened_at | string (ISO date) | |
| tax_rate | string | Decimal as string, e.g. "0.06" |

### Customer (Abstract Base + 6 Personas)

Immutable entity representing a customer with behavioral traits.

| Field | Type | Generation |
| --- | --- | --- |
| id | UUID | Faker |
| store | Store | Assigned at creation (which store they frequent) |
| name | string | `Faker.name()` |
| favorite_number | int | `randint(1, 100)` — drives variance in buying propensity per-persona |
| fan_level | int | `randint(1, 5)` — drives tweet sentiment |

**Core probability logic:**

```text
p_buy(day) = sqrt(p_buy_season(day) * p_buy_persona(day))

where:
  p_buy_season = store.p_buy(day) = store.base_popularity * day.get_effect()
  p_buy_persona = persona-specific (see persona table below)
```

The geometric mean (square root of product) blends seasonal and persona effects.

**Output schema (`customers.csv`):**

| Column | Type |
| --- | --- |
| id | string (UUID) |
| name | string |

Note: `favorite_number`, `fan_level`, and persona type are NOT exported — they are
internal simulation parameters only.

### Customer Personas

Six concrete customer types, each with distinct purchasing and tweeting behavior:

#### Persona: Commuter (25% of market)

"the regular, thanks"

| Attribute | Value |
| --- | --- |
| p_buy_persona (weekday) | `0.5 + (favorite_number / 100) * 0.3` → [0.5, 0.8] |
| p_buy_persona (weekend) | 0.001 |
| p_tweet | 0.2 |
| Order time distribution | N(mu=60, sigma=30) minutes from midnight |
| Items ordered | 1 beverage |

> **BUG IN PYTHON**: Order time mean of 60 minutes = 1:00 AM. Stores open at 7:00 AM.
> Nearly all sampled order times will be before opening, causing `is_open_at` to return
> false and the order to be discarded. This makes Commuters nearly absent from the data
> despite being 25% of the addressable market.
>
> **Go fix**: Use `mu=7*60` (420, i.e. 7:00 AM) or similar morning commute time.

#### Persona: RemoteWorker (25% of market)

"This person works from a coffee shop"

| Attribute | Value |
| --- | --- |
| p_buy_persona (weekday) | `(favorite_number / 100) * 0.4` → [0.0, 0.4] |
| p_buy_persona (weekend) | 0.001 |
| p_tweet | 0.01 |
| Order time distribution | N(mu=420, sigma=180) minutes from midnight (7 AM, wide spread) |
| Items ordered | 1–2 beverages (30% chance of 2nd) + 0–1 jaffle (30% chance) |

#### Persona: BrunchCrowd (10% of market)

"Do you sell mimosas?"

| Attribute | Value |
| --- | --- |
| p_buy_persona (weekend) | `0.2 + (favorite_number / 100) * 0.2` → [0.2, 0.4] |
| p_buy_persona (weekday) | 0 (never buys on weekdays) |
| p_tweet | 0.8 |
| Order time distribution | N(mu=300+((fav_num-50)/50)*120, sigma=120) — centers between 3–7 AM depending on favorite_number |
| Items ordered | `1 + floor(favorite_number / 20)` jaffles AND same number of beverages |

Note: This persona creates the largest orders (up to 6 jaffles + 6 beverages for fav_num=100).

#### Persona: Student (20% of market)

"coffee might help"

| Attribute | Value |
| --- | --- |
| p_buy_persona (non-summer) | `0.1 + (favorite_number / 100) * 0.4` → [0.1, 0.5] |
| p_buy_persona (summer) | 0 (on break, absent from simulation) |
| p_tweet | 0.8 |
| Order time distribution | N(mu=540, sigma=120) minutes (9 AM) |
| Items ordered | 1 beverage + 0–1 jaffle (50% chance) |

#### Persona: Casuals (10% of market)

"just popping in"

| Attribute | Value |
| --- | --- |
| p_buy_persona | 0.1 (constant, all days) |
| p_tweet | 0.1 |
| Order time distribution | N(mu=300, sigma=120) minutes (5 AM) |
| Items ordered | `floor(random()*10/3)` beverages + same formula for jaffles (0–3 each) |

#### Persona: HealthNut (10% of market)

"A light beverage in the sunshine as a treat"

| Attribute | Value |
| --- | --- |
| p_buy_persona (summer) | `0.1 + (favorite_number / 100) * 0.4` → [0.1, 0.5] |
| p_buy_persona (other) | 0.2 |
| p_tweet | 0.6 |
| Order time distribution | N(mu=300, sigma=120) minutes (5 AM) |
| Items ordered | 1 beverage only |

### Persona Summary Table

| Persona | Market % | Weekday/Weekend | Tweet Rate | Order Size | Time Center |
| --- | --- | --- | --- | --- | --- |
| Commuter | 25% | Weekday only | 20% | 1 bev | 1 AM* |
| RemoteWorker | 25% | Weekday only | 1% | 1-2 bev, 0-1 jaffle | 7 AM |
| BrunchCrowd | 10% | Weekend only | 80% | 1-6 each | Varies |
| Student | 20% | All (not summer) | 80% | 1 bev, 0-1 jaffle | 9 AM |
| Casuals | 10% | All | 10% | 0-3 each | 5 AM |
| HealthNut | 10% | All | 60% | 1 bev | 5 AM |

*Likely a bug; see Commuter section above.

### Order

Mutable entity (computed fields set in `__post_init__`).

| Field | Type | Notes |
| --- | --- | --- |
| id | UUID | Faker |
| customer | Customer | Back-reference |
| day | Day | Includes intra-day minute for timestamp |
| store | Store | Back-reference |
| items | list[Item] | Selected menu items |
| subtotal | float | `sum(item.price for item in items)` (dollars) |
| tax_paid | float | `store.tax_rate * subtotal` |
| total | float | `subtotal + tax_paid` |

**Output schema (`orders.csv`):**

| Column | Type | Notes |
| --- | --- | --- |
| id | string (UUID) | |
| customer | string (UUID) | FK to customers.id |
| ordered_at | string (ISO datetime) | Full datetime including time component |
| store_id | string (UUID) | FK to stores.id |
| subtotal | int | Cents (dollars * 100, truncated) |
| tax_paid | int | Cents |
| order_total | int | `int(subtotal_cents) + int(tax_cents)` — note: computed from individually-truncated cent values, not from float total |

**Output schema (`items.csv`):**

Each order's items are flattened into individual rows.

| Column | Type | Notes |
| --- | --- | --- |
| sku | string | e.g. "JAF-001" |
| name | string | Product name |
| type | string | "ItemType.JAFFLE" or "ItemType.BEVERAGE" (Python enum str) |
| price | int | Cents |
| description | string | Product description |

### Tweet

Generated for some orders depending on persona tweet probability.

| Field | Type | Notes |
| --- | --- | --- |
| id | UUID | Faker |
| day | Day | Order time + random 0–19 minute delay |
| customer | Customer | Back-reference |
| order | Order | The triggering order |
| content | string | Generated based on fan_level and items ordered |

**Content generation logic:**

1. Build items sentence:
   - 1 item: `"Ordered a {name}"`
   - 2 items: `"Ordered a {name1} and a {name2}"`
   - 3+ items: `"Ordered a {name1}, a {name2}, ..., and a {nameN}"`

2. Select template by fan_level:
   - **fan_level > 3** (positive): `"Jaffles from the Jaffle Shop are {adj}! {items}."`
     - Adjectives: "the best", "awesome", "delicious", "amazing", "fantastic", "sooo gooood", "my favorite"
   - **fan_level < 3** (negative): `"Jaffle Shop again. {items}. This place is {adj}."`
     - Adjectives: "terrible", "the worst", "awful", "disgusting", "gross", "inedible", "my least favorite"
   - **fan_level == 3** (neutral): `"Jaffle shop is {adj}. {items}."`
     - Adjectives: "okay", "fine", "alright", "average", "pretty decent", "solid", "not bad", "just meh"

**Output schema (`tweets.csv`):**

| Column | Type |
| --- | --- |
| id | string (UUID) |
| user_id | string (UUID) — FK to customers.id |
| tweeted_at | string (ISO datetime) |
| content | string |

### Item (Menu Product)

Immutable, statically defined. 10 total items in the catalog.

| Field | Type |
| --- | --- |
| sku | string | e.g. "JAF-001", "BEV-003" |
| name | string |
| description | string |
| type | ItemType enum — JAFFLE or BEVERAGE |
| price | float (dollars) |

### Supply (Ingredient/Material)

Immutable, statically defined. 29 total supplies.

| Field | Type |
| --- | --- |
| id | string | e.g. "SUP-001" |
| name | string |
| cost | float (dollars) |
| perishable | bool |
| skus | list[string] | Which product SKUs use this supply |

**Output schema (`supplies.csv`):**

Supplies are denormalized: one row per (supply, sku) pair. A supply used by 5 products
appears as 5 rows.

| Column | Type | Notes |
| --- | --- | --- |
| id | string | e.g. "SUP-001" |
| name | string | |
| cost | int | Cents |
| perishable | string | "True" or "False" (string, not bool) |
| sku | string | Single SKU for this row |

---

## 6. Market & Customer Activation

Each store has a `Market` that manages its customer pool.

### Market Initialization

1. For each persona in the PersonaMix, create `weight * num_customers` instances
2. Shuffle the full list randomly
3. All start in `addressable_customers` (inactive pool)
4. `active_customers` starts empty

### Market Penetration Curve

Each simulated day, the market calculates how many customers should be active:

```text
days_since_open = store.days_since_open(day)

if days_since_open < 0:
    market_penetration = 0          // store not yet open
elif days_since_open < 7:           // first week: steeper initial ramp
    pct = min(days_since_open / 365, 1)
    market_penetration = min(ln(1.2 + pct * (e - 1.2)), 1)
else:                               // after first week: standard log ramp
    pct = min(days_since_open / 365, 1)
    market_penetration = min(ln(1 + pct * (e - 1)), 1)
```

**Penetration progression:**

- Day 0: ~18% (first-week formula: ln(1.2) ≈ 0.182)
- Day 7: ~3% (switches to standard formula: ln(1.019) ≈ 0.019) — **note the discontinuity**
- Day 30: ~14% — ln(1 + 0.082 * 1.718) ≈ 0.131
- Day 180: ~62% — ln(1 + 0.493 * 1.718) ≈ 0.617
- Day 365: 100% — ln(e) = 1.0

> **BUG/QUIRK**: There's a **discontinuity at day 7** where the penetration drops from
> ~18% (first-week formula) to ~3% (standard formula). This likely wasn't intended.
> The first-week formula starts higher because of the 1.2 offset. In the Go rewrite,
> consider using a single smooth logarithmic curve.

New customers are activated by popping from the shuffled `addressable_customers` list:

```text
desired = market_penetration * len(addressable_customers)
to_add = int(desired - len(active_customers))
for i in range(to_add):
    customer = addressable_customers.pop()
    active_customers.append(customer)
```

Once activated, customers remain active for the rest of the simulation.

### Daily Customer Simulation

For each active customer on each day:

```text
p_buy = sqrt(p_buy_season * p_buy_persona)
p_tweet = p_tweet_persona

roll_buy = random()     // uniform [0, 1)
roll_tweet = random()

if p_buy > roll_buy:
    if p_tweet > roll_tweet:
        order = generate_order(day)
        if order and len(order.items) > 0:
            return (order, generate_tweet(order))
        else:
            return (None, None)
    else:
        return (generate_order(day), None)
else:
    return (None, None)
```

**Important detail**: The tweet probability is checked *before* the order is generated.
If the buy check passes but tweet check fails, only the order is generated (no tweet).
If both pass, the order is generated, and if valid, the tweet is generated too.
But if the order turns out to be None (store closed at sampled time), the tweet is also
discarded.

### Order Generation

```text
1. Call persona's get_order_items(day) -> list of Items
   (randomly selects items from inventory by type)
2. Call persona's get_order_minute(day) -> int
   (samples from normal distribution, clamped to >= 0)
3. Create a Day at that minute: day.at_minute(order_minute)
4. Check store.is_open_at(order_day) -> bool
5. If open: create Order with items, store, customer, day
6. If closed: return None (order dropped)
```

Items are selected via `Inventory.get_item_type(type, count)` which randomly picks
`count` items of the given type (JAFFLE or BEVERAGE) with replacement.

---

## 7. Static Data

### Store Configuration

| Name | Popularity | Opens (day index) | TAM | Tax Rate |
| --- | --- | --- | --- | --- |
| Philadelphia | 0.85 | 0 | 900 | 6.00% |
| Brooklyn | 0.95 | 192 | 1400 | 4.00% |
| Chicago | 0.92 | 605 | 1200 | 6.25% |
| San Francisco | 0.87 | 615 | 1100 | 7.50% |
| New Orleans | 0.92 | 920 | 800 | 4.00% |
| Los Angeles | 0.87 | 1107 | 800 | 8.00% |

TAM = Total Addressable Market (number of customers). Derived from a base number *scale
factor (100). E.g. Philadelphia = 9* 100 = 900 customers.

Note: New Orleans and Los Angeles don't open until day 920 and 1107 respectively.
With the default 3-year (1095-day) simulation, all 6 stores open and generate data.
A 1-year simulation would only include Philadelphia and Brooklyn.

### Store Opening Behavior by Sizing Mode

The store opening schedule depends on which sizing mode is active:

**Simulation mode** (default, `--years`): Stores open on their original staggered
schedule (day 0, 192, 605, 615, 920, 1107). The default of 3 years ensures all stores
appear in the output. This mode tells a "growth story" — the chain expands city by city,
and the data reflects realistic staggered market entry.

**Target-rows mode** (`--target-rows N`): All 6 stores open on day 0. The tool
auto-calculates the number of simulation days needed to produce approximately N order
rows, using an estimate based on:

```text
days ≈ target_rows / (avg_orders_per_day_per_store * num_active_stores)
```

This mode is a "data generation utility" — the user wants a specific volume of data,
not a growth narrative. Having all stores active immediately produces more predictable,
evenly distributed output and makes hitting the target row count more precise. The tool
may overshoot slightly; output is truncated to exactly N rows.

### Menu Items (Inventory)

#### Jaffles

| SKU | Name | Price | Description |
| --- | --- | --- | --- |
| JAF-001 | nutellaphone who dis? | $11 | nutella and banana jaffle |
| JAF-002 | doctor stew | $11 | house-made beef stew jaffle |
| JAF-003 | the krautback | $12 | lamb and pork bratwurst with house-pickled cabbage sauerkraut and mustard |
| JAF-004 | flame impala | $14 | pulled pork and pineapple al pastor marinated in ghost pepper sauce |
| JAF-005 | mel-bun | $12 | melon and minced beef bao, in a jaffle, savory and sweet |

#### Beverages

| SKU | Name | Price | Description |
| --- | --- | --- | --- |
| BEV-001 | tangaroo | $6 | mango and tangerine smoothie |
| BEV-002 | chai and mighty | $5 | oatmilk chai latte with protein boost |
| BEV-003 | vanilla ice | $6 | iced coffee with house-made french vanilla syrup |
| BEV-004 | for richer or pourover | $7 | daily selection of single estate beans for a delicious hot pourover |
| BEV-005 | adele-ade | $4 | a kiwi and lime agua fresca |

### Supplies (Stock)

29 supplies. Non-perishable items (packaging) are shared across all products of a type.
Perishable items (ingredients) are specific to individual products.

#### Non-Perishable (Packaging)

| ID | Name | Cost | Used By |
| --- | --- | --- | --- |
| SUP-001 | compostable cutlery - knife | $0.07 | All jaffles |
| SUP-002 | cutlery - fork | $0.07 | All jaffles |
| SUP-003 | serving boat | $0.11 | All jaffles |
| SUP-004 | napkin | $0.04 | All jaffles |
| SUP-005 | 16oz compostable clear cup | $0.13 | All beverages |
| SUP-006 | 16oz compostable clear lid | $0.04 | All beverages |
| SUP-007 | biodegradable straw | $0.13 | All beverages |

#### Perishable (Ingredients)

| ID | Name | Cost | Used By |
| --- | --- | --- | --- |
| SUP-008 | chai mix | $0.98 | BEV-002 |
| SUP-009 | bread | $0.33 | All jaffles |
| SUP-010 | cheese | $0.20 | JAF-002, JAF-003, JAF-004, JAF-005 |
| SUP-011 | nutella | $0.46 | JAF-001 |
| SUP-012 | banana | $0.13 | JAF-001 |
| SUP-013 | beef stew | $1.69 | JAF-002 |
| SUP-014 | lamb and pork bratwurst | $2.34 | JAF-003 |
| SUP-015 | house-pickled cabbage sauerkraut | $0.43 | JAF-003 |
| SUP-016 | mustard | $0.07 | JAF-003 |
| SUP-017 | pulled pork | $2.15 | JAF-004 |
| SUP-018 | pineapple | $0.26 | JAF-004 |
| SUP-019 | melon | $0.33 | JAF-005 |
| SUP-020 | minced beef | $1.24 | JAF-005 |
| SUP-021 | ghost pepper sauce | $0.20 | JAF-004 |
| SUP-022 | mango | $0.32 | BEV-001 |
| SUP-023 | tangerine | $0.20 | BEV-001 |
| SUP-024 | oatmilk | $0.11 | BEV-002 |
| SUP-025 | whey protein | $0.36 | BEV-002 |
| SUP-026 | coffee | $0.52 | BEV-003, BEV-004 |
| SUP-027 | french vanilla syrup | $0.72 | BEV-003 |
| SUP-028 | kiwi | $0.20 | BEV-005 |
| SUP-029 | lime | $0.13 | BEV-005 |

---

## 8. CLI Parameters

### Current Python Parameters

| Parameter | Type | Default | Description |
| --- | --- | --- | --- |
| `years` | int (positional arg) | 1 | Number of years to simulate (365 days each) |
| `--pre` | string (option) | "raw" | Prefix for output CSV filenames |

### Go Version: Two Sizing Modes

The Go version supports two mutually exclusive sizing modes:

**Simulation mode** (default) — specify duration and scale, get a growth story:

| Flag | Type | Default | Description |
| --- | --- | --- | --- |
| `--years` | int | 3 | Number of years to simulate (365 days each) |
| `--scale` | int | 100 | Customer pool multiplier (controls data volume) |
| `--start-date` | string | "2018-09-01" | Simulation epoch date |

**Target-rows mode** — specify a row count, get exactly that many order rows:

| Flag | Type | Default | Description |
| --- | --- | --- | --- |
| `--target-rows` | int | — | Target number of order rows to generate |

`--target-rows` is mutually exclusive with `--years`. When active, all 6 stores open
from day 0 and the tool auto-calculates the simulation duration needed. `--scale` and
`--start-date` still apply in target-rows mode.

Rationale: target-rows is a "data generation utility" mode, not a "tell a growth story"
mode. Having all stores active immediately is the right trade-off for predictable,
precise row counts.

### Go Version: Output Options

| Flag | Type | Default | Description |
| --- | --- | --- | --- |
| `--format` | enum | "csv" | Output format: `csv`, `jsonl`, `parquet` |
| `--messy` | bool | false | Inject realistic data quality issues (see Output Modes section) |
| `--output-dir` | string | "./output" | Output directory path |
| `--pre` | string | "raw" | Prefix for output filenames |
| `--compress` | bool | false | Gzip output files (`.csv.gz`, `.jsonl.gz`; no-op for parquet which has built-in compression) |

### Go Version: Execution Options

| Flag | Type | Default | Description |
| --- | --- | --- | --- |
| `--seed` | int64 | 0 (random) | Seed for deterministic, reproducible output. 0 = use random seed. |
| `--workers` | int | runtime.NumCPU() | Parallel workers for store simulation |
| `--quiet` | bool | false | Suppress progress output |

---

## 9. Known Bugs to Fix in Go Rewrite

### Bug 1: WeekendCurve Never Activates

The Python WeekendCurve's `Expr(x)` checks `x >= 6` but the domain only produces values
0–5, so the 0.6 weekend multiplier is never applied. In Go, implement the weekend curve
correctly:

```go
func weekendEffect(weekday time.Weekday) float64 {
    if weekday == time.Saturday || weekday == time.Sunday {
        return 0.6
    }
    return 1.0
}
```

### Bug 2: Commuter Order Time (1 AM Instead of ~7 AM)

The Commuter's `get_order_minute` uses `N(mu=60, sigma=30)` which centers at 1:00 AM.
Since stores open at 7:00 AM, virtually all Commuter orders fail the `is_open_at` check.
Fix by using a realistic commute time, e.g. `N(mu=450, sigma=30)` for a 7:30 AM center.

### Bug 3: `total_minutes_elapsed` Uses Seconds Instead of Hours

In `time.py` line 33: `return t.second * 60 + t.minute` — should be `t.hour * 60 + t.minute`.
This function is used for `DayHoursOfOperation.total_minutes_open` and `iter_minutes`,
but these methods aren't called in the main simulation loop, so it doesn't affect output.
Still, implement correctly in Go.

### Bug 4: Market Penetration Discontinuity at Day 7

The penetration formula switches between two different logarithmic functions at day 7,
creating a sharp drop from ~18% to ~3%. Use a single smooth curve in Go.

### Bug 5: Non-Deterministic Output

No seed is set for NumPy or Faker random generators. Every run produces different data.
In Go, use a seedable PRNG (e.g. `math/rand/v2` with a configurable seed). See the
Deterministic Generation section for full requirements.

### Feature Gap: No "Messy" Data Mode

The Python version only produces clean, normalized output. For analytics engineering
students, a key learning exercise is dealing with real-world data quality issues. The
Go version should add a `--messy` flag that injects realistic data quality problems
(null names, duplicate rows, mixed date formats, etc.). See the Output Modes section
for the full specification.

---

## 10. Go Rewrite Recommendations

### Architecture

```text
cmd/rowing-machine/main.go           CLI entry (Charm libraries: huh for TUI, bubbletea)
internal/
  simulation/
    simulation.go             Orchestrator
    day.go                    Day, Season, time utilities
    curves.go                 AnnualCurve, WeekendCurve, GrowthCurve
  models/
    store.go                  Store type
    customer.go               Customer interface + persona implementations
    order.go                  Order type
    tweet.go                  Tweet type
    item.go                   Item type, ItemType enum
    supply.go                 Supply type
  market/
    market.go                 Market with customer pool and penetration
  catalog/
    inventory.go              Menu items (static data)
    stock.go                  Supply items (static data)
  output/
    writer.go                 Interface for output writers
    csv.go                    CSV writer
    parquet.go                Parquet writer (optional)
```

### Performance: Where the Big Wins Are

The Python version's performance bottlenecks and how Go addresses them:

#### 1. Per-Customer-Per-Day Object Allocation (BIGGEST WIN)

The Python version creates a new `Day` object for every day in the simulation loop,
and each Day construction triggers 3 curve evaluations involving NumPy array creation
(the Curve classes instantiate and compute domains on every `.eval()` call via `cls()`).
For 365 days *6 markets* thousands of customers, this is millions of redundant
allocations and computations.

**Go approach:**

- Pre-compute all 365*years Day effects in a single pass into a `[]float64` slice
- Pre-compute the curve lookup tables once at startup (they're deterministic)
- Pass day effects by index, not by object creation

```go
type DayEffects struct {
    Annual  [366]float64  // indexed by day-of-year
    Weekend [7]float64    // indexed by weekday
    Growth  []float64     // indexed by month offset
}

// Pre-compute once
func NewDayEffects() *DayEffects { ... }
```

#### 2. Parallel Store Simulation (MAJOR WIN)

Each store/market's daily simulation is completely independent. The Python version
processes them sequentially. In Go, use goroutines — but the merge must be deterministic
to ensure reproducible output (see Deterministic Generation section).

```go
// Each market simulates into its own buffer
type MarketResult struct {
    MarketIndex int
    Orders      []Order
    Tweets      []Tweet
}

results := make([]MarketResult, len(markets))
var wg sync.WaitGroup

for i, market := range markets {
    wg.Add(1)
    go func(idx int, m *Market) {
        defer wg.Done()
        var orders []Order
        var tweets []Tweet
        for dayIdx := 0; dayIdx < simDays; dayIdx++ {
            for order, tweet := range m.SimDay(dayIdx) {
                if order != nil { orders = append(orders, *order) }
                if tweet != nil { tweets = append(tweets, *tweet) }
            }
        }
        results[idx] = MarketResult{idx, orders, tweets}
    }(i, market)
}

wg.Wait()

// Merge in fixed market order (0, 1, 2, ...) for determinism
for _, r := range results {
    writeOrders(r.Orders)
    writeTweets(r.Tweets)
}
```

Each market has its own PRNG instance seeded deterministically from the global seed
plus the market index: `rand.New(rand.NewPCG(seed+uint64(marketIndex), 0))` from
`math/rand/v2`. Markets simulate in parallel but results are merged in fixed index
order so that identical seeds always produce byte-identical output.

Alternative ordering: day-major instead of market-major. Simulate all markets for
day 0, merge by market index, then day 1, etc. This produces chronologically ordered
output but requires synchronizing goroutines per-day. Choose based on output format
needs — JSONL event stream mode benefits from chronological ordering.

#### 3. UUID Generation (MODERATE WIN — Critical for Determinism)

The Python version uses Faker for UUID generation which is slow and non-deterministic
(Faker uses `uuid.uuid4()` which reads from `os.urandom` / `crypto/rand`).

**For the Go version, UUIDs MUST be generated from the seeded PRNG**, not from
`uuid.New()` (which also uses `crypto/rand`). This is essential for reproducible
output — same seed must produce the same UUIDs every time.

```go
func uuidFromRNG(rng *rand.Rand) uuid.UUID {
    var id uuid.UUID
    rng.Read(id[:])
    id[6] = (id[6] & 0x0f) | 0x40 // version 4
    id[8] = (id[8] & 0x3f) | 0x80 // variant RFC4122
    return id
}
```

Every entity (store, customer, order, tweet) gets its UUID from the appropriate
market's PRNG. Since each market has a deterministically-seeded PRNG and processes
customers in fixed order, the UUID sequence is fully reproducible.

#### 4. Name Generation (MODERATE WIN)

Faker name generation is relatively expensive in Python. In Go, pre-generate a pool
of names at startup (or use a lightweight name generator). Since names just need to
look realistic, a pool of ~1000 first names + ~1000 last names combined randomly is
sufficient and avoids per-customer overhead.

#### 5. Output Writing (MODERATE WIN)

The Python version builds all entities into in-memory lists of dicts, then writes CSV.
For multi-GB datasets, this will OOM in Python.

**Go approach — streaming output:**

- Write orders and tweets as they're generated (streaming to buffered CSV writers)
- Use `encoding/csv` with a `bufio.Writer` for fast I/O
- For Parquet output, batch rows and flush periodically
- Track unique customers in a map, write the customers CSV at the end
- Products and supplies are static — write once at the end

```go
type StreamingWriter struct {
    orders   *csv.Writer
    items    *csv.Writer
    tweets   *csv.Writer
    customers map[uuid.UUID]Customer  // deduplicate
}
```

This changes memory usage from O(total_orders) to O(active_customers) — critical for
generating GBs of data.

#### 6. Avoid Float Arithmetic for Money

The Python version stores prices as floats ($11.00) then converts to cents at output time
with `int(float * 100)`. This can cause rounding errors (the code has a TODO comment about
this).

**Go approach:** Store all monetary values as `int64` cents from the start. Define menu
prices in cents. Compute tax in cents with explicit rounding:

```go
type Money int64 // cents

func (m Money) Tax(rate float64) Money {
    return Money(math.Round(float64(m) * rate))
}
```

#### 7. Normal Distribution Sampling

The Python version uses `np.random.normal()` which is excellent but incurs Python overhead
per call. In Go, use `rand.NormFloat64()` from the standard library, which is implemented
in pure Go and very fast:

```go
orderMinute := int(rng.NormFloat64()*sigma + mu)
if orderMinute < 0 {
    orderMinute = 0
}
```

### Suggested Data Structures

#### Customer as Interface with Persona Structs

```go
type Customer struct {
    ID             uuid.UUID
    StoreID        uuid.UUID
    Name           string
    FavoriteNumber int
    FanLevel       int
    Persona        Persona  // interface
}

type Persona interface {
    PBuyPersona(day *DayState) float64
    PTweetPersona(day *DayState) float64
    OrderMinute(rng *rand.Rand, day *DayState) int
    OrderItems(rng *rand.Rand, inv *Inventory) []Item
}
```

Since Customer is a frozen dataclass in Python (value type), in Go it can be a plain
struct passed by pointer. The persona-specific behavior is cleanly expressed as an
interface, avoiding the need for an abstract base class pattern.

#### Day as Lightweight Value

Instead of the Python Day object which carries datetime objects and computed effects,
use a lightweight struct:

```go
type DayState struct {
    Index      int
    Date       time.Time
    IsWeekend  bool
    Season     Season
    Effect     float64  // pre-computed product of all curves
    OpensAt    int      // minutes from midnight
    ClosesAt   int      // minutes from midnight
}
```

Pre-compute a `[]DayState` slice for the entire simulation at startup.

### Concurrency Model Summary

```text
Main goroutine:
  1. Parse CLI / run TUI form
  2. Initialize catalog (items, supplies) — static, shared read-only
  3. Pre-compute []DayState for all sim days
  4. Create Markets (one per store, each with deterministically-seeded PRNG)
  5. Launch market goroutines (one per store)
       Each: iterate days, sim customers, collect results into per-market buffer
  6. Wait for ALL market goroutines to finish
  7. Merge results in fixed market index order (0, 1, 2, ...)
  8. Write output files (orders, items, tweets) from merged results
  9. Write static files (products, supplies, stores)
  10. Write customers (from deduplicated map, sorted by first appearance)
```

This architecture keeps each goroutine's work independent and lock-free. The only
shared state is the read-only catalog and day effects array. The deterministic merge
in step 7 ensures that identical seeds produce byte-identical output regardless of
goroutine scheduling order.

For streaming very large datasets, the merge step can write directly to output files
rather than holding all results in memory — but the market-index ordering must still
be preserved.

### Expected Performance Comparison

| Aspect | Python | Go (estimated) |
| --- | --- | --- |
| 1 year, scale=100 | ~5-15 seconds | <100ms |
| 3 years, scale=100 | ~15-45 seconds | <300ms |
| 10 years, scale=1000 | minutes, high memory | seconds, streaming |
| Multi-GB generation | likely OOM | streaming output, bounded memory |

The major gains come from: no interpreter overhead, no per-object allocation tax,
parallel market simulation, streaming I/O, and pre-computed lookup tables. For the
target use case of generating several GBs of data, the Go version with streaming
output is the critical architectural difference — the Python version would run out of
memory before it could write.

---

## 11. Deterministic Generation

A core requirement for the Go version: **same seed = byte-identical output**. Every
run with the same `--seed` value must produce exactly the same CSVs (or JSONL, etc.),
byte-for-byte. This is critical for both data engineering use cases (reproducible test
data) and for testing the generator itself.

### Single-Seed Architecture

All randomness flows from a single seed value provided via `--seed`. The seed of 0
(default) means "pick a random seed" — the tool should print the chosen seed so the
user can reproduce the run later.

### Per-Market PRNGs

Each market gets its own PRNG, seeded deterministically from the global seed:

```go
marketRNG := rand.New(rand.NewPCG(seed+uint64(marketIndex), 0))
```

This ensures that:

- Adding/removing a market doesn't change other markets' output
- Markets can simulate in parallel without PRNG contention
- The market index (0–5) provides a stable, deterministic offset

### PRNG-Based UUIDs

All UUIDs must be generated from the market's PRNG, not from `uuid.New()` (which uses
`crypto/rand`). See the UUID Generation subsection in Go Rewrite Recommendations for
the implementation pattern.

### Deterministic Name Generation

No external randomness sources (no Faker, no `crypto/rand`). Customer names are
generated from a deterministic pool:

1. At startup, build the full cross-product of `firstNames × lastNames` from
   hardcoded name lists
2. Shuffle the pool using the market's PRNG
3. Assign names by index as customers are created

This avoids any dependency on locale data, external libraries, or system entropy.
The name lists should be large enough to avoid obvious repetition at high scale
factors (~500 first names × ~500 last names = 250,000 unique combinations).

### No External Randomness Sources

Every source of randomness in the simulation must come from `math/rand/v2` seeded
as described above. Specifically:

- Item selection (which jaffle/beverage) — from market PRNG
- Order timing (normal distribution sampling) — from market PRNG
- Buy probability rolls — from market PRNG
- Tweet probability rolls — from market PRNG
- Tweet delay (0–19 minutes) — from market PRNG
- Customer activation order (pool shuffle) — from market PRNG
- Messy mode corruption decisions — from market PRNG

### Deterministic Parallelism

Even though markets simulate in parallel via goroutines, the output must be
deterministic:

- Markets process in a fixed index order (0, 1, 2, 3, 4, 5)
- Each market's PRNG is independent and deterministically seeded
- Results are merged in fixed market index order by the main goroutine
- Within a market, customers are iterated in their original (shuffled-at-init) order
- The shuffle itself is deterministic because it uses the market's PRNG

This means goroutine scheduling order doesn't matter — the merge step imposes
a deterministic ordering on the output.

---

## 12. Random Item Selection Detail

The `Inventory.get_item_type(type, count)` method picks `count` random items of the
given type **with replacement** (each pick is independent). This means an order can
contain duplicate items (e.g., two "vanilla ice" beverages). This is realistic for
a restaurant order and should be preserved in Go.

```go
func (inv *Inventory) RandomItems(rng *rand.Rand, itemType ItemType, count int) []Item {
    pool := inv.byType[itemType]
    items := make([]Item, count)
    for i := range items {
        items[i] = pool[rng.IntN(len(pool))]
    }
    return items
}
```

---

## 13. Output File Details

### File Naming

Pattern: `{prefix}_{entity}.csv`

Default prefix is "raw", producing: `raw_customers.csv`, `raw_orders.csv`, etc.

All files written to `{output_dir}/` (default `./jaffle-data/`, created if missing).

### Row Counts (Approximate, default 3-year simulation at scale=100)

These are rough estimates since output is stochastic:

| Entity | Approximate Rows | Notes |
| --- | --- | --- |
| stores | 6 | All 6 stores open within 3 years |
| customers | ~2,000–5,000 | Subset of TAM that placed at least 1 order |
| orders | ~50,000–200,000 | Depends on persona mix, effects, and store count |
| items | ~75,000–350,000 | 1–6 items per order |
| products | 10 | Always 10 (static catalog) |
| supplies | ~120 | 29 supplies × ~4 avg SKU associations |
| tweets | ~20,000–100,000 | Subset of orders based on tweet probability |

For reference, a 1-year simulation at scale=100 produces roughly 1/5 of the above
(and only includes 2 stores: Philadelphia and Brooklyn).

### Monetary Value Handling

- Menu prices are defined in **dollars** (float in Python; use cents in Go)
- Output values are in **cents** (integer)
- Conversion: `int(float_dollars * 100)` — truncation, not rounding
- Order total is computed as `int(subtotal_cents) + int(tax_cents)` to avoid
  accumulating floating-point errors

### Date/Time Format

All timestamps in ISO 8601 format: `YYYY-MM-DDTHH:MM:SS`

The `ordered_at` field includes time (from the sampled order minute).
The `tweeted_at` field includes time (order time + 0–19 minute random delay).
The `opened_at` field for stores is date-only (midnight of the epoch + day offset).

---

## 14. Output Modes

The Go version supports three output modes, controlled by `--format` and `--messy`.

### Clean CSV Mode (default: `--format csv`)

The default behavior: 7 normalized CSV files as documented in the Output File Details
section. No changes from the current schema.

### Event Stream Mode (`--format jsonl`)

Produces a single JSONL file: `{prefix}_events.jsonl`. One JSON object per line,
ordered chronologically. Designed to mimic a real application's event log that students
would need to parse, flatten, and load into a warehouse.

**Event types:**

| Event Type | Trigger | Payload Contents |
| --- | --- | --- |
| `store_opened` | Store's opening day is reached | Store fields (id, name, tax_rate, opened_at) |
| `customer_created` | Customer's first order | Customer fields (id, name) |
| `order_placed` | Order generated | Order fields + nested items array |
| `tweet_posted` | Tweet generated | Tweet fields (id, user_id, content, tweeted_at) |

**Event schema:**

```json
{"event_type": "store_opened", "timestamp": "2018-09-01T00:00:00", "payload": {"store_id": "...", "name": "Philadelphia", "tax_rate": 0.06}}
{"event_type": "customer_created", "timestamp": "2018-09-03T07:23:00", "payload": {"customer_id": "...", "name": "Jane Smith"}}
{"event_type": "order_placed", "timestamp": "2018-09-03T07:23:00", "payload": {"order_id": "...", "customer_id": "...", "store_id": "...", "items": [{"sku": "BEV-003", "name": "vanilla ice", "price": 600}], "subtotal": 600, "tax_paid": 36, "order_total": 636}}
{"event_type": "tweet_posted", "timestamp": "2018-09-03T07:31:00", "payload": {"tweet_id": "...", "user_id": "...", "content": "Jaffles from the Jaffle Shop are awesome! Ordered a vanilla ice."}}
```

**Implementation notes:**

- Events are ordered by timestamp (chronological), not by entity type
- `store_opened` events emit when the simulation reaches a store's opening day
- `customer_created` events emit on the customer's first order (not at pool creation)
- For chronological ordering, use day-major merge: simulate all markets for day N,
  sort events within that day by timestamp, then proceed to day N+1
- Static data (products, supplies) is not emitted as events — write separate reference
  files or include a `catalog_loaded` event at timestamp 0

### Messy Mode (`--messy`)

Combinable with any output format (`--format csv --messy`, `--format jsonl --messy`).
Introduces realistic data quality issues that analytics engineering students must
detect and handle. The same `--seed` controls which rows are affected, ensuring
messy output is also deterministic.

**Data quality issues injected:**

| Issue | Frequency | Affects | Example |
| --- | --- | --- | --- |
| Null/empty customer names | ~2% of customers | `customers.csv` name field | `""` or null |
| Duplicate order rows | ~1% of orders | `orders.csv` | Same order appears twice |
| Mixed date formats | ~3% of timestamps | `ordered_at`, `tweeted_at` | `"09/03/2018 7:23 AM"` instead of ISO |
| String monetary values | ~1% of orders | `subtotal`, `tax_paid`, `order_total` | `"$6.36"` instead of `636` |
| Orphaned foreign keys | ~0.5% of orders | `customer` FK in orders | References a customer_id not in customers.csv |
| Trailing whitespace | ~2% of string fields | Any string column | `"vanilla ice "` |

**Implementation approach:**

- After generating each entity, roll a corruption check using the market PRNG
- Each corruption type has an independent probability roll
- A single row can have multiple issues (e.g., both a mixed date format and trailing
  whitespace)
- The corruption logic runs as a post-processing step on the generated entity, so
  the "clean" simulation logic remains untouched
- Document each issue type in the CLI help text or README so students know what
  categories of problems to look for (without revealing exact percentages or which
  specific rows are affected)

**Messy mode for JSONL:** The same corruptions apply to JSON field values. String
monetary values become JSON strings (`"$6.36"`) instead of integers. Null names
become JSON `null`. Mixed date formats appear in timestamp strings.

---

## 15. Simulation Pseudo-code (Complete Algorithm)

```text
FUNCTION run_simulation(years, prefix, seed, scale):
    // Setup
    epoch = 2018-09-01
    sim_days = years * 365

    // Pre-compute temporal effects
    day_effects = []
    for i in 0..sim_days:
        date = epoch + i days
        annual = (cos(day_of_year_to_radians(date)) + 1) / 10 + 0.8
        weekend = 0.6 if is_weekend(date) else 1.0
        month_offset = (date.year - 2016) * 12 + date.month
        growth = 1 + (month_offset / 12) * 0.2
        day_effects[i] = annual * weekend * growth

    // Create stores and markets, each with a deterministically-seeded PRNG
    markets = []
    for idx, store_config in enumerate(STORE_CONFIGS):
        store = new Store(store_config)
        market_rng = new PRNG(seed + idx)  // per-market PRNG
        market = new Market(store, store_config.tam, market_rng)
        // Market creates customers using its own PRNG:
        //   for each (persona, weight) in PERSONA_MIX:
        //     create int(weight * tam) customers of that persona type
        //   shuffle all customers using market_rng
        markets.append(market)

    // Main simulation loop — each market simulates independently
    // Markets can run in parallel; results merged in index order for determinism
    for each market in markets:  // or: parallel, merge in index order
        for day_idx in 0..sim_days:
            effect = day_effects[day_idx]
            date = epoch + day_idx days
            is_wknd = is_weekend(date)
            season = season_from_date(date)
            opens = 8:00 if is_wknd else 7:00
            closes = 15:00 if is_wknd else 20:00

            // Activate customers based on penetration
            days_open = day_idx - market.store.opened_day
            if days_open < 0: continue
            penetration = log_penetration_curve(days_open)
            desired_active = penetration * len(market.all_customers)
            activate (desired_active - current_active) customers

            // Simulate each active customer
            rng = market.rng  // use this market's PRNG
            for each customer in market.active_customers:
                p_season = store.popularity * effect
                p_persona = customer.persona.p_buy(is_wknd, season, fav_num)
                p_buy = sqrt(p_season * p_persona)

                if rng.float64() < p_buy:
                    // Generate order
                    order_minute = customer.persona.sample_order_time(rng)
                    if order_minute < opens_minutes or order_minute >= closes_minutes:
                        continue  // store closed at this time

                    items = customer.persona.select_items(rng, inventory)
                    order = new Order(customer, items, store, date_at_minute)
                    emit_order(order)
                    seen_customers[customer.id] = customer

                    // Maybe generate tweet
                    p_tweet = customer.persona.p_tweet()
                    if rng.float64() < p_tweet:
                        delay = rng.intn(20)
                        tweet = new Tweet(customer, order, date_at_minute + delay)
                        emit_tweet(tweet)

    // Merge results in market index order (0, 1, 2, ...) for determinism
    // Write output
    write_csv(prefix + "_stores.csv", stores)
    write_csv(prefix + "_customers.csv", seen_customers.values())
    write_csv(prefix + "_orders.csv", merged_orders)
    write_csv(prefix + "_items.csv", flatten(order.items for order in merged_orders))
    write_csv(prefix + "_products.csv", INVENTORY)
    write_csv(prefix + "_supplies.csv", STOCK)
    write_csv(prefix + "_tweets.csv", merged_tweets)
```

Note: In the streaming Go version, orders/items/tweets would be written as generated
rather than accumulated in memory. The merge step writes each market's results
sequentially in index order.

**Important**: The Python version checks the tweet probability *before* generating
the order (see Section 6). The Go version should check it *after* generating the
order — this is simpler and avoids wasting a PRNG roll when the order fails (store
closed). The trade-off is that the PRNG sequence differs from the Python version,
but since the Go version is a fresh implementation with bug fixes, exact Python
parity is not a goal.

---

## 16. Roadmap: Expanding the Schema

A real analytics engineering project doesn't have 7 tables. It has 20, 30, 50 — with
different shapes, granularities, update patterns, and levels of cleanliness. The current
jaffle-shop schema (which will be adapted post-migration for Rowing Outfitters) is excellent for teaching dbt fundamentals (staging, joins, tests,
docs), but students quickly hit a ceiling where there's not much left to model.

This section proposes a two-tier schema architecture and a prioritized list of new
entities designed to create a richer, more pedagogically useful dataset — one where
students can clean, explore, discover, and build for weeks rather than hours.

### Design Principles for New Entities

Every candidate entity should earn its place by contributing on three axes:

1. **Data variety** — Does it introduce a genuinely different *shape* of data? More
   transactional tables with the same (id, timestamp, amount) shape teach less than
   a table with a fundamentally different structure (time-series snapshots, slowly
   changing dimensions, nested/sparse fields, bridge tables). Variety of structure
   is more valuable than volume of similar data.

2. **Messy mode potential** — Can we realistically corrupt this data in ways that
   mirror actual data quality problems? The best candidates have multiple natural
   failure modes (not just null fields, but structural issues like overlapping ranges,
   impossible state transitions, or mismatched granularity).

3. **Discoverable effects** — Does it create patterns that reward exploration? For
   a 5-year fully random run, we want analysts to be able to find semi-realistic
   trends, correlations, and anomalies — things like "this store has an evening rush
   in winter" or "loyalty members have 3x higher LTV" or "promotions cannibalize
   full-price sales the following week." These don't need to be hyper-realistic. They
   should be exaggerated enough to find with a decent query or chart, simple enough
   to model in the simulation without massive complexity, and variable enough across
   random seeds that each run tells a slightly different story.

### Schema Tiers

**Core schema** — the migration target. Included by default in every run.

| Entity | Status | Notes |
| --- | --- | --- |
| stores | Existing | Keep as-is |
| customers | Existing | Keep as-is |
| orders | Existing | Keep as-is |
| items (order line items) | Existing | Keep as-is |
| products | Existing | Static catalog, keep as-is |
| supplies | Existing | Static catalog, keep as-is |

**Expanded schema** — enabled via `--expanded` flag (or similar). Adds entities that
broaden the analytical surface area. This is a post-migration milestone.

| Entity | Status | Priority | Notes |
| --- | --- | --- | --- |
| tweets | Move from core | — | Social/review data, already exists |
| payments | New | High | Payment method diversity, split tenders |
| promotions + order_promotions | New | High | SCD-like dimension + bridge table |
| staff + shifts | New | High | Completely different data shape (schedule data) |
| inventory_movements | New | Medium | Time-series supply chain data |
| customer_loyalty | New | Medium | Cohort/retention analysis |
| store_expenses | New | Low | Monthly aggregates, profitability analysis |

Moving tweets out of core is deliberate: the core schema becomes a clean, normalized
transactional dataset. Expanded mode adds the messier, more varied data that represents
the reality of a growing business generating data across multiple systems.

### Proposed New Entities

#### Payments (High Priority)

**What it is:** One row per payment applied to an order. Most orders have a single
payment, but some have split tenders (e.g., part gift card, part credit card).

**Schema (`payments.csv`):**

| Column | Type | Notes |
| --- | --- | --- |
| payment_id | UUID | |
| order_id | UUID | FK to orders |
| payment_method | string | "credit_card", "debit_card", "cash", "gift_card", "mobile_pay" |
| amount | int | Cents |
| payment_timestamp | datetime | Usually same as order time, occasional slight delay |

**Why it matters:**

- *Data variety:* Introduces a one-to-many relationship below orders (orders → items
  is already one-to-many, but payments add a *second* child table with completely
  different semantics — financial reconciliation vs. menu composition). Students
  must correctly join payments to orders without double-counting.
- *Messy mode:* Payments that don't sum to order total (~1%). Duplicate payment
  records (~0.5%). `payment_method` with inconsistent casing ("Credit_Card" vs
  "credit_card", ~2%). Null payment methods (~1%).
- *Discoverable effects:*
  - **Mobile payment adoption curve** — mobile_pay starts at 0% of transactions in
    2018 and grows to ~30% by 2023, following a logistic S-curve. Cash declines
    inversely. Students can build a payment method trend analysis and see the shift.
  - **Gift card seasonality** — gift card payments spike in January (holiday gifts
    being redeemed) and again in late December (people buying for themselves with
    new gift cards). A simple seasonal curve on gift_card probability.
  - **Persona payment preferences** — Commuters skew heavily toward mobile_pay
    (speed matters). BrunchCrowd uses more credit cards. Students use more
    debit/cash. Discoverable by joining payments → orders → customers and segmenting.

**Simulation complexity:** Low. After generating an order, roll payment method from
a persona-weighted probability table adjusted by a time-based mobile adoption curve.
Split tenders are rare (~5% of orders) and just split the total into two payments.

---

#### Promotions + Order Promotions (High Priority)

**What it is:** Two tables. `promotions` is a slowly-changing catalog of promotional
offers. `order_promotions` is a bridge table linking orders to the promotions that
were applied.

**Schema (`promotions.csv`):**

| Column | Type | Notes |
| --- | --- | --- |
| promotion_id | UUID | |
| name | string | e.g., "Summer Sip Fest", "Back to School 20%" |
| type | string | "percentage_off", "bogo", "free_item", "flat_discount" |
| value | int | Cents (flat discount) or basis points (percentage, e.g., 2000 = 20%) |
| applies_to | string | "all", "jaffles", "beverages", or a specific SKU |
| start_date | date | |
| end_date | date | |
| min_order_amount | int | Cents, 0 if no minimum |

**Schema (`order_promotions.csv`):**

| Column | Type | Notes |
| --- | --- | --- |
| order_id | UUID | FK to orders |
| promotion_id | UUID | FK to promotions |
| discount_amount | int | Cents actually deducted |

**Why it matters:**

- *Data variety:* Promotions are an SCD-like structure — they have validity windows,
  and the relationship to orders is a many-to-many bridge. This is a fundamentally
  different modeling challenge from the core transactional tables. Students need to
  handle date-range logic, bridge table fan-out, and conditional application rules.
- *Messy mode:* Promotions applied outside their valid date range (~2%). Discount
  amounts that don't match the promo rules (~1%, e.g., 15% discount applied but
  promo says 20%). Overlapping promotions with ambiguous stacking (~1%). Promo names
  with trailing whitespace or inconsistent capitalization.
- *Discoverable effects:*
  - **Post-promotion dip** — after a popular promotion ends, order volume drops below
    the pre-promo baseline for 1–2 weeks before recovering. This "pull-forward" effect
    is a classic retail phenomenon that students can find by comparing weekly order
    counts around promo boundaries.
  - **Promo effectiveness by store** — some stores respond more strongly to promotions
    (higher-popularity stores see bigger lifts). Students can calculate lift % by store.
  - **Seasonal promo calendar** — promotions follow a realistic pattern: summer drink
    specials, fall jaffle combos, holiday BOGO deals. The promo calendar repeats
    annually with slight variations, creating year-over-year comparables.

**Simulation complexity:** Medium. Pre-generate a promo calendar at startup (8–12
promos per year, with seasonal alignment). During order generation, check if any active
promo applies and roll a probability for whether the customer uses it (persona-driven:
Students and Casuals are more promo-sensitive, Commuters less so).

---

#### Staff + Shifts (High Priority)

**What it is:** Two tables. `staff` is an employee dimension. `shifts` is a schedule
fact table with clock-in/clock-out times.

**Schema (`staff.csv`):**

| Column | Type | Notes |
| --- | --- | --- |
| staff_id | UUID | |
| store_id | UUID | FK to stores |
| name | string | |
| role | string | "barista", "cook", "shift_lead", "manager" |
| hired_date | date | |
| terminated_date | date | Null if still active |

**Schema (`shifts.csv`):**

| Column | Type | Notes |
| --- | --- | --- |
| shift_id | UUID | |
| staff_id | UUID | FK to staff |
| store_id | UUID | FK to stores |
| clock_in | datetime | |
| clock_out | datetime | Null if shift still open (messy mode) |
| scheduled_start | datetime | The planned start time |
| scheduled_end | datetime | The planned end time |
| role_during_shift | string | May differ from staff.role (covering for someone) |

**Why it matters:**

- *Data variety:* This is a completely different data shape — interval/schedule data
  rather than point-in-time transactions. Students must reason about overlapping time
  ranges, duration calculations, scheduled-vs-actual variance, and join orders to
  shifts via time-range predicates (not simple FK joins). This is one of the most
  common and challenging patterns in real warehouse work.
- *Messy mode:* Missing clock-out records (~3%, shift appears "still open" from weeks
  ago). Clock-in before scheduled start by hours (~1%, data entry error). Overlapping
  shifts for the same employee (~0.5%). `terminated_date` before last shift worked
  (~0.5%). Role mismatches between staff.role and shifts.role_during_shift that aren't
  legitimate covers (~1%).
- *Discoverable effects:*
  - **Turnover patterns** — newer stores have higher staff turnover in their first
    year (realistic for restaurant openings). Baristas turn over faster than managers.
    Students can calculate tenure distributions and turnover rates by store/role.
  - **Understaffing correlation** — days with fewer shifts scheduled correlate with
    slightly lower order counts (customers see a long line and leave). This is a
    subtle effect that requires joining shifts to orders by store and date.
  - **Schedule adherence** — the gap between `scheduled_start` and `clock_in` varies
    by role and day of week. Managers are more punctual. Weekend shifts have more
    late arrivals. Students can build a schedule adherence dashboard.

**Simulation complexity:** Medium. Each store maintains a staff pool (hired relative
to store opening). Generate a weekly schedule template, then add per-shift noise for
actual clock-in/clock-out times. Staff hire/terminate events follow a turnover curve
(higher in year 1, stabilizing later).

---

#### Inventory Movements (Medium Priority)

**What it is:** A time-series of supply deliveries and consumption events per store.
Transforms the static `supplies` table into a dynamic system.

**Schema (`inventory_movements.csv`):**

| Column | Type | Notes |
| --- | --- | --- |
| movement_id | UUID | |
| store_id | UUID | FK to stores |
| supply_id | string | FK to supplies |
| movement_type | string | "delivery", "consumption", "waste", "adjustment" |
| quantity | int | Positive for inbound, negative for outbound |
| recorded_at | datetime | |
| unit_cost | int | Cents, cost at time of movement |

**Why it matters:**

- *Data variety:* Running-balance / event-sourced data. Students need to compute stock
  levels by aggregating movements over time — a cumulative sum pattern that's common
  in real warehouses but absent from the current schema. The `waste` and `adjustment`
  types add nuance beyond simple in/out.
- *Messy mode:* Negative running balances (~2% of store-supply combinations hit
  negative briefly due to consumption recorded before delivery). Missing delivery
  records (~1%, consumption happens but no corresponding delivery). Unit costs that
  don't match the supplies table (~2%, price changes over time or data entry errors).
- *Discoverable effects:*
  - **Perishable waste cycles** — perishable supplies have higher waste rates in
    slow periods (summer for stores that lose Student traffic). Students can
    correlate waste events with seasonal order volume.
  - **Delivery frequency patterns** — stores receive deliveries weekly, but the
    day varies by store. High-volume stores get mid-week top-ups. Discoverable
    by aggregating delivery counts by day-of-week and store.
  - **Cost trend over time** — `unit_cost` for certain supplies drifts upward (~3%
    annually, with occasional jumps). Students can track ingredient cost inflation
    and its impact on margins.

**Simulation complexity:** Medium-High. Requires tracking per-store supply levels and
generating delivery events when stock drops below a reorder threshold. Consumption is
derived from orders (each order consumes its items' supplies). Waste events are
periodic for perishables.

---

#### Customer Loyalty Program (Medium Priority)

**What it is:** A loyalty program that a subset of customers enroll in over time.
Two tables: accounts and point transactions.

**Schema (`loyalty_accounts.csv`):**

| Column | Type | Notes |
| --- | --- | --- |
| loyalty_id | UUID | |
| customer_id | UUID | FK to customers |
| enrolled_at | datetime | |
| tier | string | "bronze", "silver", "gold" — computed from lifetime points |

**Schema (`loyalty_transactions.csv`):**

| Column | Type | Notes |
| --- | --- | --- |
| transaction_id | UUID | |
| loyalty_id | UUID | FK to loyalty_accounts |
| order_id | UUID | FK to orders (null for redemptions, adjustments) |
| points_change | int | Positive = earned, negative = redeemed |
| balance_after | int | Running balance snapshot |
| transaction_type | string | "earn", "redeem", "bonus", "expire" |
| recorded_at | datetime | |

**Why it matters:**

- *Data variety:* Account-level data with state (tier) that changes over time based
  on accumulated behavior — a natural SCD Type 2 candidate. The point transactions
  are an event-sourced ledger. Students get to practice both snapshot-based and
  event-based modeling patterns.
- *Messy mode:* `balance_after` that doesn't match the running sum of `points_change`
  (~2%). Duplicate enrollments for the same customer (~0.5%). Tier that doesn't match
  actual point totals (~1%). Points expiring but balance not decremented (~1%).
- *Discoverable effects:*
  - **Loyalty lift** — enrolled customers spend ~15-25% more after joining (partially
    real behavior change, partially selection bias from already-frequent customers
    enrolling). Students can attempt to measure this and grapple with the selection
    bias problem — a great analytical thinking exercise.
  - **Tier progression** — gold-tier customers formed in year 1 have different
    retention patterns than those who reach gold later. Cohort analysis opportunity.
  - **Redemption patterns** — customers hoard points and then redeem in bursts,
    creating lumpy revenue impact. Students can build a points liability model.
  - **Enrollment adoption curve** — loyalty enrollment follows an S-curve per store,
    starting at 0% and plateauing at ~40-60% of active customers. Newer stores see
    faster adoption (the program is more established when they open).

**Simulation complexity:** Low-Medium. On each order, check if the customer is enrolled
(probability increases with order count). If enrolled, earn points. Periodically roll
for redemption. Tier is computed from cumulative points. Point expiry is calendar-based
(points older than 12 months expire monthly).

---

#### Store Expenses (Low Priority)

**What it is:** Monthly operational cost records per store.

**Schema (`store_expenses.csv`):**

| Column | Type | Notes |
| --- | --- | --- |
| expense_id | UUID | |
| store_id | UUID | FK to stores |
| month | date | First of month |
| category | string | "rent", "utilities", "labor", "marketing", "maintenance" |
| amount | int | Cents |

**Why it matters:**

- *Data variety:* Monthly-grain aggregate data — a different granularity from the
  daily/transactional core. Students must handle grain mismatches when joining
  expenses to order revenue for profitability analysis.
- *Messy mode:* Miscategorized expenses (~3%). Missing months (~1%). Duplicate
  entries for the same store-month-category (~1%).
- *Discoverable effects:*
  - **Seasonal utility costs** — heating costs spike in winter for northern stores
    (Philadelphia, Brooklyn, Chicago), not for southern (New Orleans, Los Angeles).
  - **New store ramp costs** — stores have elevated marketing and maintenance costs
    in their first 6 months, then normalize. Combined with the revenue ramp from
    market penetration, this creates a clear "path to profitability" curve per store.
  - **Labor cost scaling** — labor expenses correlate with order volume but with a
    floor (minimum staffing). Students can calculate labor cost per order and see
    how it decreases with scale.

**Simulation complexity:** Low. Generate monthly records from templates with per-city
base costs, seasonal adjustments, and volume-linked components. No interaction with
the daily simulation loop.

### Effects Worth Engineering Into the Simulation

The expanded entities above create opportunities for discoverable effects, but some
of the most interesting patterns come from interactions *between* entities and the
existing temporal/persona systems. These effects should be tunable in intensity
(stronger in some random seeds than others) to keep each run's story fresh.

**Cross-entity effects to prioritize:**

| Effect | Entities Involved | What Students Find |
| --- | --- | --- |
| Evening rush before closing | orders × curves | Philadelphia and Brooklyn show a second order peak 30-60 min before weekday closing in winter — Commuters grabbing dinner on the way home. Caused by a small secondary order-time mode in the Commuter persona, only active in WINTER/FALL. |
| Persona lifetime value divergence | orders × customers | RemoteWorkers have highest LTV despite lower per-order spend because they buy consistently over years. BrunchCrowd has high per-order spend but lower frequency. Students can build a CLV model and find that "big spenders" aren't the most valuable segment. |
| Promo cannibalization | orders × promotions | Orders spike during promos but dip afterward. Net effect over a 4-week window is only +5-10% incremental (not the +30% the promo-week lift suggests). Students learn that naive promo analysis overstates impact. |
| Loyalty selection bias | orders × loyalty | "Loyalty members spend 20% more" is true in the data but misleading — high-frequency customers are more likely to enroll. A proper diff-in-diff or matching analysis shows the real lift is ~8%. |
| Staff-order correlation | shifts × orders | Days with below-average staffing have ~5-10% fewer orders (some customers balk at long lines). This is a confounded effect — slow days might just have fewer staff *scheduled*. Students need to think about causality. |
| New store profitability curve | orders × expenses | Stores lose money in their first 3-6 months (high ramp costs, low penetration), break even around month 8-10, and reach steady-state profitability by month 14-18. Students can build a store P&L and identify the pattern. |
| Menu item seasonality | items × curves | Iced beverages (BEV-003 vanilla ice, BEV-001 tangaroo) are 2-3x more popular in summer. Hot beverages (BEV-002 chai, BEV-004 pourover) peak in winter. Students can build a product mix dashboard and see seasonal shifts. |
| Payment method shift | payments × time | Cash usage halves over a 5-year run while mobile_pay goes from 0% to 30%. The crossover point varies by city (SF adopts faster than New Orleans). Students can build a trends analysis and forecast when cash drops below 10%. |

These effects don't need to be individually toggled. They emerge naturally from
the simulation mechanics — persona behavior, temporal curves, and entity interactions.
The key implementation principle is: **keep each effect's simulation logic simple
(one or two probability adjustments), but let the effects compound when entities
are joined together.** The analytical richness comes from the *combination* of simple
mechanisms, not from any single complex one.

### Simulation Complexity Budget

Adding entities must not turn the generator into an enterprise simulation engine.
Guardrails:

- **Core simulation loop stays simple.** The day → market → customer → order loop
  is the heart of the tool. New entities should attach to this loop as lightweight
  post-processing steps (generate an order → then generate its payment, check promo
  applicability, log supply consumption), not as new simulation dimensions that
  require their own state machines.
- **Static where possible.** Store expenses and the promo calendar can be pre-generated
  at startup from templates + seasonal curves — they don't need to participate in the
  per-customer daily loop at all.
- **Expanded entities are optional.** The `--expanded` flag gates all of this. The
  core schema remains fast and simple. A student's first week uses core mode; they
  graduate to expanded mode when they're ready for more.
- **Complexity ceiling: ~12 output tables total.** Core (6) + expanded (up to 6 new
  - tweets moved over) keeps the project ambitious but bounded. Going beyond ~12
  tables adds diminishing pedagogical returns and significant simulation complexity.

### Implementation Priority

For the post-migration expanded schema milestone:

1. **Phase 1** — Payments + Promotions. These are the highest-value additions: they
   introduce new data shapes (bridge table, SCD-like validity windows), create the
   most interesting discoverable effects (payment trends, promo cannibalization), and
   have low-medium simulation complexity. Ship together so students immediately get a
   richer analytical surface.

2. **Phase 2** — Staff + Shifts. The highest data-variety addition (interval/schedule
   data). More simulation complexity than payments/promos, but the analytical payoff
   is worth it — time-range joins are a critical real-world skill that the current
   schema can't teach.

3. **Phase 3** — Customer Loyalty. Builds on the existing customer/order foundation
   to add account-state modeling and cohort analysis opportunities. The selection bias
   effect alone makes this worthwhile as a teaching tool.

4. **Phase 4** (stretch) — Inventory Movements and Store Expenses. These round out
   the supply chain and financial analysis angles. Inventory movements have the highest
   simulation complexity of any proposed entity (requires tracking running state), so
   they're deferred until the foundation is solid.
