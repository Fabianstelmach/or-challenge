use fastrand::Rng;

use crate::{
    modification::{Chain, Modification},
    problem::Problem,
    solution::Solution,
};

#[derive(Debug)]
pub struct Instance {
    pub problem: Problem,
    pub solution: Solution,
    pub chain: Chain,
    rng: Rng,
}

impl Instance {
    #[inline(always)]
    fn rand(&self, slice: &[usize]) -> usize {
        self.rng.usize(..slice.len())
    }

    #[inline(always)]
    fn rand_index(&self, slice: &[usize]) -> usize {
        let index = self.rand(slice);
        unsafe { *slice.get_unchecked(index) }
    }

    pub fn new(problem: Problem, solution: Solution) -> Self {
        Self {
            problem,
            solution,
            chain: Chain::new(),
            rng: Rng::new(),
        }
    }

    pub fn neighbor(&mut self) {
        let reservation =
            self.rand_index(&self.problem.allocation_penalty.reallocatable_reservations[..]);

        let modification = Modification::unassign(reservation, &self.problem, &self.solution);

        self.chain.add(modification);
        self.chain.progress(&self.problem, &mut self.solution);
        self.repair();
    }

    pub fn get_random_cottage(&mut self, reservation: usize) -> usize {
        let cottages = self
            .problem
            .allocation_penalty
            .possible_targets(reservation);

        self.rand_index(cottages)
    }

    pub fn accept_chain(&mut self) {
        self.chain.clear();
    }

    pub fn reject_chain(&mut self) {
        self.chain.regress(&self.problem, &mut self.solution);
        self.chain.clear();
    }

    fn random_unallocated(&self) -> usize {
        let unallocated = self.solution.unallocated();
        self.rand_index(unallocated)
    }

    fn repair_single(&mut self) {
        let reservation = self.random_unallocated();
        let cottage = self.get_random_cottage(reservation);

        let modification = Modification::assign(reservation, cottage, &self.problem);

        let prio_new = self.problem.allocation_penalty.priority[reservation];
        let prio_old: f64 = self
            .solution
            .is_taken_by(cottage, modification.target().range.clone())
            .map(|x| self.problem.allocation_penalty.priority[x])
            .sum();
        // .fold(0f64, f64::max);

        if prio_old > prio_new {
            if self.rng.f64() > 0.005 {
                return;
            }
        }

        // if self
        //     .solution
        //     .is_taken_by(cottage, modification.target().range.clone())
        //     .count()
        //     > 0
        // {
        //     if self.rng.f64() > 0.005 {
        //         return;
        //     }
        // }

        // if prio_old > prio_new {
        //     let p = prio_new / prio_old;
        //     if fastrand::f64() > p * 0.01 {
        //         return;
        //     }
        // }

        self.solution
            .is_taken_by(cottage, modification.target().range.clone())
            .map(|reservation| Modification::unassign(reservation, &self.problem, &self.solution))
            .for_each(|modification| self.chain.add(modification));

        self.chain.add(modification);
        self.chain.progress(&self.problem, &mut self.solution);
    }

    fn repair_single_force(&mut self) {
        let reservation = self.random_unallocated();
        let cottage = self.get_random_cottage(reservation);

        let modification = Modification::assign(reservation, cottage, &self.problem);

        self.solution
            .is_taken_by(cottage, modification.target().range.clone())
            .map(|reservation| Modification::unassign(reservation, &self.problem, &self.solution))
            .for_each(|modification| self.chain.add(modification));

        self.chain.add(modification);
        self.chain.progress(&self.problem, &mut self.solution);
    }

    pub fn repair(&mut self) {
        self.repair_single_force();

        while !self.solution.unallocated().is_empty() {
            self.repair_single();
        }
    }
}

#[cfg(test)]
mod test {

    use crate::problem::{Cottages, Reservations};

    use super::*;
    #[test]
    fn test_neighbor() {
        let problem = Problem::empty(2, 1);
        let solution = Solution::naive(&problem);
        let mut instance = Instance::new(problem, solution);
        instance.neighbor();
        assert_eq!(instance.solution.mapping(), vec![Some(1)]);
        instance
            .chain
            .regress(&instance.problem, &mut instance.solution);
        assert_eq!(instance.solution.mapping(), vec![Some(0)]);
        instance
            .chain
            .progress(&instance.problem, &mut instance.solution);
        assert_eq!(instance.solution.mapping(), vec![Some(1)]);
        instance
            .chain
            .regress(&instance.problem, &mut instance.solution);
        assert_eq!(instance.solution.mapping(), vec![Some(0)]);
    }

    #[test]
    fn test_neighbor_evicted() {
        let problem = Problem::empty(2, 2);
        let solution = Solution::naive(&problem);
        let mut instance = Instance::new(problem, solution);
        instance.neighbor();
        assert_eq!(instance.solution.mapping(), vec![Some(1), Some(0)]);
        instance
            .chain
            .regress(&instance.problem, &mut instance.solution);
        assert_eq!(instance.solution.mapping(), vec![Some(0), Some(1)]);
        instance
            .chain
            .progress(&instance.problem, &mut instance.solution);
        assert_eq!(instance.solution.mapping(), vec![Some(1), Some(0)]);
        instance
            .chain
            .regress(&instance.problem, &mut instance.solution);
        assert_eq!(instance.solution.mapping(), vec![Some(0), Some(1)]);
    }

    #[test]
    fn test_repair() {
        let size = 8;

        let mut cottages = Cottages::empty(size);
        let mut reservations = Reservations::empty(size);
        cottages.class = (0..size).collect();
        reservations.class = (0..size).collect();

        let problem = Problem::new(cottages, reservations, 0);
        let solution = Solution::empty(&problem);
        let mut instance = Instance::new(problem, solution);

        instance.repair();
        assert!(instance.solution.unallocated().is_empty());
    }

    // #[test]
    // fn test_repair_large() {
    //     let size = 2500;
    //     let s2 = (size as f64 * 1.01) as usize;
    //     let problem = Problem::empty(s2, size);

    //     let solution = Solution::empty(&problem);
    //     let mut instance = Instance::new(problem, solution);
    //     instance.repair();

    //     let x = std::time::Instant::now();
    //     for _ in 0..4000 {
    //         instance.neighbor();
    //     }
    //     dbg!(1000 * 4000 / x.elapsed().as_millis());
    //     assert!(instance.solution.unallocated().is_empty());
    // }
}
