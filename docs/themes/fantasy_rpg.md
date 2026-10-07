# `fantasy_rpg` theme

`fantasy_rpg` skins the ecommerce shop as the Arcanum Collective, a mage guild whose six guild halls sell weapons, armor, and elixirs to adventurers. It is the data factory for Queria, a retro-RPG SQL trainer. Source: [`themes/fantasy_rpg.toml`](../../themes/fantasy_rpg.toml).

| Scenario | Reference |
| --- | --- |
| `ecommerce` | [Ecommerce scenario](../scenarios/ecommerce.md) |

## Table and column names

| Generic | `fantasy_rpg` |
| --- | --- |
| `tweets` | `sparrows` |
| `tweets.tweeted_at` | `sent_at` |
| `customers.loyalty_tier` | `guild_rank` |
| `products.category` | `type` |
| `products.type` | `power_level` |
| `supplies.origin_country` | `origin_region` |

`products.category` becomes `type` and `products.type` becomes `power_level`, so `type` holds weapon, armor, or elixir in this theme.

## Labels

Guild halls are in store index order, which fixes their opening days ([stores and customer pools](../scenarios/ecommerce.md#stores-and-customer-pools)). Ranks run from least to most frequent customer.

| Label set | Values |
| --- | --- |
| `stores` | Thornwall, Misthollow, Ironvale, Starfen, Duskmarsh, Sunspire |
| `product_categories` | weapon, armor, elixir |
| `product_types` | common, uncommon, rare, epic, legendary |
| `supply_origins` | Thornwall, Misthollow, Ironvale, Starfen, Duskmarsh, Sunspire |
| `ranks` | initiate, journeyman, adept, master |
| `rank_voices` | A novice's discovery, A practiced hand's report, An adept's appraisal, A master's verdict |
| `tweet_templates` | Name the guild "the Arcanum Collective" |
| `acquired_templates` | "Acquired a {one}" and its two- and three-item forms |

## Products

`price` is stored in cents; the table shows dollars. The `type` and `power_level` columns hold the category and type labels.

| SKU | Name | `type` | `power_level` | Price |
| --- | --- | --- | --- | --- |
| `WEP-001` | wyrmfang edge | weapon | common | $11.00 |
| `WEP-002` | stormcaller bow | weapon | uncommon | $11.00 |
| `WEP-003` | emberveil dagger | weapon | rare | $12.00 |
| `WEP-004` | inferno maul | weapon | epic | $14.00 |
| `WEP-005` | void sigil staff | weapon | legendary | $12.00 |
| `ARM-001` | ironbark buckler | armor | common | $8.00 |
| `ARM-002` | glacial bulwark | armor | uncommon | $12.00 |
| `ARM-003` | drake scale cuirass | armor | rare | $15.00 |
| `ARM-004` | phoenix ward mantle | armor | epic | $18.00 |
| `ARM-005` | voidweave vestments | armor | legendary | $20.00 |
| `ELX-001` | sunfire tonic | elixir | common | $6.00 |
| `ELX-002` | ironbark draught | elixir | uncommon | $5.00 |
| `ELX-003` | frostmint vial | elixir | rare | $6.00 |
| `ELX-004` | oracle's brew | elixir | epic | $7.00 |
| `ELX-005` | serpent's kiss | elixir | legendary | $4.00 |

The 41 supplies are reagents and materials (drake fire ember, thunderhawk sinew, void crystal, glass vial) costing 4–234 cents each. Prices and supply costs match `plain` item for item.

## Parameters

`fantasy_rpg` sets no parameters, so ecommerce runs with its defaults.

## Name capacity

| Kind | Distinct names | Default-run demand |
| --- | --- | --- |
| `person` | 7,047 | About 6,100 customers |

Names are a given name plus a family name, such as Fenris Ironbrook.

## Sample

```bash
rowing-machine --seed 42 --years 1 --scale 1 --theme fantasy_rpg --output-dir out/fantasy_rpg
```

`raw_customers.csv`, `raw_sparrows.csv`, and `raw_supplies.csv`:

```csv
id,name,guild_rank
ffde6d35-ca90-46a2-b0ee-e71eab351db4,Fenris Ironbrook,journeyman
842fb0de-6bf7-434a-9267-a0a7af18f08e,Kaida Rainbrook,journeyman
```

```csv
id,user_id,sent_at,content
f8415e47-27f5-41cb-9baa-043413492104,a5894a46-8c73-49d6-94a3-65c87d1c228d,2023-01-09T07:43:00,A master's verdict: The Arcanum Collective is fair enough. Acquired a sunfire tonic.
d4d700df-36cb-4bbe-bbff-d7ebd33def9b,a5894a46-8c73-49d6-94a3-65c87d1c228d,2023-01-25T08:16:00,A master's verdict: The Arcanum Collective is passable. Acquired a ironbark draught.
```

```csv
id,name,cost,volatile,origin_region,sku
SUP-001,enchanted wrapping cloth,7,False,Thornwall,WEP-001
SUP-001,enchanted wrapping cloth,7,False,Thornwall,WEP-002
```
