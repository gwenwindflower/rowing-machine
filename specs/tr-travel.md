# Travel

## Goals

The `travel` scenario simulates a transport network that people pay to ride: locations, routes between them, vehicles with seats, scheduled trips, and the bookings, tickets, and add-ons travellers buy. The bundled `airline` theme shapes it as SuperAir, a low-cost airline with six bases; other themes and `--param` values can reshape the same entities into a denser or sparser network. The data should let a learner compute load factor, revenue per seat and per kilometre, on-time performance, booking curves, add-on attach rates, route seasonality, and traveller loyalty. Rates and curves live in `docs/travel.md`, columns in `docs/output-schema.md`.

## Vocabulary

- **Base** — a location where vehicles are stationed; every route starts or ends at one.
- **Service** — a base and another location linked by out-and-back rotations.
- **Rotation** — one vehicle flying a service out and back; its two legs are trips.
- **Load factor** — tickets sold on a trip divided by its capacity.

## Requirements

### Network

- **tr-R001** — Routes come in out-and-back pairs between a base and another location in range, and the network stays fixed for the whole run.
- **tr-R002** — Each base's fleet is sized from its busiest month's schedule, and every vehicle stays at its base.
- **tr-R003** — Destinations follow their kind's season: beach routes fly less in winter, ski routes fly less in summer, and a route can pause for a season entirely.

### Trips

- **tr-R010** — A vehicle's next trip on a day never departs before its previous arrival plus the turnaround time, so a delay carries into its later trips.
- **tr-R011** — A cancelled trip has no departure or arrival time.
- **tr-R012** — A trip never sells more tickets than its capacity, and no seat is sold twice on one trip.
- **tr-R013** — Cancelling a trip cancels the other leg of its rotation.
- **tr-R014** — Every ticket on a cancelled trip is refunded.

### Bookings

- **tr-R020** — Every booking is made before its trip's scheduled departure.
- **tr-R021** — A booking's `party_size` equals its ticket count, and its `total_price` equals its tickets' fares plus their add-ons.
- **tr-R022** — Demand follows the destination's season, the day of the week, and yearly growth, so load factor peaks in summer and on Fridays and Sundays.
- **tr-R023** — Only travellers who booked at least once appear in the traveller output, each with their home base.
- **tr-R024** — Travellers split into four booking-count cohorts whose sizes differ by at most one; more bookings never means a lower tier.
- **tr-R025** — Fares for a trip rise as its departure gets closer.

### Parameters

- **tr-R030** — Changing a parameter that shapes schedules or demand, such as `daily_frequency`, leaves locations and routes unchanged.
