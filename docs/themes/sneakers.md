# `sneakers` theme

`sneakers` skins the ecommerce shop as Starcloud Sneakers, a Swiss-engineered running shoe and apparel brand selling through six global flagships. Source: [`themes/sneakers.toml`](../../themes/sneakers.toml).

| Scenario | Reference |
| --- | --- |
| `ecommerce` | [Ecommerce scenario](../scenarios/ecommerce.md) |

## Table and column names

`sneakers` has no `[schema.ecommerce]` section, so it writes the generic names from [the output schema](../output-schema.md#stores).

## Labels

Flagships are in store index order, which fixes their opening days ([stores and customer pools](../scenarios/ecommerce.md#stores-and-customer-pools)). Ranks run from least to most frequent customer.

| Label set | Values |
| --- | --- |
| `stores` | Zurich, New York, Tokyo, London, Paris, Melbourne |
| `product_categories` | road, trail, apparel |
| `product_types` | everyday, comfort, tempo, performance, race |
| `supply_origins` | China, Vietnam, Germany, Japan, Taiwan, Indonesia |
| `ranks` | new runner, regular, run club, ambassador |
| `rank_voices` | A new runner's review, A regular runner's review, A run club member's review, A Starcloud ambassador's review |
| `tweet_templates` | Name the brand "Starcloud" |
| `acquired_templates` | "Picked up {one}" and its two- and three-item forms |

## Products

`price` is stored in cents; the table shows dollars.

| SKU | Name | Category | Type | Price |
| --- | --- | --- | --- | --- |
| `WEP-001` | Vega | road | everyday | $150.00 |
| `WEP-002` | Lyra | road | comfort | $150.00 |
| `WEP-003` | Altair | road | tempo | $160.00 |
| `WEP-004` | Sirius | road | performance | $170.00 |
| `WEP-005` | Comet Racer | road | race | $250.00 |
| `ARM-001` | Polaris Trail | trail | everyday | $160.00 |
| `ARM-002` | Rigel Trail | trail | comfort | $160.00 |
| `ARM-003` | Antares Waterproof | trail | tempo | $180.00 |
| `ARM-004` | Deneb Ultra | trail | performance | $180.00 |
| `ARM-005` | Orion Summit | trail | race | $200.00 |
| `ELX-001` | Aurora Tee | apparel | everyday | $60.00 |
| `ELX-002` | Zenith Shorts | apparel | comfort | $70.00 |
| `ELX-003` | Nova Tights | apparel | tempo | $110.00 |
| `ELX-004` | Meteor Half Zip | apparel | performance | $85.00 |
| `ELX-005` | Eclipse Race Cap | apparel | race | $45.00 |

The 41 supplies are shoe and garment components (midsole foam compound, nylon propulsion plate, waterproof membrane, recycled polyester jersey) costing 52–3,042 cents each.

## Parameters

| Parameter | Value | Default | Effect |
| --- | --- | --- | --- |
| `purchase_rate` | 0.02 | 1.0 | Customers order about 2% as often as under `plain` |

## Name capacity

| Kind | Distinct names | Default-run demand |
| --- | --- | --- |
| `person` | 276,078 | About 5,000 customers |

Names come from US, UK, German, Japanese, French, Italian, Spanish, Dutch, and Portuguese pools, weighted toward the first four, with and without a middle initial.

## Sample

```bash
rowing-machine --seed 42 --years 1 --scale 1 --theme sneakers --output-dir out/sneakers
```

`raw_customers.csv` and `raw_tweets.csv`:

```csv
id,name,loyalty_tier
842fb0de-6bf7-434a-9267-a0a7af18f08e,Katharina H. Krause,new runner
30d75fe8-cfb1-40b1-8083-2fb838daf76e,Saki Sato,run club
```

```csv
id,user_id,tweeted_at,content
888d9ca7-d088-4da7-a5e0-ce573bc2ee5e,30d75fe8-cfb1-40b1-8083-2fb838daf76e,2023-01-31T07:18:00,A run club member's review: Starcloud is nothing special. Picked up Meteor Half Zip.
```
