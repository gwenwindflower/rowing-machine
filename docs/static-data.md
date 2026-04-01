# Static Data Reference

Hardcoded simulation data. Authoritative source: Go structs in `internal/catalog/` and `internal/models/store.go`.

## Store Configs

| Index | Name | Popularity | Opens (day) | TAM (base) | Tax Rate |
| --- | --- | --- | --- | --- | --- |
| 0 | Thornwall | 0.85 | 0 | 9 | 0.0600 |
| 1 | Misthollow | 0.95 | 192 | 14 | 0.0400 |
| 2 | Ironvale | 0.92 | 605 | 12 | 0.0625 |
| 3 | Starfen | 0.87 | 615 | 11 | 0.0750 |
| 4 | Duskmarsh | 0.92 | 920 | 8 | 0.0400 |
| 5 | Sunspire | 0.87 | 1107 | 8 | 0.0800 |

TAM = base * scale (default scale=100). Index is critical — used as PRNG seed offset.

## Catalog (15 items)

Weapons: WEP-001 to WEP-005. Armor: ARM-001 to ARM-005. Elixirs: ELX-001 to ELX-005.

| SKU | Name | Cents | Type | Power Level |
| --- | --- | --- | --- | --- |
| WEP-001 | Shadowfang Dagger | 1100 | weapon | common |
| WEP-002 | Ironbark Staff | 1100 | weapon | uncommon |
| WEP-003 | Frostbite Halberd | 1200 | weapon | rare |
| WEP-004 | Emberclaw Greatsword | 1400 | weapon | epic |
| WEP-005 | Starfall Warhammer | 1200 | weapon | legendary |
| ARM-001 | Thistleguard Vest | 600 | armor | common |
| ARM-002 | Chainweave Tunic | 500 | armor | uncommon |
| ARM-003 | Drakescale Plate | 600 | armor | rare |
| ARM-004 | Voidmantle Cloak | 700 | armor | epic |
| ARM-005 | Celestial Aegis | 400 | armor | legendary |
| ELX-001 | Moonpetal Tonic | 800 | elixir | common |
| ELX-002 | Embervein Draught | 650 | elixir | uncommon |
| ELX-003 | Frostbloom Elixir | 750 | elixir | rare |
| ELX-004 | Starfire Philter | 900 | elixir | epic |
| ELX-005 | Voidheart Essence | 1000 | elixir | legendary |

## Supplies / Reagents (29 items)

Non-volatile components (SUP-001 to SUP-007) shared by item type. Volatile reagents (SUP-008 to SUP-029) mapped to specific SKUs. Output is denormalized: one row per (supply, SKU) pair — 65 rows total.

| ID | Name | Cost (cents) | Volatile | Origin Region | SKUs |
| --- | --- | --- | --- | --- | --- |
| SUP-001 | leather grip wrap | 7 | no | Thornwall | all weapons |
| SUP-002 | honing stone | 7 | no | Ironvale | all weapons |
| SUP-003 | weapon oil | 11 | no | Misthollow | all weapons |
| SUP-004 | polishing cloth | 4 | no | Sunspire | all weapons |
| SUP-005 | crystal vial | 13 | no | Starfen | all elixirs |
| SUP-006 | wax seal | 4 | no | Duskmarsh | all elixirs |
| SUP-007 | enchanted cork | 13 | no | Starfen | all elixirs |
| SUP-008 | moonpetal extract | 98 | yes | Misthollow | ELX-001 |
| SUP-009 | iron filings | 33 | yes | Ironvale | all weapons |
| SUP-010 | drake scale fragment | 20 | yes | Duskmarsh | ARM-002 thru ARM-005 |
| SUP-011 | shadow essence | 46 | yes | Duskmarsh | WEP-001 |
| SUP-012 | nightshade root | 13 | yes | Misthollow | WEP-001 |
| SUP-013 | troll sinew | 169 | yes | Thornwall | WEP-002 |
| SUP-014 | frost wyrm fang | 234 | yes | Starfen | WEP-003 |
| SUP-015 | permafrost crystal | 43 | yes | Starfen | WEP-003 |
| SUP-016 | winter mint oil | 7 | yes | Misthollow | WEP-003 |
| SUP-017 | ember core shard | 215 | yes | Sunspire | WEP-004 |
| SUP-018 | phoenix feather | 26 | yes | Sunspire | WEP-004 |
| SUP-019 | starmetal ingot | 33 | yes | Starfen | WEP-005 |
| SUP-020 | thunderstone dust | 124 | yes | Ironvale | WEP-005 |
| SUP-021 | ghost pepper resin | 20 | yes | Duskmarsh | WEP-004 |
| SUP-022 | embervein sap | 32 | yes | Sunspire | ELX-002 |
| SUP-023 | charcoal tincture | 20 | yes | Thornwall | ELX-002 |
| SUP-024 | frostbloom pollen | 11 | yes | Starfen | ELX-003 |
| SUP-025 | glacial spring water | 36 | yes | Starfen | ELX-003 |
| SUP-026 | starfire dust | 52 | yes | Sunspire | ELX-004, ELX-005 |
| SUP-027 | celestial dew | 72 | yes | Misthollow | ELX-004 |
| SUP-028 | voidstone powder | 20 | yes | Duskmarsh | ELX-005 |
| SUP-029 | abyssal salt | 13 | yes | Duskmarsh | ELX-005 |

## Persona Mix

| Persona | Weight | Weekday | Weekend | Sparrow | Notes |
| --- | --- | --- | --- | --- | --- |
| Courier | 0.25 | high | 0.001 | 0.20 | Order time: N(450, 30) |
| Artificer | 0.25 | low-med | 0.001 | 0.01 | Order time: N(420, 180) |
| FeastReveler | 0.10 | 0 | med | 0.80 | Weekend-only, largest orders |
| Apprentice | 0.20 | med | med | 0.80 | Absent in summer |
| Wanderer | 0.10 | 0.1 | 0.1 | 0.10 | Constant probability |
| Herbalist | 0.10 | 0.2 | varies | 0.60 | Higher in summer |
