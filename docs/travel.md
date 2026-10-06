# Travel model

The [travel scenario](../src/scenario/travel/mod.rs) builds a fixed network from the theme's catalogs, then simulates each base's day: it schedules rotations onto the base's fleet, flies them with delays and cancellations, and sells seats to the base's traveller pool. Every unit is one base-day, generated from streams keyed by day, service, rotation, and vehicle. All timestamps are UTC. The generic tables (locations, vehicles, trips, travellers) take their theme's names; `airline` writes airports, aircraft, flights, and passengers.

## Network

- **Locations** come from `catalogs.locations`. Bases (`base = true`) station vehicles; `kind` is `city`, `beach`, or `ski` and picks the seasonal curve; `weight` scales popularity.
- **Services** link a base to each location between 200 km and `max_route_km` away with probability `route_density × weight` (capped at 1). A base with no chosen service gets its nearest location in range. Pairs of bases are linked once.
- **Routes** are a service's two directions. Duration is `taxi_minutes + distance / cruise_kmh`, rounded to 5 minutes.
- **Fleets** hold enough vehicles for the base's busiest month, plus one spare. Vehicle types come from `catalogs.vehicle_types` by share.

## Schedule

A service flies `round(daily_frequency × weight)` rotations a day (at least 1) at its peak. In a given month it flies `floor(frequency × min(season, 1.15) + 0.35)` rotations, so a beach route with one daily rotation pauses from November to February and a ski route pauses from April to October.

| Kind | Jan | Feb | Mar | Apr | May | Jun | Jul | Aug | Sep | Oct | Nov | Dec |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| city | 0.85 | 0.85 | 0.95 | 1.0 | 1.0 | 1.05 | 1.1 | 1.1 | 1.0 | 1.0 | 0.9 | 1.0 |
| beach | 0.55 | 0.6 | 0.75 | 0.95 | 1.1 | 1.25 | 1.35 | 1.35 | 1.15 | 0.9 | 0.6 | 0.65 |
| ski | 1.3 | 1.35 | 1.1 | 0.8 | 0.5 | 0.5 | 0.55 | 0.55 | 0.5 | 0.6 | 0.8 | 1.25 |

Rotations spread evenly between `first_departure` and `last_departure`, shifted by a fixed per-service offset of up to an hour, so a route keeps the same departure times day to day. Each rotation goes to the vehicle that frees up first; the return leg leaves one turnaround after arrival. Trip codes are the theme's `trip_prefix` plus a number fixed by service and rotation; inbound legs are odd.

## Operations

- A rotation is cancelled with probability `cancellation_rate` (1.8× in winter). Both legs are cancelled and every ticket is refunded.
- Each departure leaves at the later of its schedule and the vehicle's previous arrival plus `turnaround_minutes`, plus its own delay: with probability `delay_rate` an exponential delay with mean `delay_minutes`, otherwise 0–5 minutes.
- Flying time varies by a normal ±6 minutes, never under 85% of the scheduled duration.

## Demand and fares

Load factor is `load_factor × (season × weekday × growth)^0.6`, times a normal ±6% draw, clamped to 0.15–1.0. Weekday factors run from 0.86 on Tuesday to 1.16 on Friday and 1.12 on Sunday; growth compounds `annual_growth` yearly. Seats sold follow a normal approximation of the binomial, capped at capacity, and get distinct seats.

Seats sell in parties of 1–4 (46%, 34%, 11%, 9%). Each party is one booking by a traveller drawn from its base's pool of `travellers_per_base × --scale`, skewed so a minority books often. Lead time is exponential with mean `lead_time_days` (1.3× for parties above two), capped at 330 days. Channel and fare class come from their catalogs by share.

Each ticket's fare is `(fare_base + fare_per_km × km) × (0.72 + 0.95 e^(-lead/18)) × (0.75 + 0.45 × load) × class multiplier × normal(1, 0.07)`, rounded to whole units minus one cent (£45.99), with a 999-cent floor. Each ticket independently buys each add-on with its `attach_rate`, at its catalog price. A ticket on a flown trip is `no_show` with probability `no_show_rate`, otherwise `flown`.

Loyalty tiers split travellers who booked into four cohorts by booking count, with ties broken by traveller UUID.

## Parameters

Set these under `[params.travel]` in a theme or with `--param name=value`.

| Parameter | Default | Range | Meaning |
| --- | --- | --- | --- |
| `daily_frequency` | 1.0 | 0.1–24.0 | Rotations per route per day for a weight-1 destination |
| `route_density` | 0.07 | 0.0–1.0 | Chance each base serves a weight-1 location in range |
| `max_route_km` | 3800.0 | 250.0–20,000.0 | Longest route a base flies |
| `load_factor` | 0.84 | 0.05–1.0 | Average share of seats sold |
| `annual_growth` | 0.05 | -0.5–1.0 | Yearly demand growth |
| `lead_time_days` | 38.0 | 1.0–300.0 | Mean days between booking and departure |
| `fare_base` | 1900.0 | 0.0–1,000,000.0 | Base fare in cents before distance |
| `fare_per_km` | 2.4 | 0.0–1000.0 | Fare cents per kilometre |
| `cancellation_rate` | 0.012 | 0.0–0.5 | Share of rotations cancelled |
| `delay_rate` | 0.32 | 0.0–1.0 | Share of departures with their own delay |
| `delay_minutes` | 24.0 | 1.0–600.0 | Mean length of a departure's own delay |
| `no_show_rate` | 0.03 | 0.0–0.5 | Share of ticketed travellers who never board |
| `turnaround_minutes` | 30.0 | 5.0–600.0 | Minimum time between a vehicle's arrival and next departure |
| `cruise_kmh` | 760.0 | 10.0–1200.0 | Average speed between taxiing |
| `taxi_minutes` | 22.0 | 0.0–120.0 | Minutes added to every trip's duration |
| `first_departure` | 360.0 | 0.0–1439.0 | Earliest scheduled departure, minutes after midnight |
| `last_departure` | 1290.0 | 0.0–1439.0 | Latest preferred departure, minutes after midnight |
| `travellers_per_base` | 300.0 | 1.0–100,000.0 | Traveller pool per base, times --scale |
