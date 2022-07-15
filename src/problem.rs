use std::ops::Range;

use crate::allocation_penalty::AllocationPenalty;

#[derive(Debug, Clone)]
pub struct Cottages {
    pub id: Vec<usize>,
    pub capacity: Vec<usize>,
    pub class: Vec<usize>,
    pub preference: Vec<[bool; 10]>,
}

impl Cottages {
    pub fn empty(size: usize) -> Self {
        Self {
            id: vec![0; size],
            capacity: vec![0; size],
            class: vec![0; size],
            preference: vec![[false; 10]; size],
        }
    }

    pub fn cottages(&self) -> usize {
        self.id.len()
    }

    pub fn new(
        id: Vec<usize>,
        capacity: Vec<usize>,
        class: Vec<usize>,
        preference: Vec<[bool; 10]>,
    ) -> Self {
        Self {
            id,
            capacity,
            class,
            preference,
        }
    }
}

impl Reservations {
    pub fn empty(size: usize) -> Self {
        let arrival = vec![0; size];
        let stay = vec![1; size];
        let departure = Self::calculate_departure(&arrival, &stay);
        Self {
            id: vec![0; size],
            arrival,
            stay,
            departure,
            people: vec![0; size],
            class: vec![0; size],
            preference: vec![[false; 10]; size],
            cottage_number: vec![None; size],
        }
    }

    pub fn new(
        id: Vec<usize>,
        arrival: Vec<usize>,
        stay: Vec<usize>,
        people: Vec<usize>,
        class: Vec<usize>,
        preference: Vec<[bool; 10]>,
        cottage_number: Vec<Option<usize>>,
    ) -> Self {
        let departure = Self::calculate_departure(&arrival, &stay);
        Self {
            id,
            arrival,
            stay,
            departure,
            people,
            class,
            preference,
            cottage_number,
        }
    }

    pub fn validate(&self) {
        assert_eq!(self.arrival.len(), self.stay.len());
        assert_eq!(self.id.len(), self.arrival.len());
        assert_eq!(self.arrival.len(), self.stay.len());
        assert_eq!(self.stay.len(), self.departure.len());
        assert_eq!(self.departure.len(), self.people.len());
        assert_eq!(self.people.len(), self.class.len());
        assert_eq!(self.class.len(), self.preference.len());
        assert_eq!(self.preference.len(), self.cottage_number.len());
    }

    pub fn update(&mut self) {
        self.departure = Self::calculate_departure(&self.arrival, &self.stay);
    }

    pub fn calculate_departure(arrival: &Vec<usize>, stay: &Vec<usize>) -> Vec<usize> {
        arrival
            .iter()
            .zip(stay.iter())
            .map(|(arrival, stay)| arrival + stay)
            .collect()
    }

    pub fn reservations(&self) -> usize {
        self.id.len()
    }

    pub fn timeslots(&self) -> usize {
        self.arrival
            .iter()
            .zip(self.stay.iter())
            .map(|(arrival, stay)| arrival + stay)
            .max()
            .unwrap()
    }

    pub fn get_overlaps(&self, reservation: usize) -> impl Iterator<Item = usize> + '_ {
        let target_arrival = self.arrival[reservation];
        let target_departure = self.departure[reservation];

        self.arrival
            .iter()
            .zip(self.departure.iter())
            .enumerate()
            .filter(move |(i, _)| *i != reservation)
            .filter(move |(_, (arrival, departure))| {
                **arrival < target_departure && target_arrival < **departure
            })
            .map(|(i, _)| i)
    }

    pub fn range(&self, reservation: usize) -> Range<usize> {
        #[cfg(feature = "supersafe")]
        {
            assert!(self.arrival.get(reservation).is_some());
            assert!(self.departure.get(reservation).is_some());
        }

        unsafe {
            *self.arrival.get_unchecked(reservation)..*self.departure.get_unchecked(reservation)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Reservations {
    pub id: Vec<usize>,
    pub arrival: Vec<usize>,
    pub stay: Vec<usize>,
    pub departure: Vec<usize>,
    pub people: Vec<usize>,
    pub class: Vec<usize>,
    pub preference: Vec<[bool; 10]>,
    pub cottage_number: Vec<Option<usize>>,
}

#[derive(Debug, Clone)]
pub struct Problem {
    pub cottages: Cottages,
    pub reservations: Reservations,
    // How many days away from friday is the first day?
    // 0: first day is friday
    // 1: first day is thursday
    pub phase: usize,
    pub allocation_penalty: AllocationPenalty,
}

impl Problem {
    pub fn empty(cottages: usize, reservations: usize) -> Self {
        let cottages = Cottages::empty(cottages);
        let reservations = Reservations::empty(reservations);
        Self::new(cottages, reservations, 0)
    }

    pub fn new(cottages: Cottages, reservations: Reservations, phase: usize) -> Self {
        let allocation_penalty = AllocationPenalty::calculate(&cottages, &reservations);
        Self {
            cottages,
            reservations,
            phase,
            allocation_penalty,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn timeslots_minimal() {
        let size = 1;
        let mut problem = Problem::empty(size, size);
        problem.reservations.arrival = vec![0];
        problem.reservations.stay = vec![1];

        let timeslots = problem.reservations.timeslots();
        assert_eq!(timeslots, 1);
    }

    #[test]
    fn timeslots_mixed() {
        let size = 3;
        let mut problem = Problem::empty(size, size);
        problem.reservations.arrival = vec![0, 1];
        problem.reservations.stay = vec![2, 2];

        let timeslots = problem.reservations.timeslots();
        assert_eq!(timeslots, 3);
    }
}
