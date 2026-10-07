# `airline` theme

`airline` skins the travel network as SuperAir, a fictional low-cost airline flying European city, beach, and ski routes from six UK bases. It is the default theme for travel. Source: [`themes/airline.toml`](../../themes/airline.toml).

| Scenario | Reference |
| --- | --- |
| `travel` | [Travel scenario](../scenarios/travel.md) |

## Table and column names

| Generic | `airline` |
| --- | --- |
| `locations` | `airports` |
| `vehicles` | `aircraft` |
| `trips` | `flights` |
| `travellers` | `passengers` |
| `vehicles.name` | `registration` |
| `vehicles.base_id` | `base_airport_id` |
| `trips.code` | `flight_number` |
| `trips.vehicle_id` | `aircraft_id` |
| `trips.capacity` | `seats` |
| `tickets.trip_id` | `flight_id` |
| `bookings.traveller_id` | `passenger_id` |
| `travellers.home_location_id` | `home_airport_id` |
| `routes.origin_id` | `origin_airport_id` |
| `routes.destination_id` | `destination_airport_id` |

`routes`, `bookings`, `tickets`, `add_ons`, and `ticket_add_ons` keep their generic table names.

## Locations

43 airports. `weight` scales how often a base serves a location and how many rotations fly there.

| Group | Count | Airports (weight) |
| --- | --- | --- |
| Bases (`city`) | 6 | LGW London Gatwick (2.5), LTN London Luton (1.6), BRS Bristol (1.2), MAN Manchester (1.8), EDI Edinburgh (1.3), BFS Belfast International (0.9) |
| `city` | 15 | AMS (2.0), CDG (1.8), BCN (2.2), FCO (1.7), MXP (1.3), BER (1.5), PRG (1.1), BUD (1.0), CPH (1.0), LIS (1.8), OPO (1.1), NAP (1.0), VCE (1.0), RAK (0.8), KEF (0.5) |
| `beach` | 17 | AGP (2.2), ALC (2.0), PMI (2.3), FAO (1.9), IBZ (1.3), TFS (1.6), ACE (1.3), LPA (1.2), FNC (0.6), NCE (1.0), MLA (0.8), DBV (0.8), SPU (0.8), CFU (1.0), HER (1.1), RHO (0.9), PFO (0.7) |
| `ski` | 5 | GVA (1.8), GNB (0.5), SZG (0.6), INN (0.6), TRN (0.4) |

Base order sets which base considers a base-to-base pair first ([network](../scenarios/travel.md#network)).

## Fleet

| Vehicle type | Seats | Share |
| --- | --- | --- |
| Airbus A319 | 156 | 10% |
| Airbus A320 | 186 | 30% |
| Airbus A320neo | 186 | 35% |
| Airbus A321neo | 235 | 25% |

Registrations are `G-SA` plus two letters, such as G-SAEG.

## Fares and add-ons

Prices are stored in cents; the tables show pounds.

| Fare class | Multiplier | Share |
| --- | --- | --- |
| Saver | ×1.0 | 84% |
| Plus | ×1.45 | 11% |
| Flex | ×1.9 | 5% |

| ID | Add-on | Category | Price | Attach rate |
| --- | --- | --- | --- | --- |
| `AO-001` | 15kg hold bag | bags | £32.00 | 18% |
| `AO-002` | 23kg hold bag | bags | £42.00 | 14% |
| `AO-003` | Large cabin bag | bags | £24.00 | 20% |
| `AO-004` | Sports equipment | bags | £48.00 | 2% |
| `AO-005` | Standard seat selection | seats | £9.00 | 16% |
| `AO-006` | Extra legroom seat | seats | £22.00 | 7% |
| `AO-007` | Front row seat | seats | £15.00 | 6% |
| `AO-008` | Priority boarding | boarding | £8.00 | 8% |
| `AO-009` | Pre-ordered snack box | onboard | £9.00 | 4% |
| `AO-010` | Free date change | flexibility | £25.00 | 3% |

| Channel | Share |
| --- | --- |
| app | 55% |
| web | 38% |
| travel partner | 7% |

## Labels

| Label set | Values |
| --- | --- |
| `ranks` | occasional, regular, frequent, superflyer |
| `trip_prefix` | SA, so flight numbers run SA1000, SA1001, and on |

## Parameters

`airline` sets no `[params.travel]` values, so every [travel parameter](../scenarios/travel.md#parameters) takes its default.

## Name capacity

| Kind | Distinct names | Default-run demand |
| --- | --- | --- |
| `person` | 185,878 | About 180,000 travellers |
| `vehicle` | 676 | About 20 vehicles |

Passenger names come from UK, French, Spanish, Italian, German, Dutch, and Portuguese pools, weighted toward UK names, with and without a middle initial.

## Sample

`route_density=0` gives each base only its nearest destination, so the network has 6 routes each way (about 1.2 million tickets in a year).

```bash
rowing-machine --seed 42 --years 1 --scale 1 --theme airline --scenario travel --param route_density=0 --output-dir out/airline
```

`raw_aircraft.csv`, `raw_flights.csv`, and `raw_bookings.csv`:

```csv
id,registration,model,capacity,base_airport_id
5516b1ba-9cf1-4628-990b-42ed99e9864b,G-SAEG,Airbus A320,186,4dbdf827-74a9-4295-9e91-76b68923c8d2
bb02d058-1316-47de-ac0d-7e2167811c99,G-SASU,Airbus A320neo,186,4dbdf827-74a9-4295-9e91-76b68923c8d2
```

```csv
id,flight_number,route_id,aircraft_id,scheduled_departure_at,scheduled_arrival_at,departed_at,arrived_at,status,seats
abcfc449-345a-4142-a192-0c6f8b03158a,SA1000,5b9bb6bc-aeff-4fdc-a642-3aea1e212d53,5516b1ba-9cf1-4628-990b-42ed99e9864b,2023-01-01T09:20:00,2023-01-01T10:05:00,2023-01-01T09:23:00,2023-01-01T10:09:00,completed,186
c579226f-c80e-47cf-bbba-497184020680,SA1001,20f20daf-3016-4933-a705-aaac876e2167,5516b1ba-9cf1-4628-990b-42ed99e9864b,2023-01-01T10:35:00,2023-01-01T11:20:00,2023-01-01T10:43:00,2023-01-01T11:26:00,completed,186
```

```csv
id,passenger_id,booked_at,channel,fare_class,party_size,total_price
d984c48f-7829-48f4-9f1a-a17f2e5b65b8,1ce170a0-a957-4954-bfd2-7269858ed9b7,2022-12-26T05:44:00,web,Plus,1,6499
19eba92d-2d6e-4258-bdc6-bec4a8828809,f1391538-a103-43c3-9a96-0f4e6e8c91dc,2022-11-21T15:49:00,app,Saver,1,5499
```
