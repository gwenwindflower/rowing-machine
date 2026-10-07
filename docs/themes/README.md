# Themes

A theme is one TOML file that skins one or more scenarios. The scenario is the skeleton: generic entities, the keys between them, and parameters. The theme supplies everything else: table and column names (`[schema.<scenario>]`), name generators, label sets, catalogs, and parameter values (`[params.<scenario>]`). [Scenarios and themes](../scenarios/README.md) explains how the two fit together.

## Bundled themes

| Theme | Scenarios | World | Page |
| --- | --- | --- | --- |
| `plain` | ecommerce, saas | Market Collective, a neutral six-store shop, and a generic B2B software company | [plain](plain.md) |
| `fantasy_rpg` | ecommerce | The Arcanum Collective, a mage guild selling enchanted wares; the data factory for Queria, a retro-RPG SQL trainer | [fantasy_rpg](fantasy_rpg.md) |
| `sneakers` | ecommerce | Starcloud Sneakers, a Swiss running shoe brand with six global flagships | [sneakers](sneakers.md) |
| `airline` | travel | SuperAir, a low-cost airline flying European routes from six UK bases | [airline](airline.md) |

Bundled themes are compiled into the binary; `rowing-machine themes` lists them. `--theme ./shop.toml` loads a custom theme; branded variants for a real company belong in files like that, since bundled themes use no real brand names. Copy a bundled file from [`themes/`](../../themes/) as a starting point.

## File layout

| Key | Holds | Read by |
| --- | --- | --- |
| `name`, `description` | Non-empty strings | `rowing-machine themes` and error messages |
| `[names.<kind>]` | A name generator per name kind | Scenarios that declare the kind |
| `[labels]` | Ordered string lists of exact lengths | Scenarios that declare the set |
| `[[catalogs.<name>]]` | Ordered records with typed fields | Scenarios that declare the catalog |
| `[params.<scenario>]` | Numbers within declared ranges | That scenario only |
| `[schema.<scenario>]` | Table and column renames | That scenario only |

One file can cover several scenarios. Each scenario reference lists the slots it declares: [ecommerce](../scenarios/ecommerce.md#theme-slots), [saas](../scenarios/saas.md#theme-slots), and [travel](../scenarios/travel.md#theme-slots). A theme is compatible with a scenario when it fills every one of them; entries no selected scenario reads are ignored.

## Name generators

```toml
[names.person]
formats = [
    { format = "{given} {family}", weight = 3 },
    { format = "{given} {middle} {family}", weight = 1 },
]

[names.person.components]
given = ["Ada", "Grace"]
middle = ["River", "Sage"]
family = ["Meadow", "Stone"]
```

- Formats mix literal text and `{component}` references. Each reference expands over its pool of whole tokens, and repeated references expand independently.
- `weight` is a positive integer and defaults to `1`.
- Malformed references, unknown components, empty pools or tokens, blank names, and zero weights are errors.
- Duplicate full names from overlapping formats or pools count as one combination.

### Assignment and exhaustion

Each format's weight is divided among its distinct full names, and overlapping formats add their shares. A stream seeded by the run seed and the name kind produces a weighted permutation of the distinct names: heavier names tend to come earlier, and every name appears once before any repeats. After the last name, assignment starts the same permutation again.

The name for an entity depends only on the seed, the kind, and the entity's index, so worker count never changes who gets which name. Scenarios hand out contiguous indices to the entities that are written: ecommerce to ordering customers in store and pool order, travel to booking travellers in UUID order, and SaaS to accounts as they arrive and to people across leads, reps, and users.

Each bundled theme page lists its distinct names per kind against the default run's demand.

The generator materializes every distinct name and caches permutations by seed and kind, so memory and startup time grow with a theme's combinations.

## Label sets

```toml
[labels]
ranks = ["member", "regular", "supporter", "ambassador"]
```

Labels are ordered: a scenario reads them by index, so the first rank is always the least frequent customer and the third product category always names the third block of products. Each set must have exactly the length its scenario declares, and no value may be blank.

## Catalogs

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

A catalog is an ordered list of records. Fields are strings, numbers, or booleans, exactly as the scenario declares them; a record may not carry undeclared fields. Every money field is whole cents. Scenarios may add rules beyond types, such as a fixed record count or unique codes, and name them in their own reference.

## Parameters

```toml
[params.ecommerce]
price_scale = 13.0
```

Scenarios declare each parameter with a default and an inclusive range. A theme sets values in a section named for the scenario; parameters it leaves out take the default. `--param name=value` overrides one value for a run and is checked the same way.

## Table and column names

```toml
[schema.ecommerce]
tables = { tweets = "sparrows" }
columns = { "tweets.tweeted_at" = "sent_at", "customers.loyalty_tier" = "guild_rank" }
```

Keys use the scenario's generic names, which [the output schema](../output-schema.md) lists; column keys are `entity.column`. Renamed tables, columns, and primary keys apply in every format, and files are named for the renamed tables. New names must be lowercase identifiers (letters, digits, and underscores, starting with a letter) and unique within their table.

## Validation

The run fails before any rows are generated, naming the file and entry, when a theme:

- is invalid TOML or has an unknown top-level key;
- is missing a name kind, label set, or catalog the scenario declares, or a label set has the wrong length;
- has a catalog with too few records, or a record with a missing, mistyped, blank, or undeclared field (`catalogs.<name>[<index>].<field>`);
- sets an undeclared parameter or a value outside its range (`params.<scenario>.<name>`);
- renames an unknown table or column, uses a name that is not a lowercase identifier, or reuses a name within a table (`schema.<scenario>`).

Compatibility errors also list the bundled themes that would work.
