# Themes

`rowing-machine themes` lists bundled themes with descriptions and compatible scenarios. Bundled themes are compiled into the binary and use no real brand names:

| Theme | Scenarios | Skin |
| --- | --- | --- |
| `plain` | ecommerce, saas | Neutral business vocabulary; the default for both |
| `fantasy_rpg` | ecommerce | The Arcanum Collective mage guild |
| `sneakers` | ecommerce | Starcloud Sneakers, a running shoe brand |
| `airline` | travel | SuperAir, a low-cost airline with six UK bases; the travel default |

A TOML path, such as `--theme ./shop.toml`, loads a custom theme; branded variants for a specific company belong in files like that.

A theme is the skin over a scenario: names, labels, catalogs, and parameter values. Entity and column names always come from the scenario. Between two themes with the same catalogs and parameters, switching changes only names and labels. Product SKUs and supply IDs remain stable identifiers, including their fantasy prefixes under `plain`.

## File schema

A theme declares `name`, `description`, a `names` table of generators, and a `labels` table of ordered lists. This excerpt illustrates a generator; a usable ecommerce theme also needs all the label sets below. Copy a bundled TOML file as a starting point.

```toml
name = "shop"
description = "Names for a neighborhood shop."

[names.person]
formats = [
    { format = "{given} {family}", weight = 3 },
    { format = "{given} {middle} {family}", weight = 1 },
]

[names.person.components]
given = ["Ada", "Grace"]
middle = ["River", "Sage"]
family = ["Meadow", "Stone"]

[labels]
ranks = ["member", "regular", "supporter", "ambassador"]
```

Formats contain literal text and `{component}` references. Each reference expands over its declared pool of whole tokens; repeated references expand independently. A positive integer `weight` defaults to `1`. Malformed references, unknown components, empty required pools or tokens, blank names, and zero weights are errors. Duplicate full names from overlapping formats or pools count as one combination.

The loader rejects invalid TOML, unknown schema fields, empty labels, missing scenario entries, and incorrect label lengths before generation. Errors identify the source and field; compatibility errors also list suitable bundled themes.

## Parameters

Scenarios declare parameters with a default and an inclusive range. A theme sets them in a section named for the scenario, and `--param name=value` overrides one for a run:

```toml
[params.ecommerce]
price_scale = 13.0
```

Parameters a theme leaves out take the scenario default. Sections for other scenarios are ignored, so one file can carry values for each scenario it covers. An undeclared name or out-of-range value fails before generation. Ecommerce declares `price_scale` (default 1.0, 0.01–1000), which multiplies product prices and supply costs, and `purchase_rate` (default 1.0, 0.0001–1), which multiplies each customer's daily chance of ordering so considered purchases like shoes stay rare; [the travel model](travel.md#parameters) lists travel's parameters.

## Catalogs

Catalogs are ordered lists of records with the typed fields a scenario declares. Fields are strings, numbers, or booleans, and records may not carry undeclared fields:

```toml
[[catalogs.locations]]
name = "Harbor City"
code = "HBR"
latitude = 51.5
longitude = -0.1
base = true
kind = "city"
weight = 1.2
```

A missing catalog, too few records, or a missing, mistyped, or unknown field fails before generation with the record's index.

## Assignment and exhaustion

For each format, its weight is divided among its distinct full-name combinations. Contributions from overlapping formats add together. A dedicated seeded stream produces a weighted permutation of the distinct names. Weights favor earlier assignment; every combination still appears once before any repeats.

Assignment depends only on seed, name kind, and entity index. Ecommerce assigns contiguous indices to customers who placed orders, in market and customer order, after the order stage completes. Non-ordering customers leave no gaps in the name sequence. SaaS assigns contiguous organization indices to arrived accounts and person indices across their lifecycles. After exhausting the unique combinations, assignment repeats the same permutation. `plain` has 126,242 person combinations and 5,120 organizations, enough for its default scenarios. `fantasy_rpg` has 7,047 person combinations and `sneakers` 12,549, exceeding ecommerce's default population of 6,200. Travel assigns contiguous indices to travellers who booked, in UUID order, after the booking stage completes; `airline` has 185,878 person combinations for its default pool of 180,000. Larger runs may reuse names after that capacity.

The generator materializes distinct combinations and caches permutations by seed and kind. Memory and startup work therefore grow with the number of combinations in the pack. Customer names and sparrow wording use streams separate from simulation decisions.

## Ecommerce label sets

Ecommerce requires the `person` name kind. Fixed catalog names use ordered labels so store openings, product properties, and supply origins keep their indexed relationships.

| Label set | Length | Meaning |
| --- | --- | --- |
| `stores` | 6 | Store names in opening order; also supply origin labels |
| `products` | 15 | Product names in SKU order |
| `product_descriptions` | 15 | Descriptions in the same product order |
| `product_types` | 3 | Categories for each block of five products |
| `power_levels` | 5 | Tier labels within each product block |
| `supplies` | 41 | Supply names in ID order |
| `ranks` | 4 | Customer cohorts from least to most frequent ordering |
| `rank_voices` | 4 | Message prefixes in rank order |
| `positive_adjectives` | 7 | Positive message descriptions |
| `negative_adjectives` | 7 | Negative message descriptions |
| `neutral_adjectives` | 8 | Neutral message descriptions |
| `sparrow_templates` | 3 | Positive, negative, and neutral message templates |
| `acquired_templates` | 3 | Templates for one, two, and three-or-more products |
| `item_separator` | 1 | Separator between products in longer lists |

Sparrow templates substitute `{adjective}` and `{acquired}`. Acquisition templates substitute `{one}` and `{two}`; for three or more products, `{one}` contains all but the last product, joined with `item_separator`. Product mentions come from the selected theme's `products` labels. The message starts with its rank voice followed by a colon and space.

## SaaS names and labels

SaaS requires `person`, `organization`, `plan`, `feature`, and `campaign` generators. `plain` covers these kinds. Feature and campaign vocabulary is reserved for product usage and marketing entities.

| Label set | Length | Meaning |
| --- | --- | --- |
| `industries` | 6 | Account industries |
| `roles` | 3 | User roles |
| `regions` | 4 | Account regions |
| `plan_tiers` | 3 | Plan tiers, lowest to highest |

User emails combine generated person and organization slugs under the reserved `.example` domain. Billing intervals, employee bands, movement types, and subscription statuses are scenario values shared by every theme.

## Travel names, labels, and catalogs

Travel requires `person` and `vehicle` generators, and these label sets and catalogs. `airline` covers them.

| Entry | Shape | Meaning |
| --- | --- | --- |
| `labels.ranks` | 4 values | Loyalty tiers from least to most frequent traveller |
| `labels.trip_prefix` | 1 value | Prefix for trip codes, such as `SA` |
| `catalogs.locations` | `name`, `code`, `latitude`, `longitude`, `base`, `kind`, `weight`; at least 2 | Places the network links; bases station vehicles, `kind` is `city`, `beach`, or `ski` |
| `catalogs.vehicle_types` | `name`, `capacity`, `share`; at least 1 | Fleet mix; capacity is whole seats |
| `catalogs.add_ons` | `name`, `category`, `price`, `attach_rate`; at least 1 | Extras bought per ticket; price in whole cents, attach rate 0–1 |
| `catalogs.fare_classes` | `name`, `multiplier`, `share`; at least 1 | Fare bundles; the multiplier applies to the base fare |
| `catalogs.channels` | `name`, `share`; at least 1 | Where bookings are made |

Location codes must be unique, and at least one location must be a base.
