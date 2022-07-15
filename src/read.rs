use std::fs::read_to_string;

use crate::problem::{Cottages, Problem, Reservations};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct CottageItem {
    #[serde(alias = "ID")]
    id: usize,

    #[serde(alias = "Max # Pers")]
    capacity: usize,

    #[serde(alias = "Class")]
    class: usize,

    #[serde(alias = "Face South")]
    face_south: usize,

    #[serde(alias = "Near Playground")]
    near_playground: usize,

    #[serde(alias = "Close to the Centre")]
    close_to_the_center: usize,

    #[serde(alias = "Near Lake ")]
    near_lake: usize,

    #[serde(alias = "Near car park")]
    near_car_park: usize,

    #[serde(alias = "Accessible for Wheelchair")]
    accessible_for_wheelchairs: usize,

    #[serde(alias = "Child Friendly")]
    child_friendly: usize,

    #[serde(alias = "Dish Washer ")]
    dish_washer: usize,

    #[serde(alias = "Wi-Fi Coverage ")]
    wifi_coverage: usize,

    #[serde(alias = "Covered Terrace")]
    covered_terrace: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReservationItem {
    #[serde(alias = "ID")]
    id: usize,

    #[serde(alias = "Arrival Date")]
    arrival: i64,

    #[serde(alias = "Length of Stay")]
    stay: usize,

    #[serde(alias = "# Persons")]
    people: usize,

    #[serde(alias = "Class")]
    class: usize,

    #[serde(alias = "Face South")]
    face_south: usize,

    #[serde(alias = "Near Playground")]
    near_playground: usize,

    #[serde(alias = "Close to the Centre")]
    close_to_the_center: usize,

    #[serde(alias = "Near Lake ")]
    near_lake: usize,

    #[serde(alias = "Near car park")]
    near_car_park: usize,

    #[serde(alias = "Accessible for Wheelchair")]
    accessible_for_wheelchairs: usize,

    #[serde(alias = "Child Friendly")]
    child_friendly: usize,

    #[serde(alias = "Dish Washer ")]
    dish_washer: usize,

    #[serde(alias = "Wi-Fi Coverage ")]
    wifi_coverage: usize,

    #[serde(alias = "Covered Terrace")]
    covered_terrace: usize,

    #[serde(alias = "Cottage (Fixed)")]
    cottage_number: usize,
}

pub fn read_cottages_json(path: String) -> Cottages {
    let json_string = read_to_string(path).expect("File not found");
    let json: Vec<CottageItem> = serde_json::from_str(&json_string).unwrap();

    let id = json.iter().map(|x| x.id).collect();
    let capacity = json.iter().map(|x| x.capacity).collect();
    let class = json.iter().map(|x| x.class).collect();
    let preference = json
        .iter()
        .map(|x| {
            [
                x.face_south == 1,
                x.near_playground == 1,
                x.close_to_the_center == 1,
                x.near_lake == 1,
                x.near_car_park == 1,
                x.accessible_for_wheelchairs == 1,
                x.child_friendly == 1,
                x.dish_washer == 1,
                x.wifi_coverage == 1,
                x.covered_terrace == 1,
            ]
        })
        .collect();

    Cottages::new(id, capacity, class, preference)
}

// TODO: Fix this
fn map_people(i: usize) -> usize {
    match i {
        1 => 2,
        2 => 2,
        3 => 4,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 8,
        8 => 8,
        9 => 12,
        10 => 12,
        11 => 12,
        12 => 12,
        _ => todo!(),
    }
}

pub fn read_reservations_json(path: String) -> (usize, Reservations) {
    let json_string = read_to_string(path).expect("File not found");
    let json: Vec<ReservationItem> = serde_json::from_str(&json_string).unwrap();

    let datetime_0 = Utc.timestamp_millis(json[0].arrival);
    let phase = (datetime_0.weekday() as usize + 7 - 4).rem_euclid(7);

    let id = json.iter().map(|x| x.id).collect();
    let arrival: Vec<usize> = json
        .iter()
        .map(|x| (Utc.timestamp_millis(x.arrival) - datetime_0).num_days() as usize)
        .collect();

    let mut stay: Vec<usize> = json.iter().map(|x| x.stay).collect();
    let people = json.iter().map(|x| map_people(x.people)).collect();
    let class = json.iter().map(|x| x.class).collect();
    let preference = json
        .iter()
        .map(|x| {
            [
                x.face_south == 1,
                x.near_playground == 1,
                x.close_to_the_center == 1,
                x.near_lake == 1,
                x.near_car_park == 1,
                x.accessible_for_wheelchairs == 1,
                x.child_friendly == 1,
                x.dish_washer == 1,
                x.wifi_coverage == 1,
                x.covered_terrace == 1,
            ]
        })
        .collect();
    let cottage_number = json
        .iter()
        .map(|x| match x.cottage_number {
            0 => None,
            x => Some(x - 1),
        })
        .collect();

    // TODO: Fix!
    let latest_arrival = arrival.iter().max().unwrap();
    for reservation in 0..arrival.len() {
        if arrival[reservation] + stay[reservation] > *latest_arrival {
            stay[reservation] = latest_arrival - arrival[reservation] + 1;
        }
    }

    (
        phase,
        Reservations::new(id, arrival, stay, people, class, preference, cottage_number),
    )
}

pub fn read_problem_json(cottages_path: String, reservations_path: String) -> Problem {
    let cottages = read_cottages_json(cottages_path);
    let (phase, reservations) = read_reservations_json(reservations_path);

    Problem::new(cottages, reservations, phase)
}
