use std::ops::Range;

use crate::{cost::Cost, problem::Problem, solution::Solution};

#[derive(Debug)]
pub struct Target {
    pub reservation: usize,
    pub cottage: usize,
    pub range: Range<usize>,
}

#[derive(Debug)]
pub struct Chain {
    chain: Vec<Modification>,
    cost: Cost,
    step: usize,
}

impl Chain {
    pub fn new() -> Self {
        Self {
            chain: Vec::with_capacity(4096),
            cost: Cost::empty(),
            step: 0,
        }
    }

    pub fn progress(&mut self, problem: &Problem, solution: &mut Solution) {
        #[cfg(feature = "supersafe")]
        {
            assert!(self.chain.get(self.step..).is_some())
        }

        unsafe {
            for modification in self.chain.get_unchecked(self.step..).iter() {
                self.cost += modification.progress(problem, solution);
            }
        }

        self.step = self.chain.len();
    }

    pub fn regress(&mut self, problem: &Problem, solution: &mut Solution) {
        for modification in self.chain.iter().rev() {
            self.cost += modification.regress(problem, solution);
        }
        self.step = 0;
    }

    pub fn add(&mut self, modification: Modification) {
        self.chain.push(modification);
    }

    pub fn len(&self) -> usize {
        self.chain.len()
    }

    pub fn clear(&mut self) {
        self.chain.clear();
        self.cost = Cost::empty();
        self.step = 0;
    }

    pub fn cost(&self) -> &Cost {
        &self.cost
    }
}

impl Target {
    pub fn new(reservation: usize, cottage: usize, range: Range<usize>) -> Self {
        Self {
            reservation,
            cottage,
            range,
        }
    }
}

#[derive(Debug)]
pub enum Modification {
    Unassign(Target),
    Assign(Target),
}

impl Modification {
    pub fn regress(&self, problem: &Problem, solution: &mut Solution) -> Cost {
        let target = self.target();
        let cost = Cost::calculate_single(problem, solution, target.reservation, target.cottage);

        match self {
            Modification::Unassign(target) => {
                solution.assign(target.cottage, target.reservation, target.range.clone())
            }
            Modification::Assign(target) => {
                solution.unassign(target.cottage, target.reservation, target.range.clone())
            }
        }

        let mut end_cost =
            Cost::calculate_single(problem, solution, target.reservation, target.cottage);
        end_cost -= cost;
        end_cost
    }

    pub fn progress(&self, problem: &Problem, solution: &mut Solution) -> Cost {
        let target = self.target();
        let cost = Cost::calculate_single(problem, solution, target.reservation, target.cottage);
        match self {
            Modification::Unassign(_) => {
                solution.unassign(target.cottage, target.reservation, target.range.clone())
            }
            Modification::Assign(_) => {
                solution.assign(target.cottage, target.reservation, target.range.clone())
            }
        }

        let mut end_cost =
            Cost::calculate_single(problem, solution, target.reservation, target.cottage);
        end_cost -= cost;
        end_cost
    }

    pub fn unassign(reservation: usize, problem: &Problem, solution: &Solution) -> Self {
        #[cfg(feature = "supersafe")]
        {
            assert!(solution.mapping().get(reservation).is_some());
            assert!(solution.mapping().get(reservation).unwrap().is_some());
        }

        let cottage = unsafe {
            solution
                .mapping()
                .get_unchecked(reservation)
                .unwrap_unchecked()
        };
        let range = problem.reservations.range(reservation);
        let target = Target::new(reservation, cottage, range);
        Self::Unassign(target)
    }

    pub fn assign(reservation: usize, cottage: usize, problem: &Problem) -> Self {
        let range = problem.reservations.range(reservation);
        let target = Target::new(reservation, cottage, range);
        Self::Assign(target)
    }

    pub fn target(&self) -> &Target {
        match self {
            Modification::Unassign(target) => &target,
            Modification::Assign(target) => &target,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::problem::{Cottages, Reservations};

    use super::*;
    #[test]
    fn test_assign_cost() {
        let mut cottages = Cottages::empty(2);
        cottages.class = vec![0, 1];

        let mut reservations = Reservations::empty(2);
        reservations.arrival = vec![0, 5];
        reservations.stay = vec![5, 5];
        reservations.class = vec![0, 0];
        reservations.update();

        let problem = Problem::new(cottages, reservations, 0);
        let mut solution = Solution::naive(&problem);

        let mut chain = Chain::new();
        chain.add(Modification::unassign(0, &problem, &solution));
        chain.add(Modification::unassign(1, &problem, &solution));

        chain.add(Modification::assign(0, 0, &problem));
        chain.add(Modification::assign(1, 0, &problem));

        for _ in 0..100 {
            chain.progress(&problem, &mut solution);
            assert_eq!(&chain.cost, &Cost::new(-1, 1, 0, -1));

            chain.regress(&problem, &mut solution);
            assert_eq!(&chain.cost, &Cost::empty());
        }
    }
}
