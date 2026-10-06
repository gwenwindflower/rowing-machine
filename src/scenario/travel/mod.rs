//! Travel: locations, routes, vehicles, and scheduled trips that travellers book seats on.

mod network;

use std::collections::BTreeMap;

use anyhow::{Context, Result, anyhow};
use jiff::civil::Weekday;

use super::{Scenario, UnitRows, WorkUnit};
use crate::engine::{
    calendar::{DayState, Season},
    stream::Stream,
};
use crate::output::{Column, ColumnType, EntitySchema, Value};
use crate::theme::{CatalogSpec, FieldKind, ParamSpec, Theme, ThemeRequirements};
use network::{Network, pick, rotations};

const OPERATIONS_STREAM: &str = "travel-operations";
const DEMAND_STREAM: &str = "travel-demand";
const TRAVELLER_STREAM: &str = "travel-travellers";
const MINUTE: i64 = 60_000_000;
const SEAT_LETTERS: &[u8] = b"ABCDEF";
const PARTY_SIZES: [f64; 4] = [0.46, 0.34, 0.11, 0.09];

const PARAMS: &[ParamSpec] = &[
    param(
        "daily_frequency",
        1.0,
        0.1,
        24.0,
        "rotations per route per day for a weight-1 destination",
    ),
    param(
        "route_density",
        0.07,
        0.0,
        1.0,
        "chance each base serves a weight-1 location in range",
    ),
    param(
        "max_route_km",
        3800.0,
        250.0,
        20_000.0,
        "longest route a base flies",
    ),
    param(
        "load_factor",
        0.84,
        0.05,
        1.0,
        "average share of seats sold",
    ),
    param("annual_growth", 0.05, -0.5, 1.0, "yearly demand growth"),
    param(
        "lead_time_days",
        38.0,
        1.0,
        300.0,
        "mean days between booking and departure",
    ),
    param(
        "fare_base",
        1900.0,
        0.0,
        1_000_000.0,
        "base fare in cents before distance",
    ),
    param("fare_per_km", 2.4, 0.0, 1000.0, "fare cents per kilometre"),
    param(
        "cancellation_rate",
        0.012,
        0.0,
        0.5,
        "share of rotations cancelled",
    ),
    param(
        "delay_rate",
        0.32,
        0.0,
        1.0,
        "share of departures with their own delay",
    ),
    param(
        "delay_minutes",
        24.0,
        1.0,
        600.0,
        "mean length of a departure's own delay",
    ),
    param(
        "no_show_rate",
        0.03,
        0.0,
        0.5,
        "share of ticketed travellers who never board",
    ),
    param(
        "turnaround_minutes",
        30.0,
        5.0,
        600.0,
        "minimum time between a vehicle's arrival and next departure",
    ),
    param(
        "cruise_kmh",
        760.0,
        10.0,
        1200.0,
        "average speed between taxiing",
    ),
    param(
        "taxi_minutes",
        22.0,
        0.0,
        120.0,
        "minutes added to every trip's duration",
    ),
    param(
        "first_departure",
        360.0,
        0.0,
        1439.0,
        "earliest scheduled departure, minutes after midnight",
    ),
    param(
        "last_departure",
        1290.0,
        0.0,
        1439.0,
        "latest preferred departure, minutes after midnight",
    ),
    param(
        "travellers_per_base",
        300.0,
        1.0,
        100_000.0,
        "traveller pool per base, times --scale",
    ),
];

const fn param(
    name: &'static str,
    default: f64,
    min: f64,
    max: f64,
    about: &'static str,
) -> ParamSpec {
    ParamSpec {
        name,
        default,
        min,
        max,
        about,
    }
}

const CATALOGS: &[CatalogSpec] = &[
    CatalogSpec {
        name: "locations",
        min_len: 2,
        fields: &[
            ("name", FieldKind::Text),
            ("code", FieldKind::Text),
            ("latitude", FieldKind::Number),
            ("longitude", FieldKind::Number),
            ("base", FieldKind::Boolean),
            ("kind", FieldKind::Text),
            ("weight", FieldKind::Number),
        ],
    },
    CatalogSpec {
        name: "vehicle_types",
        min_len: 1,
        fields: &[
            ("name", FieldKind::Text),
            ("capacity", FieldKind::Number),
            ("share", FieldKind::Number),
        ],
    },
    CatalogSpec {
        name: "add_ons",
        min_len: 1,
        fields: &[
            ("name", FieldKind::Text),
            ("category", FieldKind::Text),
            ("price", FieldKind::Number),
            ("attach_rate", FieldKind::Number),
        ],
    },
    CatalogSpec {
        name: "fare_classes",
        min_len: 1,
        fields: &[
            ("name", FieldKind::Text),
            ("multiplier", FieldKind::Number),
            ("share", FieldKind::Number),
        ],
    },
    CatalogSpec {
        name: "channels",
        min_len: 1,
        fields: &[("name", FieldKind::Text), ("share", FieldKind::Number)],
    },
];

/// Parameter values resolved from the theme and `--param` overrides.
pub struct Params {
    daily_frequency: f64,
    route_density: f64,
    max_route_km: f64,
    load_factor: f64,
    annual_growth: f64,
    lead_time_days: f64,
    fare_base: f64,
    fare_per_km: f64,
    cancellation_rate: f64,
    delay_rate: f64,
    delay_minutes: f64,
    no_show_rate: f64,
    turnaround_minutes: f64,
    cruise_kmh: f64,
    taxi_minutes: f64,
    first_departure: f64,
    last_departure: f64,
    travellers_per_base: f64,
}

impl Params {
    fn from_theme(theme: &Theme) -> Self {
        let requirements = Travel::theme_requirements();
        let value = |name| theme.param(&requirements, name);
        Self {
            daily_frequency: value("daily_frequency"),
            route_density: value("route_density"),
            max_route_km: value("max_route_km"),
            load_factor: value("load_factor"),
            annual_growth: value("annual_growth"),
            lead_time_days: value("lead_time_days"),
            fare_base: value("fare_base"),
            fare_per_km: value("fare_per_km"),
            cancellation_rate: value("cancellation_rate"),
            delay_rate: value("delay_rate"),
            delay_minutes: value("delay_minutes"),
            no_show_rate: value("no_show_rate"),
            turnaround_minutes: value("turnaround_minutes"),
            cruise_kmh: value("cruise_kmh"),
            taxi_minutes: value("taxi_minutes"),
            first_departure: value("first_departure"),
            last_departure: value("last_departure"),
            travellers_per_base: value("travellers_per_base"),
        }
    }
}

pub struct Travel {
    days: Vec<DayState>,
    theme: Theme,
    params: Params,
    network: Network,
    pool: usize,
    counts: BTreeMap<[u8; 16], u64>,
    ranks: BTreeMap<[u8; 16], usize>,
    name_indices: BTreeMap<[u8; 16], usize>,
}

/// One scheduled leg flown by a vehicle.
struct Leg {
    service: usize,
    rotation: usize,
    inbound: bool,
    departure: i64,
}

impl Travel {
    /// Declares the names, labels, catalogs, and parameters travel reads from a theme.
    #[must_use]
    pub fn theme_requirements() -> ThemeRequirements {
        ThemeRequirements {
            scenario: "travel",
            name_kinds: &["person", "vehicle"],
            label_sets: &[("ranks", 4), ("trip_prefix", 1)],
            catalogs: CATALOGS,
            params: PARAMS,
        }
    }

    /// Builds the network and traveller pools for a compatible theme.
    ///
    /// # Errors
    /// Returns an error for incompatible themes, unusable catalogs, or an empty calendar.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn with_theme(seed: u64, scale: usize, days: Vec<DayState>, theme: Theme) -> Result<Self> {
        theme.validate(&Self::theme_requirements())?;
        anyhow::ensure!(
            !days.is_empty(),
            "travel requires at least one simulation day"
        );
        let params = Params::from_theme(&theme);
        let network = Network::build(seed, &theme, &params)?;
        let pool = (params.travellers_per_base.round() as usize)
            .checked_mul(scale)
            .ok_or_else(|| {
                anyhow!("--scale {scale} exceeds the traveller pool limit; use a smaller value")
            })?;
        Ok(Self {
            days,
            theme,
            params,
            network,
            pool,
            counts: BTreeMap::new(),
            ranks: BTreeMap::new(),
            name_indices: BTreeMap::new(),
        })
    }

    fn traveller(seed: u64, slot: usize, index: usize) -> [u8; 16] {
        Stream::derive(seed, TRAVELLER_STREAM, &[slot as u64, index as u64]).uuid()
    }

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap
    )]
    fn static_rows(&self) -> UnitRows {
        let network = &self.network;
        let mut rows = Vec::new();
        for location in &network.locations {
            rows.push((
                "locations",
                vec![
                    Value::Uuid(location.id),
                    text(&location.code),
                    text(&location.name),
                    Value::Float(location.latitude),
                    Value::Float(location.longitude),
                    text(location.kind.name()),
                    Value::Boolean(location.base),
                ],
            ));
        }
        for service in &network.services {
            for (direction, (from, to)) in [
                (service.origin, service.destination),
                (service.destination, service.origin),
            ]
            .into_iter()
            .enumerate()
            {
                rows.push((
                    "routes",
                    vec![
                        Value::Uuid(service.routes[direction]),
                        Value::Uuid(network.locations[from].id),
                        Value::Uuid(network.locations[to].id),
                        Value::Integer(service.distance_km.round() as i64),
                        Value::Integer(service.block_minutes),
                    ],
                ));
            }
        }
        for (slot, fleet) in network.fleets.iter().enumerate() {
            for vehicle in fleet {
                rows.push((
                    "vehicles",
                    vec![
                        Value::Uuid(vehicle.id),
                        text(&vehicle.name),
                        text(&network.vehicle_types[vehicle.model].name),
                        Value::Integer(vehicle.capacity as i64),
                        Value::Uuid(network.locations[network.bases[slot]].id),
                    ],
                ));
            }
        }
        for add_on in &network.add_ons {
            rows.push((
                "add_ons",
                vec![
                    text(&add_on.code),
                    text(&add_on.name),
                    text(&add_on.category),
                    Value::Cents(add_on.price),
                ],
            ));
        }
        rows
    }

    /// Assigns the day's rotations at one base to its fleet, earliest-free vehicle first.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_possible_wrap
    )]
    fn schedule(&self, slot: usize, month: usize) -> Vec<Vec<Leg>> {
        let network = &self.network;
        let first = self.params.first_departure as i64;
        let last = self.params.last_departure as i64;
        let turnaround = self.params.turnaround_minutes as i64;
        let mut wanted = Vec::new();
        for index in &network.services_by_base[slot] {
            let service = &network.services[*index];
            let count = rotations(service, network.locations[service.destination].kind, month);
            for rotation in 0..count {
                let spread = (last - first) as f64 * (rotation as f64 + 0.5) / count as f64;
                let preferred = (first + spread as i64 + service.offset_minutes).clamp(first, last);
                wanted.push((preferred, *index, rotation));
            }
        }
        wanted.sort_unstable();
        let fleet = &network.fleets[slot];
        let mut free = vec![first - turnaround; fleet.len()];
        let mut legs: Vec<Vec<Leg>> = fleet.iter().map(|_| Vec::new()).collect();
        for (preferred, index, rotation) in wanted {
            let vehicle = (0..fleet.len())
                .min_by_key(|vehicle| (free[*vehicle], *vehicle))
                .expect("every base has a vehicle");
            let block = network.services[index].block_minutes;
            let outbound = round_up(preferred.max(free[vehicle] + turnaround), 5);
            let inbound = round_up(outbound + block + turnaround, 5);
            free[vehicle] = inbound + block;
            legs[vehicle].push(Leg {
                service: index,
                rotation,
                inbound: false,
                departure: outbound,
            });
            legs[vehicle].push(Leg {
                service: index,
                rotation,
                inbound: true,
                departure: inbound,
            });
        }
        legs
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_wrap,
        clippy::too_many_lines
    )]
    fn base_day(&self, seed: u64, day_index: usize, slot: usize) -> Result<UnitRows> {
        let day = self
            .days
            .get(day_index)
            .ok_or_else(|| anyhow!("invalid travel day {day_index}"))?;
        let network = &self.network;
        let params = &self.params;
        let month = usize::try_from(day.date.month() - 1)?;
        let midnight = day
            .date
            .at(0, 0, 0, 0)
            .to_zoned(jiff::tz::TimeZone::UTC)?
            .timestamp()
            .as_microsecond();
        let growth = (1.0 + params.annual_growth).powf(day_index as f64 / 365.0);
        let weekday = match day.date.weekday() {
            Weekday::Monday => 1.0,
            Weekday::Tuesday => 0.86,
            Weekday::Wednesday => 0.9,
            Weekday::Thursday => 1.02,
            Weekday::Friday => 1.16,
            Weekday::Saturday => 0.97,
            Weekday::Sunday => 1.12,
        };
        let winter = if day.season == Season::Winter {
            1.8
        } else {
            1.0
        };
        let prefix = self.theme.label("trip_prefix", 0);
        let mut rows = Vec::new();
        for (vehicle_index, legs) in self.schedule(slot, month).into_iter().enumerate() {
            let vehicle = &network.fleets[slot][vehicle_index];
            let mut operations = Stream::derive(
                seed,
                OPERATIONS_STREAM,
                &[day_index as u64, slot as u64, vehicle_index as u64],
            );
            let mut ready: Option<i64> = None;
            let mut cancelled = false;
            for leg in legs {
                let service = &network.services[leg.service];
                let destination = &network.locations[service.destination];
                if !leg.inbound {
                    cancelled = operations.uniform() < params.cancellation_rate * winter;
                }
                let scheduled = midnight + leg.departure * MINUTE;
                let scheduled_arrival = scheduled + service.block_minutes * MINUTE;
                let (departed, arrived) = if cancelled {
                    (None, None)
                } else {
                    let own = if operations.uniform() < params.delay_rate {
                        -params.delay_minutes * (1.0 - operations.uniform()).ln()
                    } else {
                        operations.uniform() * 5.0
                    };
                    let earliest = ready.map_or(scheduled, |ready| ready.max(scheduled));
                    let departed = earliest + (own.round() as i64) * MINUTE;
                    let airborne = (service.block_minutes as f64 + operations.normal(0.0, 6.0))
                        .max(service.block_minutes as f64 * 0.85);
                    let arrived = departed + (airborne.round() as i64) * MINUTE;
                    ready = Some(arrived + (params.turnaround_minutes as i64) * MINUTE);
                    (Some(departed), Some(arrived))
                };
                let mut demand = Stream::derive(
                    seed,
                    DEMAND_STREAM,
                    &[
                        day_index as u64,
                        leg.service as u64,
                        leg.rotation as u64,
                        u64::from(leg.inbound),
                    ],
                );
                let trip_id = demand.uuid();
                let number = 1000 + leg.service * 48 + leg.rotation * 2 + usize::from(leg.inbound);
                rows.push((
                    "trips",
                    vec![
                        Value::Uuid(trip_id),
                        text(&format!("{prefix}{number}")),
                        Value::Uuid(service.routes[usize::from(leg.inbound)]),
                        Value::Uuid(vehicle.id),
                        Value::Timestamp(scheduled),
                        Value::Timestamp(scheduled_arrival),
                        departed.map_or(Value::Null, Value::Timestamp),
                        arrived.map_or(Value::Null, Value::Timestamp),
                        text(if cancelled { "cancelled" } else { "completed" }),
                        Value::Integer(vehicle.capacity as i64),
                    ],
                ));
                let season = destination.kind.season(month);
                let load = (params.load_factor
                    * (season * weekday * growth).powf(0.6)
                    * demand.normal(1.0, 0.06))
                .clamp(0.15, 1.0);
                let capacity = vehicle.capacity as f64;
                let spread = (capacity * load * (1.0 - load)).sqrt();
                let sold = (capacity * load + demand.normal(0.0, spread))
                    .round()
                    .clamp(0.0, capacity) as usize;
                let mut seats: Vec<usize> = (0..vehicle.capacity).collect();
                for index in 0..sold {
                    let swap = index + demand.index(vehicle.capacity - index);
                    seats.swap(index, swap);
                }
                let distance_fare = params.fare_base + params.fare_per_km * service.distance_km;
                let mut seated = 0;
                while seated < sold {
                    let mut party = 1 + pick_weights(&PARTY_SIZES, demand.uniform());
                    party = party.min(sold - seated);
                    let traveller_index =
                        ((self.pool as f64) * demand.uniform().powf(2.0)) as usize;
                    let traveller = Self::traveller(seed, slot, traveller_index.min(self.pool - 1));
                    let mean_lead = params.lead_time_days * if party > 2 { 1.3 } else { 1.0 };
                    let lead_days = (-mean_lead * (1.0 - demand.uniform()).ln()).min(330.0);
                    let booked_at =
                        scheduled - ((lead_days * 1440.0).max(45.0).round() as i64) * MINUTE;
                    let channel = pick(&network.channels, demand.uniform());
                    let class = pick(&network.fare_classes, demand.uniform());
                    let booking_id = demand.uuid();
                    let mut total = 0;
                    let mut booking_rows = Vec::new();
                    for seat in &seats[seated..seated + party] {
                        let multiplier = (0.72 + 0.95 * (-lead_days / 18.0).exp())
                            * (0.75 + 0.45 * load)
                            * network.fare_classes[class].value
                            * demand.normal(1.0, 0.07).max(0.6);
                        let fare = ((distance_fare * multiplier / 100.0).round() as i64 * 100 - 1)
                            .max(999);
                        total += fare;
                        let status = if cancelled {
                            "refunded"
                        } else if demand.uniform() < params.no_show_rate {
                            "no_show"
                        } else {
                            "flown"
                        };
                        let ticket_id = demand.uuid();
                        booking_rows.push((
                            "tickets",
                            vec![
                                Value::Uuid(ticket_id),
                                Value::Uuid(booking_id),
                                Value::Uuid(trip_id),
                                text(&format!(
                                    "{}{}",
                                    seat / SEAT_LETTERS.len() + 1,
                                    SEAT_LETTERS[seat % SEAT_LETTERS.len()] as char
                                )),
                                Value::Cents(fare),
                                text(status),
                            ],
                        ));
                        for add_on in &network.add_ons {
                            if demand.uniform() < add_on.attach_rate {
                                total += add_on.price;
                                booking_rows.push((
                                    "ticket_add_ons",
                                    vec![
                                        Value::Uuid(demand.uuid()),
                                        Value::Uuid(ticket_id),
                                        text(&add_on.code),
                                        Value::Cents(add_on.price),
                                    ],
                                ));
                            }
                        }
                    }
                    rows.push((
                        "bookings",
                        vec![
                            Value::Uuid(booking_id),
                            Value::Uuid(traveller),
                            Value::Timestamp(booked_at),
                            text(&network.channels[channel].name),
                            text(&network.fare_classes[class].name),
                            Value::Integer(party as i64),
                            Value::Cents(total),
                        ],
                    ));
                    rows.extend(booking_rows);
                    seated += party;
                }
            }
        }
        Ok(rows)
    }

    fn travellers(&self, seed: u64, slot: usize) -> UnitRows {
        let home = self.network.locations[self.network.bases[slot]].id;
        (0..self.pool)
            .filter_map(|index| {
                let id = Self::traveller(seed, slot, index);
                self.ranks.get(&id).map(|rank| {
                    (
                        "travellers",
                        vec![
                            Value::Uuid(id),
                            text(&self.theme.name(seed, "person", self.name_indices[&id])),
                            Value::Uuid(home),
                            text(self.theme.label("ranks", *rank)),
                        ],
                    )
                })
            })
            .collect()
    }
}

impl Scenario for Travel {
    fn name(&self) -> &'static str {
        "travel"
    }

    #[allow(clippy::too_many_lines)]
    fn entities(&self) -> Vec<EntitySchema> {
        use ColumnType::{Boolean, Cents, Float, Integer, Text, Timestamp, Uuid};
        vec![
            schema(
                "locations",
                &[
                    ("id", Uuid, false),
                    ("code", Text, false),
                    ("name", Text, false),
                    ("latitude", Float, false),
                    ("longitude", Float, false),
                    ("kind", Text, false),
                    ("is_base", Boolean, false),
                ],
                &["id"],
            ),
            schema(
                "routes",
                &[
                    ("id", Uuid, false),
                    ("origin_id", Uuid, false),
                    ("destination_id", Uuid, false),
                    ("distance_km", Integer, false),
                    ("duration_minutes", Integer, false),
                ],
                &["id"],
            ),
            schema(
                "vehicles",
                &[
                    ("id", Uuid, false),
                    ("name", Text, false),
                    ("model", Text, false),
                    ("capacity", Integer, false),
                    ("base_id", Uuid, false),
                ],
                &["id"],
            ),
            schema(
                "add_ons",
                &[
                    ("id", Text, false),
                    ("name", Text, false),
                    ("category", Text, false),
                    ("price", Cents, false),
                ],
                &["id"],
            ),
            schema(
                "trips",
                &[
                    ("id", Uuid, false),
                    ("code", Text, false),
                    ("route_id", Uuid, false),
                    ("vehicle_id", Uuid, false),
                    ("scheduled_departure_at", Timestamp, false),
                    ("scheduled_arrival_at", Timestamp, false),
                    ("departed_at", Timestamp, true),
                    ("arrived_at", Timestamp, true),
                    ("status", Text, false),
                    ("capacity", Integer, false),
                ],
                &["id"],
            ),
            schema(
                "bookings",
                &[
                    ("id", Uuid, false),
                    ("traveller_id", Uuid, false),
                    ("booked_at", Timestamp, false),
                    ("channel", Text, false),
                    ("fare_class", Text, false),
                    ("party_size", Integer, false),
                    ("total_price", Cents, false),
                ],
                &["id"],
            ),
            schema(
                "tickets",
                &[
                    ("id", Uuid, false),
                    ("booking_id", Uuid, false),
                    ("trip_id", Uuid, false),
                    ("seat", Text, false),
                    ("fare", Cents, false),
                    ("status", Text, false),
                ],
                &["id"],
            ),
            schema(
                "ticket_add_ons",
                &[
                    ("id", Uuid, false),
                    ("ticket_id", Uuid, false),
                    ("add_on_id", Text, false),
                    ("price", Cents, false),
                ],
                &["id"],
            ),
            schema(
                "travellers",
                &[
                    ("id", Uuid, false),
                    ("name", Text, false),
                    ("home_location_id", Uuid, false),
                    ("loyalty_tier", Text, false),
                ],
                &["id"],
            ),
        ]
    }

    fn units(&self) -> Vec<WorkUnit> {
        let bases = self.network.bases.len() as u64;
        let mut units = vec![WorkUnit {
            stage: 0,
            indices: [0, 0],
        }];
        for day in 0..self.days.len() as u64 {
            units.extend((0..bases).map(|slot| WorkUnit {
                stage: 1,
                indices: [day, slot],
            }));
        }
        units.extend((0..bases).map(|slot| WorkUnit {
            stage: 2,
            indices: [0, slot],
        }));
        units
    }

    fn generate(&self, seed: u64, unit: WorkUnit) -> Result<UnitRows> {
        let day = usize::try_from(unit.indices[0])?;
        let slot = usize::try_from(unit.indices[1])?;
        anyhow::ensure!(
            slot < self.network.bases.len(),
            "invalid travel base {slot}"
        );
        match unit.stage {
            0 => Ok(self.static_rows()),
            1 => self
                .base_day(seed, day, slot)
                .with_context(|| format!("generating travel day {day} at base {slot}")),
            2 => Ok(self.travellers(seed, slot)),
            _ => anyhow::bail!("invalid travel stage {}", unit.stage),
        }
    }

    fn observe(&mut self, rows: &UnitRows) -> Result<()> {
        for (entity, row) in rows {
            if *entity == "bookings" {
                let Some(Value::Uuid(id)) = row.get(1) else {
                    anyhow::bail!("booking row has no traveller UUID");
                };
                *self.counts.entry(*id).or_default() += 1;
            }
        }
        Ok(())
    }

    fn complete_stage(&mut self, stage: u32) -> Result<()> {
        if stage == 1 {
            let mut counts: Vec<_> = self
                .counts
                .iter()
                .map(|(id, count)| (*count, *id))
                .collect();
            counts.sort_unstable();
            let length = counts.len();
            self.ranks = counts
                .into_iter()
                .enumerate()
                .map(|(index, (_, id))| (id, index * 4 / length))
                .collect();
            self.name_indices = self
                .counts
                .keys()
                .enumerate()
                .map(|(index, id)| (*id, index))
                .collect();
        }
        Ok(())
    }
}

fn pick_weights(weights: &[f64], uniform: f64) -> usize {
    let mut threshold = uniform * weights.iter().sum::<f64>();
    for (index, weight) in weights.iter().enumerate() {
        if threshold < *weight {
            return index;
        }
        threshold -= weight;
    }
    weights.len() - 1
}

const fn round_up(minutes: i64, step: i64) -> i64 {
    (minutes + step - 1) / step * step
}

fn text(value: &str) -> Value {
    Value::Text(value.to_owned())
}

fn schema(
    name: &'static str,
    fields: &[(&'static str, ColumnType, bool)],
    key: &[&'static str],
) -> EntitySchema {
    EntitySchema {
        name,
        columns: fields
            .iter()
            .map(|(name, column_type, nullable)| Column {
                name,
                column_type: *column_type,
                nullable: *nullable,
            })
            .collect(),
        primary_key: key.to_vec(),
    }
}
