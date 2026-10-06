//! The fixed network: locations, the services between them, fleets, and add-ons.

use anyhow::{Result, bail, ensure};

use super::Params;
use crate::engine::stream::Stream;
use crate::theme::{Record, Theme};

const LOCATION_STREAM: &str = "travel-locations";
const SERVICE_STREAM: &str = "travel-services";
const VEHICLE_STREAM: &str = "travel-vehicles";
const MIN_ROUTE_KM: f64 = 200.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    City,
    Beach,
    Ski,
}

impl Kind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::City => "city",
            Self::Beach => "beach",
            Self::Ski => "ski",
        }
    }

    /// Demand relative to the yearly average, by calendar month (0 = January).
    pub const fn season(self, month: usize) -> f64 {
        const CITY: [f64; 12] = [
            0.85, 0.85, 0.95, 1.0, 1.0, 1.05, 1.1, 1.1, 1.0, 1.0, 0.9, 1.0,
        ];
        const BEACH: [f64; 12] = [
            0.55, 0.6, 0.75, 0.95, 1.1, 1.25, 1.35, 1.35, 1.15, 0.9, 0.6, 0.65,
        ];
        const SKI: [f64; 12] = [
            1.3, 1.35, 1.1, 0.8, 0.5, 0.5, 0.55, 0.55, 0.5, 0.6, 0.8, 1.25,
        ];
        match self {
            Self::City => CITY[month],
            Self::Beach => BEACH[month],
            Self::Ski => SKI[month],
        }
    }
}

pub struct Location {
    pub id: [u8; 16],
    pub name: String,
    pub code: String,
    pub latitude: f64,
    pub longitude: f64,
    pub base: bool,
    pub kind: Kind,
    pub weight: f64,
}

/// A scheduled link between a base and another location, flown as out-and-back rotations.
pub struct Service {
    pub origin: usize,
    pub destination: usize,
    pub routes: [[u8; 16]; 2],
    pub distance_km: f64,
    pub block_minutes: i64,
    pub frequency: f64,
    pub offset_minutes: i64,
}

pub struct Vehicle {
    pub id: [u8; 16],
    pub name: String,
    pub model: usize,
    pub capacity: usize,
}

/// A named option with a selection share and one numeric value, such as a fare class.
pub struct Choice {
    pub name: String,
    pub share: f64,
    pub value: f64,
}

pub struct AddOn {
    pub code: String,
    pub name: String,
    pub category: String,
    pub price: i64,
    pub attach_rate: f64,
}

pub struct Network {
    pub locations: Vec<Location>,
    pub bases: Vec<usize>,
    pub services: Vec<Service>,
    pub services_by_base: Vec<Vec<usize>>,
    pub fleets: Vec<Vec<Vehicle>>,
    pub vehicle_types: Vec<Choice>,
    pub fare_classes: Vec<Choice>,
    pub channels: Vec<Choice>,
    pub add_ons: Vec<AddOn>,
}

impl Network {
    /// Builds the network from the theme's catalogs and the run's parameters.
    ///
    /// # Errors
    /// Names the catalog entry when a value is outside what the simulation can use.
    pub fn build(seed: u64, theme: &Theme, params: &Params) -> Result<Self> {
        let locations = locations(seed, theme)?;
        let bases: Vec<_> = (0..locations.len())
            .filter(|index| locations[*index].base)
            .collect();
        ensure!(
            !bases.is_empty(),
            "theme {}: catalogs.locations needs at least one entry with base = true",
            theme.name
        );
        let vehicle_types = choices(theme, "vehicle_types", "capacity")?;
        for (index, vehicle) in vehicle_types.iter().enumerate() {
            ensure!(
                vehicle.value >= 1.0 && vehicle.value.fract() == 0.0,
                "theme {}: catalogs.vehicle_types[{index}].capacity must be a positive whole number",
                theme.name
            );
        }
        let fare_classes = choices(theme, "fare_classes", "multiplier")?;
        let channels = choices(theme, "channels", "")?;
        let add_ons = add_ons(theme)?;
        let (services, services_by_base) = services(seed, &locations, &bases, params);
        let fleets = fleets(
            seed,
            theme,
            params,
            &locations,
            &services,
            &services_by_base,
            &vehicle_types,
        );
        Ok(Self {
            locations,
            bases,
            services,
            services_by_base,
            fleets,
            vehicle_types,
            fare_classes,
            channels,
            add_ons,
        })
    }
}

fn locations(seed: u64, theme: &Theme) -> Result<Vec<Location>> {
    let mut locations = Vec::new();
    for (index, record) in theme.catalog("locations").iter().enumerate() {
        let kind = match record.text("kind") {
            "city" => Kind::City,
            "beach" => Kind::Beach,
            "ski" => Kind::Ski,
            other => bail!(
                "theme {}: catalogs.locations[{index}].kind is {other:?}; use city, beach, or ski",
                theme.name
            ),
        };
        let latitude = record.number("latitude");
        let longitude = record.number("longitude");
        ensure!(
            (-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude),
            "theme {}: catalogs.locations[{index}] has coordinates outside the globe",
            theme.name
        );
        let weight = record.number("weight");
        ensure!(
            weight > 0.0,
            "theme {}: catalogs.locations[{index}].weight must be greater than zero",
            theme.name
        );
        locations.push(Location {
            id: Stream::derive(seed, LOCATION_STREAM, &[index as u64]).uuid(),
            name: record.text("name").to_owned(),
            code: record.text("code").to_owned(),
            latitude,
            longitude,
            base: record.boolean("base"),
            kind,
            weight,
        });
    }
    for (index, location) in locations.iter().enumerate() {
        ensure!(
            !locations[..index]
                .iter()
                .any(|other| other.code == location.code),
            "theme {}: catalogs.locations[{index}].code {:?} repeats an earlier code",
            theme.name,
            location.code
        );
    }
    Ok(locations)
}

fn choices(theme: &Theme, catalog: &str, value: &str) -> Result<Vec<Choice>> {
    let records: &[Record] = theme.catalog(catalog);
    let mut choices = Vec::new();
    for (index, record) in records.iter().enumerate() {
        let share = record.number("share");
        ensure!(
            share > 0.0,
            "theme {}: catalogs.{catalog}[{index}].share must be greater than zero",
            theme.name
        );
        let value = if value.is_empty() {
            0.0
        } else {
            record.number(value)
        };
        ensure!(
            value >= 0.0,
            "theme {}: catalogs.{catalog}[{index}] must not be negative",
            theme.name
        );
        choices.push(Choice {
            name: record.text("name").to_owned(),
            share,
            value,
        });
    }
    Ok(choices)
}

#[allow(clippy::cast_possible_truncation)]
fn add_ons(theme: &Theme) -> Result<Vec<AddOn>> {
    let mut add_ons = Vec::new();
    for (index, record) in theme.catalog("add_ons").iter().enumerate() {
        let price = record.number("price");
        let attach_rate = record.number("attach_rate");
        ensure!(
            price >= 0.0 && price.fract() == 0.0,
            "theme {}: catalogs.add_ons[{index}].price must be whole cents",
            theme.name
        );
        ensure!(
            (0.0..=1.0).contains(&attach_rate),
            "theme {}: catalogs.add_ons[{index}].attach_rate must be from 0 to 1",
            theme.name
        );
        add_ons.push(AddOn {
            code: format!("AO-{:03}", index + 1),
            name: record.text("name").to_owned(),
            category: record.text("category").to_owned(),
            price: price as i64,
            attach_rate,
        });
    }
    Ok(add_ons)
}

/// Great-circle distance in kilometres.
pub fn distance_km(from: &Location, to: &Location) -> f64 {
    let (lat1, lat2) = (from.latitude.to_radians(), to.latitude.to_radians());
    let dlat = lat2 - lat1;
    let dlon = (to.longitude - from.longitude).to_radians();
    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    6371.0 * 2.0 * a.sqrt().asin()
}

#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
fn services(
    seed: u64,
    locations: &[Location],
    bases: &[usize],
    params: &Params,
) -> (Vec<Service>, Vec<Vec<usize>>) {
    let mut services = Vec::new();
    let mut by_base = vec![Vec::new(); bases.len()];
    for (slot, origin) in bases.iter().copied().enumerate() {
        let mut nearest: Option<(f64, usize)> = None;
        let mut chosen = Vec::new();
        for (destination, location) in locations.iter().enumerate() {
            if destination == origin || (location.base && destination < origin) {
                continue;
            }
            let distance = distance_km(&locations[origin], location);
            if !(MIN_ROUTE_KM..=params.max_route_km).contains(&distance) {
                continue;
            }
            if nearest.is_none_or(|(best, _)| distance < best) {
                nearest = Some((distance, destination));
            }
            let mut rng =
                Stream::derive(seed, SERVICE_STREAM, &[origin as u64, destination as u64]);
            if rng.uniform() < (params.route_density * location.weight).min(1.0) {
                chosen.push(destination);
            }
        }
        if chosen.is_empty() {
            chosen.extend(nearest.map(|(_, destination)| destination));
        }
        for destination in chosen {
            let distance = distance_km(&locations[origin], &locations[destination]);
            let mut rng = Stream::derive(
                seed,
                SERVICE_STREAM,
                &[origin as u64, destination as u64, 1],
            );
            let block = params.taxi_minutes + distance / params.cruise_kmh * 60.0;
            by_base[slot].push(services.len());
            services.push(Service {
                origin,
                destination,
                routes: [rng.uuid(), rng.uuid()],
                distance_km: distance,
                block_minutes: ((block / 5.0).round() * 5.0) as i64,
                frequency: (params.daily_frequency * locations[destination].weight)
                    .round()
                    .max(1.0),
                offset_minutes: rng.index(121) as i64 - 60,
            });
        }
    }
    (services, by_base)
}

/// Rotations a service flies on a day in the given month.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn rotations(service: &Service, kind: Kind, month: usize) -> usize {
    (service.frequency * kind.season(month).min(1.15) + 0.35).floor() as usize
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn fleets(
    seed: u64,
    theme: &Theme,
    params: &Params,
    locations: &[Location],
    services: &[Service],
    by_base: &[Vec<usize>],
    vehicle_types: &[Choice],
) -> Vec<Vec<Vehicle>> {
    let window = (params.last_departure - params.first_departure).max(60.0);
    let mut fleets = Vec::new();
    let mut named = 0;
    for (slot, indices) in by_base.iter().enumerate() {
        let peak = (0..12)
            .map(|month| {
                indices
                    .iter()
                    .map(|index| {
                        let service = &services[*index];
                        let rotation =
                            2.0 * (service.block_minutes as f64 + params.turnaround_minutes);
                        let kind = locations[service.destination].kind;
                        rotations(service, kind, month) as f64 * rotation
                    })
                    .sum::<f64>()
            })
            .fold(0.0, f64::max);
        let count = (peak / window).ceil() as usize + 1;
        let mut fleet = Vec::new();
        for index in 0..count {
            let mut rng = Stream::derive(seed, VEHICLE_STREAM, &[slot as u64, index as u64]);
            let model = pick(vehicle_types, rng.uniform());
            fleet.push(Vehicle {
                id: rng.uuid(),
                name: theme.name(seed, "vehicle", named),
                model,
                capacity: vehicle_types[model].value as usize,
            });
            named += 1;
        }
        fleets.push(fleet);
    }
    fleets
}

/// Selects a choice index by its share of the total.
pub fn pick(choices: &[Choice], uniform: f64) -> usize {
    let total: f64 = choices.iter().map(|choice| choice.share).sum();
    let mut threshold = uniform * total;
    for (index, choice) in choices.iter().enumerate() {
        if threshold < choice.share {
            return index;
        }
        threshold -= choice.share;
    }
    choices.len() - 1
}
