# Static Data Reference

Quick-reference for hardcoded simulation data. Authoritative source: @MIGRATION.md sections 5 and 7.

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
Prices are in dollars in the spec; store as **cents** in Go.

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

See MIGRATION.md Section 7 for full table. Key points:

- SUP-001 to SUP-007: non-perishable packaging, shared by type
- SUP-008 to SUP-029: perishable ingredients, mapped to specific SKUs
- Output is denormalized: one row per (supply, sku) pair

## Persona Mix

| Persona | Weight | Weekday | Weekend | Tweet | Notes |
| --- | --- | --- | --- | --- | --- |
| Commuter | 0.25 | high | 0.001 | 0.20 | Order time: N(450, 30) — FIXED from Python bug |
| RemoteWorker | 0.25 | low-med | 0.001 | 0.01 | Order time: N(420, 180) |
| BrunchCrowd | 0.10 | 0 | med | 0.80 | Weekend-only, largest orders |
| Student | 0.20 | med | med | 0.80 | Absent in summer |
| Casuals | 0.10 | 0.1 | 0.1 | 0.10 | Constant probability |
| HealthNut | 0.10 | 0.2 | varies | 0.60 | Higher in summer |
