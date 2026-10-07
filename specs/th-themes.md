# Themes

## Goals

A theme is the skin over a scenario. Scenarios own the skeleton: generic entities, relationships, and the logic that turns parameters into rows. Themes give that skeleton meaning: table and column names, generators for the things scenarios name (people, organizations, vehicles, products, features, plans, campaigns), the labels scenarios attach to rows, catalogs, and parameter values. A theme works with a scenario when it covers every declaration that scenario makes, so a narrow theme fits few scenarios and a broad one fits many; no theme has to support every scenario.

## Vocabulary

- **Name kind** — a category of generated name that a scenario declares, such as `person`, `organization`, `location`, `product`, `feature`, `plan`, or `campaign`.
- **Label set** — a fixed, ordered list of categorical values a scenario declares with a required length, such as five rarity tiers or four customer ranks.

## Requirements

### Theme contract

- **th-R009** — A theme is one TOML file holding its name, a description, and the generators, label sets, catalogs, parameter values, and table and column names it supplies.
- **th-R017** — A theme is compatible with a scenario exactly when it provides every name kind, label set, and catalog that scenario declares.
- **th-R018** — If the selected theme is not compatible with the selected scenario, then the run fails before simulation, naming the missing name kinds, label sets, and catalogs and listing the compatible themes.
- **th-R010** — If a theme file fails to parse or gives a label set the wrong length, then it is rejected before simulation with an error naming the file and the invalid entry.
- **th-R011** — Between two themes with the same catalogs, parameter values, and table and column names, switching changes only generated names and label values; every ID, timestamp, count, and numeric value stays the same.
- **th-R012** — Bundled themes are compiled into the binary, so a release binary needs no theme files on disk.
- **th-R013** — Bundled themes cover every scenario: `plain` covers `ecommerce` and `saas`, `fantasy_rpg` (the Arcanum Collective) and `sneakers` (Starcloud Sneakers) cover `ecommerce`, and `airline` (SuperAir) covers `travel`.

### Names

- **th-R001** — Every generated name and label comes only from the selected theme's declared generators and label sets.
- **th-R002** — Name generation uses native code and checked-in, reviewed data; it never executes or loads third-party generator code or data at runtime.
- **th-R005** — A run never repeats a full name within one name kind until that kind's valid combinations are exhausted, across all markets and workers.
- **th-R006** — Every bundled theme has enough combinations in each name kind to name the default run's population for every scenario it is compatible with, without repetition.
- **th-R007** — Every generated name traces to a declared format and whole-token components; the generator never synthesizes characters within a component.
- **th-R008** — If a name generator has an invalid format, an unknown component reference, an empty required pool, or no valid combinations, then the theme is rejected before simulation with an error naming the theme and field.
- **th-R015** — Name assignment is a pure function of the seed, the name kind, and the entity's index, so worker scheduling never changes which entity gets which name.
- **th-R016** — Derived text such as emails and landing page paths is built from the entity's generated names, so a user's email matches their name and their account's name.

### Catalogs and parameters

- **th-R019** — A theme sets a scenario's parameters under `[params.<scenario>]`; parameters it leaves out take the scenario's defaults, and sections for other scenarios are ignored.
- **th-R020** — If a theme sets a parameter its scenario does not declare, or a value outside the declared range, then the run fails before simulation naming `params.<scenario>.<name>` and the valid choices or range.
- **th-R021** — If a catalog is missing, shorter than its minimum, or has a record with a missing, mistyped, or undeclared field, then the run fails before simulation naming `catalogs.<name>[<index>].<field>`.
- **th-R022** — Bundled themes contain no real brand names; branded variants are private theme files loaded by path.
- **th-R023** — A theme renames a scenario's tables and columns under `[schema.<scenario>]`, and every format writes the renamed tables, columns, and primary keys.
- **th-R024** — If a rename targets an unknown table or column, is not a lowercase identifier, or collides with another name in its table, then the run fails before output naming `schema.<scenario>` and the entry.

Retired: th-R003, th-R004, th-R014.
