use crate::{allocation_penalty::Penalty, problem::Problem, solution::Solution};

// pub fn calculate_upgrades2(problem: &Problem, solution: &Solution) -> usize {
//     let reservation_people = problem.reservations.people.iter();
//     let solution_people = solution.index(&problem.cottages.capacity);

//     let sizes = reservation_people
//         .zip(solution_people)
//         .map(|(rs, ss)| (ss.unwrap_or(usize::MAX) > *(rs)) as usize);

//     let solution_classes = solution.index(&problem.cottages.class);
//     let reservation_classes = problem.reservations.class.iter();

//     let classes = reservation_classes
//         .zip(solution_classes)
//         .map(|(rc, sc)| (sc.unwrap_or(usize::MAX) > *rc) as usize);

//     return sizes.zip(classes).fold(0, |state, (size, class)| {
//         state + (size > 0 || class > 0) as usize
//     });
// }

// pub fn calculate_upgrade(problem: &Problem, solution: &Solution, reservation: usize) -> usize {
//     let cottage = solution.get_cottage(reservation);

//     let (reservation_people, reservation_class) =
//         problem.reservations.get_upgrade_data(reservation);

//     let (solution_people, solution_class) = cottage
//         .map(|c| problem.cottages.get_upgrade_data(c))
//         .unwrap_or((usize::MAX, usize::MAX));

//     return ((solution_people > reservation_people) || (solution_class > reservation_class))
//         as usize;
// }

pub fn calculate_upgrades(problem: &Problem, solution: &Solution) -> usize {
    solution
        .mapping()
        .iter()
        .enumerate()
        .filter_map(|(reservation, cottage)| {
            cottage.map(|cottage| problem.allocation_penalty.get(cottage, reservation))
        })
        .filter(|&x| x == Penalty::Upgrade)
        .count()
}

pub fn calculate_upgrade(problem: &Problem, solution: &Solution, reservation: usize) -> usize {
    let cottage = solution.mapping()[reservation];
    cottage
        .map(|cottage| {
            (problem.allocation_penalty.get(cottage, reservation) == Penalty::Upgrade) as usize
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod test {
    use crate::problem::{Cottages, Reservations};

    use super::*;

    #[test]
    fn test_class_no_cost() {
        let size = 1;
        let mut problem = Problem::empty(size, size);
        let solution = Solution::naive(&problem);

        problem.cottages.class = vec![1];
        problem.reservations.class = vec![1];

        let upgrade_cost = calculate_upgrades(&problem, &solution);
        assert_eq!(upgrade_cost, 0);

        let upgrade_cost = calculate_upgrade(&problem, &solution, 0);
        assert_eq!(upgrade_cost, 0);
    }

    #[test]
    fn test_class_cost() {
        let size = 1;

        let mut cottages = Cottages::empty(size);
        let mut reservations = Reservations::empty(size);
        cottages.class = vec![4];
        reservations.class = vec![1];

        let problem = Problem::new(cottages, reservations, 0);
        let solution = Solution::naive(&problem);

        let upgrade_cost = calculate_upgrades(&problem, &solution);
        assert_eq!(upgrade_cost, 1);

        let upgrade_cost = calculate_upgrade(&problem, &solution, 0);
        assert_eq!(upgrade_cost, 1);
    }

    #[test]
    fn test_class_mixed_cost() {
        let size = 3;
        let mut cottages = Cottages::empty(size);
        let mut reservations = Reservations::empty(size);
        cottages.class = vec![4, 4, 4];
        reservations.class = vec![1, 3, 4];

        let problem = Problem::new(cottages, reservations, 0);
        let solution = Solution::naive(&problem);

        let upgrade_cost = calculate_upgrades(&problem, &solution);
        assert_eq!(upgrade_cost, 2);

        let upgrade_cost = calculate_upgrade(&problem, &solution, 0);
        assert_eq!(upgrade_cost, 1);

        let upgrade_cost = calculate_upgrade(&problem, &solution, 1);
        assert_eq!(upgrade_cost, 1);

        let upgrade_cost = calculate_upgrade(&problem, &solution, 2);
        assert_eq!(upgrade_cost, 0);
    }

    #[test]
    fn test_capacity_no_cost() {
        let size = 3;

        let mut cottages = Cottages::empty(size);
        let mut reservations = Reservations::empty(size);
        cottages.capacity = vec![2, 4, 6];
        reservations.people = vec![2, 4, 6];

        let problem = Problem::new(cottages, reservations, 0);
        let solution = Solution::naive(&problem);

        let capacity_cost = calculate_upgrades(&problem, &solution);
        assert_eq!(capacity_cost, 0);
    }

    #[test]
    fn test_capacity_cost() {
        let size = 3;

        let mut cottages = Cottages::empty(size);
        let mut reservations = Reservations::empty(size);
        cottages.capacity = vec![4, 40, 400];
        reservations.people = vec![2, 4, 6];

        let problem = Problem::new(cottages, reservations, 0);
        let solution = Solution::naive(&problem);

        let capacity_cost = calculate_upgrades(&problem, &solution);
        assert_eq!(capacity_cost, 3);
    }

    #[test]
    fn test_capacity_mixed_cost() {
        let size = 4;

        let mut cottages = Cottages::empty(size);
        let mut reservations = Reservations::empty(size);
        cottages.capacity = vec![2, 20, 6, 60];
        reservations.people = vec![2, 4, 6, 8];

        let problem = Problem::new(cottages, reservations, 0);
        let solution = Solution::naive(&problem);

        let capacity_cost = calculate_upgrades(&problem, &solution);
        assert_eq!(capacity_cost, 2);
    }
}
