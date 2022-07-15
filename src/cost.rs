use crate::{
    gap_cost::{calculate_cottage_gaps2, calculate_gaps},
    instance::Instance,
    problem::Problem,
    solution::Solution,
    upgrade_cost::{calculate_upgrade, calculate_upgrades},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Cost {
    gaps: isize,
    gaps_fri_thu: isize,
    gaps_legionella: isize,
    upgrades: isize,
}

impl Cost {
    pub fn calculate(problem: &Problem, solution: &Solution) -> Self {
        let (gaps, gaps_fri_thu, gaps_legionella) = calculate_gaps(problem, solution);
        let upgrades = calculate_upgrades(problem, solution);

        Self {
            gaps: gaps as isize,
            gaps_fri_thu: gaps_fri_thu as isize,
            gaps_legionella: gaps_legionella as isize,
            upgrades: upgrades as isize,
        }
    }

    pub fn calculate_instance(instance: &Instance) -> Self {
        Self::calculate(&instance.problem, &instance.solution)
    }

    pub fn calculate_single(
        problem: &Problem,
        solution: &Solution,
        reservation: usize,
        cottage: usize,
    ) -> Self {
        let (gaps, gaps_fri_thu, gaps_legionella) =
            calculate_cottage_gaps2(problem, solution, cottage);

        let upgrades = calculate_upgrade(problem, solution, reservation);

        Self {
            gaps: gaps as isize,
            gaps_fri_thu: gaps_fri_thu as isize,
            gaps_legionella: gaps_legionella as isize,
            upgrades: upgrades as isize,
        }
    }

    pub fn objective(&self) -> isize {
        return 6 * self.gaps - 3 * self.gaps_fri_thu + 12 * self.gaps_legionella + self.upgrades;
    }

    pub fn new(gaps: isize, gaps_fri_thu: isize, gaps_legionella: isize, upgrades: isize) -> Self {
        Self {
            gaps,
            gaps_fri_thu,
            gaps_legionella,
            upgrades,
        }
    }

    pub fn empty() -> Self {
        Self {
            gaps: 0,
            gaps_fri_thu: 0,
            gaps_legionella: 0,
            upgrades: 0,
        }
    }
}

impl std::ops::Add for Cost {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            gaps: self.gaps + other.gaps,
            gaps_fri_thu: self.gaps_fri_thu + other.gaps_fri_thu,
            gaps_legionella: self.gaps_legionella + other.gaps_legionella,
            upgrades: self.upgrades + other.upgrades,
        }
    }
}

impl std::ops::AddAssign for Cost {
    fn add_assign(&mut self, rhs: Self) {
        self.gaps += rhs.gaps;
        self.gaps_fri_thu += rhs.gaps_fri_thu;
        self.gaps_legionella += rhs.gaps_legionella;
        self.upgrades += rhs.upgrades;
    }
}

impl std::ops::Sub for Cost {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            gaps: self.gaps - rhs.gaps,
            gaps_fri_thu: self.gaps_fri_thu - rhs.gaps_fri_thu,
            gaps_legionella: self.gaps_legionella - rhs.gaps_legionella,
            upgrades: self.upgrades - rhs.upgrades,
        }
    }
}

impl std::ops::SubAssign for Cost {
    fn sub_assign(&mut self, rhs: Self) {
        self.gaps -= rhs.gaps;
        self.gaps_fri_thu -= rhs.gaps_fri_thu;
        self.gaps_legionella -= rhs.gaps_legionella;
        self.upgrades -= rhs.upgrades;
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_calculate_cost() {}
}
