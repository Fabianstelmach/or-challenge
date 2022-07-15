use std::collections::HashSet;

use crate::problem::{Cottages, Reservations};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Copy)]
pub enum Penalty {
    Free,
    Upgrade,
    Impossible,
}

#[derive(Debug, Clone)]
pub struct AllocationPenalty {
    // reservations x cottages
    // For each reservation/cottage pair,
    // what is the penalty for this allocation?
    pub allocation_penalty: Vec<Penalty>,

    pub reallocatable: Vec<bool>,
    pub reallocatable_reservations: Vec<usize>,

    pub possible_targets: Vec<Vec<usize>>,
    pub priority: Vec<f64>,

    pub cottages: usize,
    pub reservations: usize,
}

impl AllocationPenalty {
    pub fn get_penalty_arr(&self, reservation: usize) -> &[Penalty] {
        #[cfg(feature = "supersafe")]
        assert!((reservation + 1) * self.cottages <= self.allocation_penalty.len());
        unsafe {
            &self
                .allocation_penalty
                .get_unchecked(reservation * self.cottages..(reservation + 1) * self.cottages)
        }
    }

    pub fn get(&self, cottage: usize, reservation: usize) -> Penalty {
        #[cfg(feature = "supersafe")]
        assert!((reservation) * self.cottages + cottage < self.allocation_penalty.len());
        unsafe { *self.get_penalty_arr(reservation).get_unchecked(cottage) }
    }

    pub fn possible_targets(&self, reservation: usize) -> &[usize] {
        #[cfg(feature = "supersafe")]
        assert!(self.possible_targets.get(reservation).is_some());

        unsafe { &self.possible_targets.get_unchecked(reservation)[..] }
    }

    fn possible_targets_iter(&self, reservation: usize) -> impl Iterator<Item = usize> + '_ {
        self.get_penalty_arr(reservation)
            .iter()
            .enumerate()
            .filter(|x| x.1 != &Penalty::Impossible)
            .map(|x| x.0)
    }

    fn calculate_possible_targets(&mut self) {
        for reservation in 0..self.reallocatable.len() {
            let v: Vec<_> = self.possible_targets_iter(reservation).collect();
            self.possible_targets.push(v);
        }
    }

    fn calculate_priority(&mut self, reservations: &Reservations) {
        let mut priority = vec![0f64; reservations.reservations()];

        let mut possible_targets_hash = vec![HashSet::new(); reservations.reservations()];
        for reservation in 0..reservations.reservations() {
            let possible_target_hash: HashSet<_> =
                HashSet::from_iter(self.possible_targets[reservation].iter());
            possible_targets_hash[reservation] = possible_target_hash;
        }

        for reservation in 0..reservations.reservations() {
            let reservation_targets = &possible_targets_hash[reservation];

            let mut actual_overlaps = 0;
            for overlap in reservations.get_overlaps(reservation) {
                let overlap_possible_targets = &possible_targets_hash[overlap];
                let intersection = overlap_possible_targets.intersection(&reservation_targets);
                let count = intersection.count();

                if count != 0 {
                    actual_overlaps += 1;
                    priority[reservation] += count as f64 / reservation_targets.len() as f64;
                }
            }

            if actual_overlaps != 0 {
                priority[reservation] /= actual_overlaps as f64;
            }
        }
        self.priority = priority;
    }

    pub fn calculate(cottages: &Cottages, reservations: &Reservations) -> Self {
        let mut allocation_penalty =
            vec![Penalty::Free; cottages.cottages() * reservations.reservations()];

        let mut reallocatable = vec![true; reservations.reservations()];

        let index =
            |reservation: usize, cottage: usize| reservation * cottages.cottages() + cottage;

        // Reservations with pre-defined cottage can only be allocated to that one
        for reservation in 0..reservations.reservations() {
            let forced_cottage = reservations.cottage_number.get(reservation).unwrap();
            if let Some(cottage) = forced_cottage {
                let start = index(reservation, 0);
                let end = index(reservation + 1, 0);
                let _ = &allocation_penalty[start..end].fill(Penalty::Impossible);
                allocation_penalty[start + *cottage] = Penalty::Free;
            }
        }

        // Reservation/cottage can be incompatible due to preferences
        for reservation in 0..reservations.reservations() {
            let reservation_preferences = reservations.preference[reservation];
            for cottage in 0..cottages.cottages() {
                let cottage_preferences = cottages.preference[cottage];
                let preferences_met = reservation_preferences
                    .iter()
                    .zip(cottage_preferences)
                    .all(|(rp, cp)| *rp <= cp);

                if !preferences_met {
                    allocation_penalty[index(reservation, cottage)] = Penalty::Impossible;
                }
            }
        }

        // Class mismatch
        for reservation in 0..reservations.reservations() {
            let reservation_class = reservations.class[reservation];
            for cottage in 0..cottages.cottages() {
                let cottage_class = cottages.class[cottage];
                let ap = match reservation_class.cmp(&cottage_class) {
                    std::cmp::Ordering::Less => Penalty::Upgrade,
                    std::cmp::Ordering::Equal => Penalty::Free,
                    std::cmp::Ordering::Greater => Penalty::Impossible,
                };
                if ap > allocation_penalty[index(reservation, cottage)] {
                    allocation_penalty[index(reservation, cottage)] = ap;
                }
            }
        }

        // Size mismatch
        for reservation in 0..reservations.reservations() {
            let reservation_people = reservations.people[reservation];
            for cottage in 0..cottages.cottages() {
                let cottage_people = cottages.capacity[cottage];
                let ap = match reservation_people.cmp(&cottage_people) {
                    std::cmp::Ordering::Less => Penalty::Upgrade,
                    std::cmp::Ordering::Equal => Penalty::Free,
                    std::cmp::Ordering::Greater => Penalty::Impossible,
                };
                if ap > allocation_penalty[index(reservation, cottage)] {
                    allocation_penalty[index(reservation, cottage)] = ap;
                }
            }
        }

        // Propagate pre-defined cottage knowledge
        // If a reservation can be assigned to only a single cottage,
        // Then overlapping reservations can not
        // TODO: Do this for 2-pairs

        loop {
            let mut count = 0;
            'r: for reservation in 0..reservations.reservations() {
                let penalties =
                    &allocation_penalty[index(reservation, 0)..index(reservation + 1, 0)];

                let mut possible_count = 0;
                let mut cottage_number = 0;
                for (i, penalty) in penalties.iter().enumerate() {
                    match penalty {
                        Penalty::Impossible => (),
                        _ if possible_count > 0 => continue 'r,
                        _ => {
                            possible_count += 1;
                            cottage_number = i
                        }
                    }
                }
                if possible_count == 1 {
                    reallocatable[reservation] = false;
                    let overlaps = reservations.get_overlaps(reservation);

                    for overlap in overlaps {
                        if allocation_penalty[index(overlap, cottage_number)] != Penalty::Impossible
                        {
                            allocation_penalty[index(overlap, cottage_number)] =
                                Penalty::Impossible;
                            count += 1;
                        }
                    }
                }
            }
            if count == 0 {
                break;
            }
        }

        let reallocatable_reservations: Vec<_> = reallocatable
            .iter()
            .enumerate()
            .filter(|x| *x.1)
            .map(|x| x.0)
            .collect();

        let mut ap = Self {
            allocation_penalty,
            cottages: cottages.cottages(),
            reservations: reservations.reservations(),
            reallocatable,
            reallocatable_reservations,
            possible_targets: Vec::new(),
            priority: Vec::new(),
        };

        ap.calculate_possible_targets();
        ap.calculate_priority(&reservations);

        ap
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_alloc_penalty_forced_cottage() {
        let cottages = Cottages::empty(4);
        let mut reservations = Reservations::empty(3);
        reservations.cottage_number = vec![Some(0), None, Some(2)];

        let allocation_penalty = AllocationPenalty::calculate(&cottages, &reservations);

        assert_eq!(
            allocation_penalty.get_penalty_arr(0),
            vec![
                Penalty::Free,
                Penalty::Impossible,
                Penalty::Impossible,
                Penalty::Impossible,
            ]
        );

        assert_eq!(
            allocation_penalty.get_penalty_arr(1),
            vec![
                Penalty::Impossible,
                Penalty::Free,
                Penalty::Impossible,
                Penalty::Free,
            ]
        );

        assert_eq!(
            allocation_penalty.get_penalty_arr(2),
            vec![
                Penalty::Impossible,
                Penalty::Impossible,
                Penalty::Free,
                Penalty::Impossible,
            ]
        );

        assert_eq!(allocation_penalty.reallocatable, vec![false, true, false]);
        assert_eq!(allocation_penalty.reallocatable_reservations, vec![1]);
    }

    #[test]
    fn test_alloc_penalty_derived() {
        let cottages = Cottages::empty(2);
        let mut reservations = Reservations::empty(2);
        reservations.cottage_number = vec![Some(0), None];

        let allocation_penalty = AllocationPenalty::calculate(&cottages, &reservations);

        assert_eq!(
            allocation_penalty.get_penalty_arr(0),
            vec![Penalty::Free, Penalty::Impossible,]
        );

        assert_eq!(
            allocation_penalty.get_penalty_arr(1),
            vec![Penalty::Impossible, Penalty::Free,]
        );

        assert_eq!(allocation_penalty.reallocatable, vec![false, false]);
        let x: Vec<usize> = vec![];
        assert_eq!(allocation_penalty.reallocatable_reservations, x);
    }

    #[test]
    fn test_alloc_penalty_preferences() {
        let mut preferences = [false; 10];
        preferences[5] = true;

        let mut cottages = Cottages::empty(3);
        cottages.preference = vec![[false; 10], preferences.clone(), [true; 10]];

        let mut reservations = Reservations::empty(1);
        reservations.preference = vec![preferences];

        let allocation_penalty = AllocationPenalty::calculate(&cottages, &reservations);

        assert_eq!(
            allocation_penalty.get_penalty_arr(0),
            vec![Penalty::Impossible, Penalty::Free, Penalty::Free]
        );

        assert_eq!(allocation_penalty.reallocatable, vec![true]);
        assert_eq!(allocation_penalty.reallocatable_reservations, vec![0]);
    }

    #[test]
    fn test_alloc_penalty_sizes() {
        let mut cottages = Cottages::empty(3);
        cottages.capacity = vec![4, 6, 8];

        let mut reservations = Reservations::empty(1);
        reservations.people = vec![6];

        let allocation_penalty = AllocationPenalty::calculate(&cottages, &reservations);

        assert_eq!(
            allocation_penalty.get_penalty_arr(0),
            vec![Penalty::Impossible, Penalty::Free, Penalty::Upgrade]
        );

        assert_eq!(allocation_penalty.reallocatable, vec![true]);
        assert_eq!(allocation_penalty.reallocatable_reservations, vec![0]);
    }

    #[test]
    fn test_alloc_penalty_class() {
        let mut cottages = Cottages::empty(3);
        cottages.class = vec![1, 2, 3];

        let mut reservations = Reservations::empty(1);
        reservations.class = vec![2];

        let allocation_penalty = AllocationPenalty::calculate(&cottages, &reservations);

        assert_eq!(
            allocation_penalty.get_penalty_arr(0),
            vec![Penalty::Impossible, Penalty::Free, Penalty::Upgrade]
        );

        assert_eq!(allocation_penalty.reallocatable, vec![true]);
        assert_eq!(allocation_penalty.reallocatable_reservations, vec![0]);
    }
}
