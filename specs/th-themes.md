# Themes

## Goals

A theme is a naming pack. It supplies generators for the things scenarios name: people, organizations, locations, products, features, plans, campaigns, and the categorical labels scenarios attach to rows (product types, rarity tiers, customer ranks, plan tiers, industries). Scenarios own everything else: entities, columns, row counts, prices, rates, and every other number. A theme works with a scenario when it covers every name kind and label set that scenario declares, so a narrow theme fits few scenarios and a broad one fits many; no theme has to support every scenario. Themes are declarative TOML, bundled into the binary or loaded from a path, and names come from reviewed whole-token component pools, never third-party generators or character-level synthesis.

## Vocabulary

- **Name kind** — a category of generated name that a scenario declares, such as `person`, `organization`, `location`, `product`, `feature`, `plan`, or `campaign`.
- **Label set** — a fixed, ordered list of categorical values a scenario declares with a required length, such as five rarity tiers or four customer ranks.

## Requirements

### Theme contract

- **th-R009** — A theme is one TOML file declaring its name, a description, a generator for each name kind, and a value list for each label set.
- **th-R017** — A theme is compatible with a scenario exactly when it provides every name kind and label set that scenario declares.
- **th-R018** — If the selected theme is not compatible with the selected scenario, then the run fails before simulation, naming the missing name kinds and label sets and listing the compatible themes.
- **th-R010** — If a theme file fails to parse or gives a label set the wrong length, then it is rejected before simulation with an error naming the file and the invalid entry.
- **th-R011** — Switching themes changes only generated names and label values; every ID, timestamp, count, and numeric value in the output stays the same.
- **th-R012** — Bundled themes are compiled into the binary, so a release binary needs no theme files on disk.
- **th-R013** — `plain` (neutral business vocabulary) ships covering every scenario, and `fantasy_rpg` (the Arcanum Collective) ships covering `ecommerce`.
- **th-R014** — Entity names and column names come from the scenario, never from the theme, so queries written against one theme run unchanged against another.

### Names

- **th-R001** — Every generated name and label comes only from the selected theme's declared generators and label sets.
- **th-R002** — Name generation uses native code and checked-in, reviewed data; it never executes or loads third-party generator code or data at runtime.
- **th-R005** — A run never repeats a full name within one name kind until that kind's valid combinations are exhausted, across all markets and workers.
- **th-R006** — Every bundled theme has enough combinations in each name kind to name the default run's population for every scenario it is compatible with, without repetition.
- **th-R007** — Every generated name traces to a declared format and whole-token components; the generator never synthesizes characters within a component.
- **th-R008** — If a name generator has an invalid format, an unknown component reference, an empty required pool, or no valid combinations, then the theme is rejected before simulation with an error naming the theme and field.
- **th-R015** — Name assignment is a pure function of the seed, the name kind, and the entity's index, so worker scheduling never changes which entity gets which name.
- **th-R016** — Derived text such as emails and landing page paths is built from the entity's generated names, so a user's email matches their name and their account's name.

Retired: th-R003, th-R004.
