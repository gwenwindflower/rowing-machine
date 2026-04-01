# Static Data Reference

Hardcoded simulation data. Authoritative source: Go structs in `internal/catalog/` and `internal/models/store.go`.

## Store Configs

| Index | Name | Popularity | Opens (day) | TAM (base) | Tax Rate |
| --- | --- | --- | --- | --- | --- |
| 0 | Philadelphia | 0.85 | 0 | 9 | 0.0600 |
| 1 | Brooklyn | 0.95 | 192 | 14 | 0.0400 |
| 2 | Chicago | 0.92 | 605 | 12 | 0.0625 |
| 3 | San Francisco | 0.87 | 615 | 11 | 0.0750 |
| 4 | New Orleans | 0.92 | 920 | 8 | 0.0400 |
| 5 | Los Angeles | 0.87 | 1107 | 8 | 0.0800 |

TAM = base * scale (default scale=100). Index is critical — used as PRNG seed offset.

## Menu (10 items)

Jaffles: JAF-001 to JAF-005. Beverages: BEV-001 to BEV-005.

| SKU | Name | Cents | Type |
| --- | --- | --- | --- |
| JAF-001 | nutellaphone who dis? | 1100 | jaffle |
| JAF-002 | doctor stew | 1100 | jaffle |
| JAF-003 | the krautback | 1200 | jaffle |
| JAF-004 | flame impala | 1400 | jaffle |
| JAF-005 | mel-bun | 1200 | jaffle |
| BEV-001 | tangaroo | 600 | beverage |
| BEV-002 | chai and mighty | 500 | beverage |
| BEV-003 | vanilla ice | 600 | beverage |
| BEV-004 | for richer or pourover | 700 | beverage |
| BEV-005 | adele-ade | 400 | beverage |

## Supplies (29 items)

Non-perishable packaging (SUP-001 to SUP-007) shared by item type. Perishable ingredients (SUP-008 to SUP-029) mapped to specific SKUs. Output is denormalized: one row per (supply, SKU) pair — 65 rows total.

| ID | Name | Cost (cents) | Perishable | SKUs |
| --- | --- | --- | --- | --- |
| SUP-001 | compostable cutlery - knife | 7 | no | all jaffles |
| SUP-002 | cutlery - fork | 7 | no | all jaffles |
| SUP-003 | serving boat | 11 | no | all jaffles |
| SUP-004 | napkin | 4 | no | all jaffles |
| SUP-005 | 16oz compostable clear cup | 13 | no | all beverages |
| SUP-006 | 16oz compostable clear lid | 4 | no | all beverages |
| SUP-007 | biodegradable straw | 13 | no | all beverages |
| SUP-008 | chai mix | 98 | yes | BEV-002 |
| SUP-009 | bread | 33 | yes | all jaffles |
| SUP-010 | cheese | 20 | yes | JAF-002 thru JAF-005 |
| SUP-011 | nutella | 46 | yes | JAF-001 |
| SUP-012 | banana | 13 | yes | JAF-001 |
| SUP-013 | beef stew | 169 | yes | JAF-002 |
| SUP-014 | lamb and pork bratwurst | 234 | yes | JAF-003 |
| SUP-015 | house-pickled cabbage sauerkraut | 43 | yes | JAF-003 |
| SUP-016 | mustard | 7 | yes | JAF-003 |
| SUP-017 | pulled pork | 215 | yes | JAF-004 |
| SUP-018 | pineapple | 26 | yes | JAF-004 |
| SUP-019 | melon | 33 | yes | JAF-005 |
| SUP-020 | minced beef | 124 | yes | JAF-005 |
| SUP-021 | ghost pepper sauce | 20 | yes | JAF-004 |
| SUP-022 | mango | 32 | yes | BEV-001 |
| SUP-023 | tangerine | 20 | yes | BEV-001 |
| SUP-024 | oatmilk | 11 | yes | BEV-002 |
| SUP-025 | whey protein | 36 | yes | BEV-002 |
| SUP-026 | coffee | 52 | yes | BEV-003, BEV-004 |
| SUP-027 | french vanilla syrup | 72 | yes | BEV-003 |
| SUP-028 | kiwi | 20 | yes | BEV-005 |
| SUP-029 | lime | 13 | yes | BEV-005 |

## Persona Mix

| Persona | Weight | Weekday | Weekend | Tweet | Notes |
| --- | --- | --- | --- | --- | --- |
| Commuter | 0.25 | high | 0.001 | 0.20 | Order time: N(450, 30) |
| RemoteWorker | 0.25 | low-med | 0.001 | 0.01 | Order time: N(420, 180) |
| BrunchCrowd | 0.10 | 0 | med | 0.80 | Weekend-only, largest orders |
| Student | 0.20 | med | med | 0.80 | Absent in summer |
| Casuals | 0.10 | 0.1 | 0.1 | 0.10 | Constant probability |
| HealthNut | 0.10 | 0.2 | varies | 0.60 | Higher in summer |
