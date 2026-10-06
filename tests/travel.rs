use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use assert_cmd::cargo::cargo_bin_cmd;

type Row = BTreeMap<String, String>;

const THEME: &str = r#"
name = "tiny_air"
description = "Two bases and three destinations for fast travel tests."

[names.person]
formats = [{ format = "{given} {family}" }]
[names.person.components]
given = ["Ada", "Ben", "Cleo", "Dev", "Eve", "Finn", "Gia", "Hal", "Ivy", "Jo", "Kai", "Lu", "Mo", "Nia", "Oz", "Pia"]
family = ["Ash", "Birch", "Cedar", "Dale", "Elm", "Fern", "Glen", "Hale", "Iris", "June", "Kerr", "Lark", "Moss", "North", "Oak", "Pine", "Quill", "Reed", "Sage", "Thorn"]

[names.vehicle]
formats = [{ format = "T-{a}{b}" }]
[names.vehicle.components]
a = ["A", "B", "C", "D"]
b = ["W", "X", "Y", "Z"]

[labels]
ranks = ["rare", "casual", "regular", "devoted"]
trip_prefix = ["TA"]

[[catalogs.locations]]
name = "North Base"
code = "NBS"
latitude = 53.35
longitude = -2.27
base = true
kind = "city"
weight = 1.0

[[catalogs.locations]]
name = "South Base"
code = "SBS"
latitude = 51.15
longitude = -0.18
base = true
kind = "city"
weight = 1.0

[[catalogs.locations]]
name = "Sunny Coast"
code = "SUN"
latitude = 38.28
longitude = -0.56
base = false
kind = "beach"
weight = 1.0

[[catalogs.locations]]
name = "Snow Peak"
code = "SNO"
latitude = 47.26
longitude = 11.34
base = false
kind = "ski"
weight = 1.0

[[catalogs.locations]]
name = "Old Town"
code = "OLD"
latitude = 52.31
longitude = 4.76
base = false
kind = "city"
weight = 1.0

[[catalogs.vehicle_types]]
name = "Small jet"
capacity = 48
share = 1.0

[[catalogs.add_ons]]
name = "Hold bag"
category = "bags"
price = 2500
attach_rate = 0.3

[[catalogs.add_ons]]
name = "Seat choice"
category = "seats"
price = 800
attach_rate = 0.2

[[catalogs.fare_classes]]
name = "Basic"
multiplier = 1.0
share = 0.9

[[catalogs.fare_classes]]
name = "Flexible"
multiplier = 1.6
share = 0.1

[[catalogs.channels]]
name = "app"
share = 0.6

[[catalogs.channels]]
name = "web"
share = 0.4

[params.travel]
route_density = 1.0
travellers_per_base = 20
"#;

fn generate(directory: &Path, extra: &[&str]) {
    let theme = directory.join("tiny_air.toml");
    std::fs::write(&theme, THEME).unwrap();
    cargo_bin_cmd!("rowing-machine")
        .args([
            "--scenario",
            "travel",
            "--years",
            "1",
            "--scale",
            "1",
            "--seed",
            "42",
            "--quiet",
            "--theme",
        ])
        .arg(&theme)
        .args(extra)
        .arg("--output-dir")
        .arg(directory.join("out"))
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

fn read(directory: &Path, entity: &str) -> Vec<Row> {
    let mut reader =
        csv::Reader::from_path(directory.join("out").join(format!("raw_{entity}.csv"))).unwrap();
    let headers = reader.headers().unwrap().clone();
    reader
        .records()
        .map(|record| {
            headers
                .iter()
                .zip(record.unwrap().iter())
                .map(|(header, value)| (header.to_owned(), value.to_owned()))
                .collect()
        })
        .collect()
}

fn by_id(rows: &[Row]) -> BTreeMap<&str, &Row> {
    rows.iter().map(|row| (row["id"].as_str(), row)).collect()
}

fn cents(row: &Row, column: &str) -> i64 {
    row[column].parse().unwrap()
}

fn minutes(timestamp: &str) -> i64 {
    let time: jiff::civil::DateTime = timestamp.parse().unwrap();
    time.to_zoned(jiff::tz::TimeZone::UTC)
        .unwrap()
        .timestamp()
        .as_second()
        / 60
}

fn month(timestamp: &str) -> i8 {
    timestamp.parse::<jiff::civil::DateTime>().unwrap().month()
}

struct Tables {
    locations: Vec<Row>,
    routes: Vec<Row>,
    vehicles: Vec<Row>,
    add_ons: Vec<Row>,
    trips: Vec<Row>,
    bookings: Vec<Row>,
    tickets: Vec<Row>,
    ticket_add_ons: Vec<Row>,
    travellers: Vec<Row>,
}

fn tables(directory: &Path) -> Tables {
    Tables {
        locations: read(directory, "locations"),
        routes: read(directory, "routes"),
        vehicles: read(directory, "vehicles"),
        add_ons: read(directory, "add_ons"),
        trips: read(directory, "trips"),
        bookings: read(directory, "bookings"),
        tickets: read(directory, "tickets"),
        ticket_add_ons: read(directory, "ticket_add_ons"),
        travellers: read(directory, "travellers"),
    }
}

#[test]
fn every_foreign_key_resolves_and_bookings_add_up() {
    let directory = tempfile::tempdir().unwrap();
    generate(directory.path(), &[]);
    let t = tables(directory.path());
    let locations = by_id(&t.locations);
    let routes = by_id(&t.routes);
    let vehicles = by_id(&t.vehicles);
    let add_ons = by_id(&t.add_ons);
    let trips = by_id(&t.trips);
    let bookings = by_id(&t.bookings);
    let tickets = by_id(&t.tickets);
    let travellers = by_id(&t.travellers);
    for route in &t.routes {
        assert!(locations.contains_key(route["origin_id"].as_str()));
        assert!(locations.contains_key(route["destination_id"].as_str()));
    }
    for vehicle in &t.vehicles {
        assert_eq!(locations[vehicle["base_id"].as_str()]["is_base"], "True");
    }
    for trip in &t.trips {
        assert!(routes.contains_key(trip["route_id"].as_str()));
        assert!(vehicles.contains_key(trip["vehicle_id"].as_str()));
    }
    for booking in &t.bookings {
        assert!(travellers.contains_key(booking["traveller_id"].as_str()));
    }
    let mut totals: BTreeMap<&str, (i64, i64)> = BTreeMap::new();
    let mut departures: BTreeMap<&str, i64> = BTreeMap::new();
    for ticket in &t.tickets {
        assert!(trips.contains_key(ticket["trip_id"].as_str()));
        assert!(bookings.contains_key(ticket["booking_id"].as_str()));
        let entry = totals.entry(ticket["booking_id"].as_str()).or_default();
        entry.0 += cents(ticket, "fare");
        entry.1 += 1;
        departures.insert(
            ticket["booking_id"].as_str(),
            minutes(&trips[ticket["trip_id"].as_str()]["scheduled_departure_at"]),
        );
    }
    for add_on in &t.ticket_add_ons {
        let ticket = tickets[add_on["ticket_id"].as_str()];
        assert_eq!(
            add_ons[add_on["add_on_id"].as_str()]["price"],
            add_on["price"]
        );
        totals.get_mut(ticket["booking_id"].as_str()).unwrap().0 += cents(add_on, "price");
    }
    assert!(!t.bookings.is_empty());
    for booking in &t.bookings {
        let (total, seats) = totals[booking["id"].as_str()];
        assert_eq!(total, cents(booking, "total_price"));
        assert_eq!(seats, cents(booking, "party_size"));
        assert!(minutes(&booking["booked_at"]) < departures[booking["id"].as_str()]);
    }
}

#[test]
fn trips_never_oversell_seats_and_cancellations_refund_every_ticket() {
    let directory = tempfile::tempdir().unwrap();
    generate(directory.path(), &["--param", "cancellation_rate=0.1"]);
    let t = tables(directory.path());
    let trips = by_id(&t.trips);
    let mut seats: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut sold: BTreeMap<&str, i64> = BTreeMap::new();
    for ticket in &t.tickets {
        let trip = trips[ticket["trip_id"].as_str()];
        assert!(
            seats
                .entry(trip["id"].as_str())
                .or_default()
                .insert(ticket["seat"].as_str())
        );
        *sold.entry(trip["id"].as_str()).or_default() += 1;
        let refunded = ticket["status"] == "refunded";
        assert_eq!(refunded, trip["status"] == "cancelled", "{ticket:?}");
    }
    let mut cancelled = 0;
    for trip in &t.trips {
        assert!(sold.get(trip["id"].as_str()).copied().unwrap_or(0) <= cents(trip, "capacity"));
        let flown = trip["status"] == "completed";
        assert_eq!(flown, !trip["departed_at"].is_empty());
        assert_eq!(flown, !trip["arrived_at"].is_empty());
        cancelled += usize::from(!flown);
    }
    assert!(cancelled > 0);
}

#[test]
fn delays_carry_through_each_vehicles_later_trips() {
    let directory = tempfile::tempdir().unwrap();
    generate(directory.path(), &["--param", "delay_rate=0.6"]);
    let t = tables(directory.path());
    let mut by_vehicle_day: BTreeMap<(&str, &str), Vec<&Row>> = BTreeMap::new();
    for trip in t.trips.iter().filter(|trip| trip["status"] == "completed") {
        by_vehicle_day
            .entry((
                trip["vehicle_id"].as_str(),
                &trip["scheduled_departure_at"][..10],
            ))
            .or_default()
            .push(trip);
    }
    let mut knocked_on = 0;
    for trips in by_vehicle_day.values_mut() {
        trips.sort_by_key(|trip| minutes(&trip["scheduled_departure_at"]));
        for pair in trips.windows(2) {
            let ready = minutes(&pair[0]["arrived_at"]) + 30;
            let departed = minutes(&pair[1]["departed_at"]);
            assert!(departed >= ready, "{:?} then {:?}", pair[0], pair[1]);
            knocked_on += usize::from(ready > minutes(&pair[1]["scheduled_departure_at"]));
        }
    }
    assert!(knocked_on > 0, "no delay ever reached a later trip");
}

#[test]
fn late_bookings_pay_more_and_destinations_follow_their_seasons() {
    let directory = tempfile::tempdir().unwrap();
    generate(directory.path(), &[]);
    let t = tables(directory.path());
    let trips = by_id(&t.trips);
    let bookings = by_id(&t.bookings);
    let (mut late, mut early) = ((0, 0), (0, 0));
    for ticket in &t.tickets {
        let booking = bookings[ticket["booking_id"].as_str()];
        let trip = trips[ticket["trip_id"].as_str()];
        let lead = minutes(&trip["scheduled_departure_at"]) - minutes(&booking["booked_at"]);
        let band = if lead < 7 * 1440 {
            &mut late
        } else if lead >= 60 * 1440 {
            &mut early
        } else {
            continue;
        };
        band.0 += cents(ticket, "fare");
        band.1 += 1;
    }
    assert!(
        late.0 * early.1 > early.0 * late.1 * 3 / 2,
        "late {late:?} early {early:?}"
    );
    let locations = by_id(&t.locations);
    let routes = by_id(&t.routes);
    let mut flights: BTreeMap<(&str, i8), usize> = BTreeMap::new();
    for trip in &t.trips {
        let route = routes[trip["route_id"].as_str()];
        let kind = locations[route["destination_id"].as_str()]["kind"].as_str();
        *flights
            .entry((kind, month(&trip["scheduled_departure_at"])))
            .or_default() += 1;
    }
    let count = |kind, month| flights.get(&(kind, month)).copied().unwrap_or(0);
    assert!(count("beach", 7) > count("beach", 1), "{flights:?}");
    assert!(count("ski", 1) > count("ski", 7), "{flights:?}");
}

#[test]
fn loyalty_tiers_rise_with_booking_counts() {
    let directory = tempfile::tempdir().unwrap();
    generate(directory.path(), &[]);
    let t = tables(directory.path());
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for booking in &t.bookings {
        *counts.entry(booking["traveller_id"].as_str()).or_default() += 1;
    }
    assert_eq!(counts.len(), t.travellers.len());
    let tiers = ["rare", "casual", "regular", "devoted"];
    let mut ranked: Vec<_> = t
        .travellers
        .iter()
        .map(|traveller| {
            let tier = tiers
                .iter()
                .position(|tier| *tier == traveller["loyalty_tier"])
                .unwrap();
            (counts[traveller["id"].as_str()], tier)
        })
        .collect();
    ranked.sort_unstable();
    assert!(ranked.windows(2).all(|pair| pair[0].1 <= pair[1].1));
    assert_eq!(ranked.first().unwrap().1, 0);
    assert_eq!(ranked.last().unwrap().1, 3);
}

#[test]
fn frequency_parameter_adds_trips_without_changing_the_network() {
    let base = tempfile::tempdir().unwrap();
    let busy = tempfile::tempdir().unwrap();
    generate(base.path(), &[]);
    generate(busy.path(), &["--param", "daily_frequency=3"]);
    assert_eq!(read(base.path(), "routes"), read(busy.path(), "routes"));
    assert!(read(busy.path(), "trips").len() > read(base.path(), "trips").len() * 2);
}

#[test]
fn travel_defaults_to_the_airline_theme_and_rejects_themes_without_catalogs() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("output");
    let result = cargo_bin_cmd!("rowing-machine")
        .args(["--scenario", "travel", "--theme", "plain", "--output-dir"])
        .arg(&output)
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&result.get_output().stderr);
    for expected in ["catalogs.locations", "names.vehicle", "airline"] {
        assert!(stderr.contains(expected), "{stderr}");
    }
    assert!(!output.exists());
    cargo_bin_cmd!("rowing-machine")
        .args([
            "--scenario",
            "travel",
            "--years",
            "1",
            "--scale",
            "1",
            "--seed",
            "7",
            "--quiet",
            "--param",
            "route_density=0",
            "--param",
            "load_factor=0.05",
            "--output-dir",
        ])
        .arg(&output)
        .assert()
        .success();
    let locations = std::fs::read_to_string(output.join("raw_locations.csv")).unwrap();
    assert!(locations.lines().count() > 30);
}
