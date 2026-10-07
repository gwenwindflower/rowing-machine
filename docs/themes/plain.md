# `plain` theme

`plain` uses everyday business vocabulary. For ecommerce it skins Market Collective, a six-store shop selling stationery, bags, and drinks; for SaaS it skins a generic B2B software company. It is the default theme for both scenarios. Source: [`themes/plain.toml`](../../themes/plain.toml).

| Scenario | Reference |
| --- | --- |
| `ecommerce` | [Ecommerce scenario](../scenarios/ecommerce.md) |
| `saas` | [SaaS scenario](../scenarios/saas.md) |

## Table and column names

`plain` has no `[schema.*]` section, so both scenarios write their generic table and column names from [the output schema](../output-schema.md).

## Ecommerce

### Labels

Stores are in index order, which fixes their opening days ([stores and customer pools](../scenarios/ecommerce.md#stores-and-customer-pools)). Ranks run from least to most frequent customer.

| Label set | Values |
| --- | --- |
| `stores` | Central, Lakeside, Hilltop, Riverside, Westside, Uptown |
| `product_categories` | stationery, accessory, beverage |
| `product_types` | basic, standard, select, premium, signature |
| `supply_origins` | United States, Canada, Germany, China, Brazil, India |
| `ranks` | new, regular, loyal, ambassador |
| `rank_voices` | A first-time shopper's review, A regular shopper's review, A loyal shopper's review, A brand ambassador's review |
| `tweet_templates` | Name the shop "Market Collective" |
| `acquired_templates` | "Purchased a {one}" and its two- and three-item forms |

### Products

`price` is stored in cents; the table shows dollars.

| SKU | Name | Category | Type | Price |
| --- | --- | --- | --- | --- |
| `WEP-001` | daily notebook | stationery | basic | $11.00 |
| `WEP-002` | weekly planner | stationery | standard | $11.00 |
| `WEP-003` | project journal | stationery | select | $12.00 |
| `WEP-004` | desk organizer | stationery | premium | $14.00 |
| `WEP-005` | document folder | stationery | signature | $12.00 |
| `ARM-001` | cotton tote | accessory | basic | $8.00 |
| `ARM-002` | canvas backpack | accessory | standard | $12.00 |
| `ARM-003` | travel pouch | accessory | select | $15.00 |
| `ARM-004` | laptop sleeve | accessory | premium | $18.00 |
| `ARM-005` | messenger bag | accessory | signature | $20.00 |
| `ELX-001` | citrus tea | beverage | basic | $6.00 |
| `ELX-002` | spiced oat latte | beverage | standard | $5.00 |
| `ELX-003` | vanilla cold brew | beverage | select | $6.00 |
| `ELX-004` | house coffee | beverage | premium | $7.00 |
| `ELX-005` | lime sparkling water | beverage | signature | $4.00 |

The 41 supplies are packaging and raw materials (wrapping paper, paper stock, cotton canvas, coffee grounds) costing 4–234 cents each.

## SaaS

| Slot | Values |
| --- | --- |
| `industries` | technology, finance, healthcare, retail, manufacturing, professional services |
| `roles` | admin, editor, viewer |
| `regions` | North America (UTC−5), Europe (UTC+1), Asia Pacific (UTC+9), Latin America (UTC−3) |
| `plan_tiers` | starter, growth, enterprise |
| Plan names | Essential, Professional, Enterprise, assigned to tiers by seed |
| Features | Dashboards, Reports, Exports, Alerts, Collections, Comments, Schedules, Search, Integrations, Workspaces, Permissions, Activity, Templates, Sharing, Metrics, Queries |
| Campaigns | A topic (such as Team Productivity or Connected Data) plus a format (Workshop, Guide, Showcase, Outreach) |
| Organizations | A quality, a place, and a business word, such as First Field Enterprises |

## Parameters

`plain` sets no parameters; ecommerce runs with its defaults, and SaaS declares none.

## Name capacity

| Kind | Distinct names | Default-run demand |
| --- | --- | --- |
| `person` | 126,242 | About 6,100 customers, or about 35,000 SaaS people |
| `organization` | 5,120 | About 3,000 accounts |
| `plan` | 3 | 3 plans |
| `feature` | 16 | 16 features |
| `campaign` | 32 | 51 campaigns over four years, so names repeat |

## Sample

```bash
rowing-machine --seed 42 --years 1 --scale 1 --theme plain --output-dir out/plain
rowing-machine --seed 42 --years 1 --scale 1 --theme plain --scenario saas --output-dir out/plain-saas
```

`raw_customers.csv` and `raw_tweets.csv` from the ecommerce run:

```csv
id,name,loyalty_tier
ffde6d35-ca90-46a2-b0ee-e71eab351db4,Micah Ira Morris,regular
842fb0de-6bf7-434a-9267-a0a7af18f08e,Luca Sanchez,regular
```

```csv
id,user_id,tweeted_at,content
f8415e47-27f5-41cb-9baa-043413492104,a5894a46-8c73-49d6-94a3-65c87d1c228d,2023-01-09T07:43:00,A brand ambassador's review: Market Collective is fair enough. Purchased a citrus tea.
d4d700df-36cb-4bbe-bbff-d7ebd33def9b,a5894a46-8c73-49d6-94a3-65c87d1c228d,2023-01-25T08:16:00,A brand ambassador's review: Market Collective is passable. Purchased a spiced oat latte.
```

`raw_accounts.csv` and `raw_plans.csv` from the SaaS run:

```csv
id,name,industry,employee_band,region,created_at,acquisition_channel,first_touch_id
9856b08a-b29b-4715-a5ca-632b7faa3410,First Field Enterprises,technology,small,Latin America,2023-01-10T00:00:00,paid_search,6a6cf7db-ac01-47d9-9c77-919d2824bef8
4758317c-2612-41b3-b859-22abe29334e0,Modern Grove Solutions,professional services,medium,Europe,2023-01-18T00:00:00,paid_search,0892fd8e-b6c9-45d3-9369-dd11d7099b01
```

```csv
id,name,tier,seat_price_monthly,seat_price_annual,included_seats
ec1592e2-3bc9-4288-b641-2a2d477e5899,Essential,starter,1500,15000,1
f5b152e6-77da-49e2-a3f5-f5e5efbb0ba2,Enterprise,growth,3000,30000,5
```
