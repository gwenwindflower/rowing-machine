# Simulation logic reference

The [ecommerce scenario](../src/scenario/ecommerce/mod.rs) generates orders and messages using [persona behavior](../src/scenario/ecommerce/persona.rs), [calendar curves](../src/engine/calendar.rs), and [indexed streams](../src/engine/stream.rs).

## Core formula

```text
p_buy = sqrt(p_buy_season * p_buy_persona)
p_buy_season = store.base_popularity * day_effect
day_effect = annual_curve * weekend_curve * growth_curve
```

## Curves

**Annual**: `(cos(x) + 1) / 10 + 0.8` where x = day-of-year mapped to [0, 2*pi]. Range [0.8, 1.0].

**Weekend**: weekday = 1.0, weekend = 0.6.

**Growth**: `1 + (month_offset / 12) * 0.2` where month_offset = `(year - 2016) * 12 + month`.

## Market penetration

Single smooth logarithmic curve:

```text
pct = min(days_since_open / 365, 1)
penetration = min(ln(1 + pct * (e - 1)), 1)
```

This gives: day 0 = 0%, day 30 ~14%, day 180 ~62%, day 365 = 100%.

## Hours of operation

| Day type | Opens | Closes |
| --- | --- | --- |
| Weekday | 07:00 (420 min) | 20:00 (1200 min) |
| Weekend | 08:00 (480 min) | 15:00 (900 min) |

## Seasons

| Season | Range |
| --- | --- |
| WINTER | Jan 1 – Mar 20, Dec 21 – Dec 31 |
| SPRING | Mar 21 – Jun 20 |
| SUMMER | Jun 21 – Sep 20 |
| FALL | Sep 21 – Dec 20 |

## Epoch

2023-01-01 by default. Day index 0 = `--start-date`.

## Order generation flow

1. Roll `p_buy` — if miss, skip
2. Sample order minute from persona's normal distribution (clamp >= 0)
3. Check store hours — if closed, discard
4. Select items per persona rules
5. Create order (subtotal, tax, total — all in cents)
6. Roll `p_sparrow` — if hit, create sparrow with 0-19 min delay

## Guild rank cohorts

Customer pools contain exactly the guild hall's TAM base times scale. Each customer's UUID, favorite number, fan level, and activation threshold come from a stream indexed by market and customer. Personas occupy shuffled blocks of 20 customers: five Couriers, five Artificers, two FeastRevelers, four Apprentices, two Wanderers, and two Herbalists. Each block's shuffle uses its own seed/market/block stream, so pool expansion preserves existing customers and a partial final block still yields exactly the requested pool size. Activation is the first day the penetration curve reaches that customer's threshold. Customer names use the hall name and customer index, such as `Thornwall patron 1`.

Ordering customers are sorted by lifetime order count and divided into four near-equal cohorts: initiate, journeyman, adept, and master. Customer UUID breaks equal-order-count ties deterministically. This keeps the rank distribution balanced across simulation durations and scales while preserving higher ranks for customers with greater order frequency.

The scenario emits catalogs first, then orders and items in day/market order. It counts emitted orders to assign ranks, regenerates each market-day from the same indexed streams to emit sparrows with final ranks, and emits only ordering customers in market/customer-index order. It retains customer pools and counts rather than the run's order rows.

## Sparrow content

Template chosen by fan_level (1-5):

- fan_level > 3: positive adjective + items sentence
- fan_level < 3: negative adjective + items sentence
- fan_level == 3: neutral adjective + items sentence

Items sentence: "Acquired a {item1}" / "Acquired a {item1} and a {item2}" / "Acquired a {item1}, a {item2}, ..., and a {itemN}"

Templates:

- "Wares from the Arcanum Collective are {adj}!"
- "Arcanum Collective again. {items}. Their craft is {adj}."
- "The Arcanum Collective is {adj}. {items}."

Each message starts with rank-specific vocabulary: `A novice's discovery` for initiates, `A practiced hand's report` for journeymen, `An adept's appraisal` for adepts, and `A master's verdict` for masters.
