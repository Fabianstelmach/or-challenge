use crate::{problem::Problem, solution::Solution};

#[derive(Default)]
struct GapFoldState {
    count: usize,
    gaps: usize,
    legionella_gaps: usize,
    fr_th_gaps: usize,
}

impl GapFoldState {
    pub fn new(count: usize, gaps: usize, legionella_gaps: usize, fr_th_gaps: usize) -> Self {
        Self {
            count,
            gaps,
            legionella_gaps,
            fr_th_gaps,
        }
    }
}

fn fold_gaps(state: GapFoldState, cell: (usize, usize)) -> GapFoldState {
    let thursday = cell.0 == 6;
    let cell_state = cell.1;

    // Count current gap size
    let count = cell_state * state.count + cell_state;

    // Total amount of gaps
    let gaps = state.gaps + (count == 1) as usize;

    // Total amount of legionella gaps
    let legionella_gaps = state.legionella_gaps + (count == 21) as usize;

    // Total amount of fr_th gaps
    let fr_th_gaps = state.fr_th_gaps + thursday as usize * (count > 6) as usize;

    GapFoldState::new(count, gaps, legionella_gaps, fr_th_gaps)
}

#[inline(never)]
fn calculate_gaps_single(timetable: &[Option<usize>], phase: usize) -> GapFoldState {
    return timetable
        .iter()
        .map(|x| x.is_none() as usize)
        .enumerate()
        .map(|(date, cell)| ((date + phase).rem_euclid(7), cell))
        .fold(GapFoldState::default(), fold_gaps);
}

pub fn calculate_cottage_gaps(
    problem: &Problem,
    solution: &Solution,
    cottage: usize,
) -> (usize, usize, usize) {
    let timetable = solution.get_timetable(cottage);

    let gaps = calculate_gaps_single(timetable, problem.phase);

    (gaps.gaps, gaps.fr_th_gaps, gaps.legionella_gaps)
}

pub fn calculate_cottage_gaps2(
    problem: &Problem,
    solution: &Solution,
    cottage: usize,
) -> (usize, usize, usize) {
    let timetable = solution.get_gap_timetable(cottage);
    let phase = problem.phase;

    let mut gaps = 0;
    let mut fr_th_gaps = 0;
    let mut legionella_gaps = 0;

    let mut i = 0;
    while i < timetable.len() {
        let gap = timetable[i];
        if gap == 0 {
            i += 1;
            continue;
        }

        gaps += 1;

        // 0: fri, 1: sat, 2: sun... 6: thu
        let phase_day = (i + phase).rem_euclid(7);
        let inverse_phase_day = (7 - phase_day).rem_euclid(7);
        fr_th_gaps += (gap.saturating_sub(inverse_phase_day) / 7) as usize;

        legionella_gaps += (gap >= 21) as usize;

        i += gap;
    }

    (gaps, fr_th_gaps, legionella_gaps)
}

pub fn calculate_gaps(problem: &Problem, solution: &Solution) -> (usize, usize, usize) {
    (0..problem.cottages.cottages())
        .map(|cottage| calculate_cottage_gaps2(problem, solution, cottage))
        .fold((0, 0, 0), |s, state| {
            (s.0 + state.0, s.1 + state.1, s.2 + state.2)
        })
}

pub fn calculate_gaps2(problem: &Problem, solution: &Solution) -> (usize, usize, usize) {
    solution
        .timetable_iter()
        .map(|timetable| calculate_gaps_single(timetable, problem.phase))
        .fold((0, 0, 0), |s, state| {
            (
                s.0 + state.gaps,
                s.1 + state.fr_th_gaps,
                s.2 + state.legionella_gaps,
            )
        })
}

#[cfg(test)]
mod test {
    use crate::problem::Problem;

    use super::*;

    #[test]
    fn calculate_gap_assign() {
        let size = 1;
        let mut problem = Problem::empty(size, size);
        problem.reservations.stay = vec![5; size];
        let mut solution = Solution::empty(&problem);
        assert_eq!(calculate_gaps(&problem, &solution).0, 1);

        // Create a gap
        // _____
        // _#___
        solution.assign(0, 0, 1..2);
        assert_eq!(calculate_gaps(&problem, &solution).0, 2);

        // Shrink a gap
        // _#___
        // _##__
        solution.assign(0, 0, 2..3);
        assert_eq!(calculate_gaps(&problem, &solution).0, 2);

        // Close a gap
        // _##__
        // ###__
        solution.assign(0, 0, 0..1);
        assert_eq!(calculate_gaps(&problem, &solution).0, 1);

        // Close all gaps
        // ###__
        // #####
        solution.assign(0, 0, 3..5);
        assert_eq!(calculate_gaps(&problem, &solution).0, 0);
    }

    #[test]
    fn calculate_gap_2_assign() {
        let size = 2;
        let mut problem = Problem::empty(size, size);
        problem.reservations.stay = vec![5; size];
        let mut solution = Solution::empty(&problem);
        assert_eq!(calculate_gaps(&problem, &solution).0, 2);

        // Create a gap in c0
        // _____
        // _#___
        solution.assign(0, 0, 1..2);
        assert_eq!(calculate_gaps(&problem, &solution).0, 3);

        // Create a gap in c1
        // _____
        // _#___
        solution.assign(1, 0, 1..2);
        assert_eq!(calculate_gaps(&problem, &solution).0, 4);

        // Shrink a gap in c0
        // _#___
        // _##__
        solution.assign(0, 0, 2..3);
        assert_eq!(calculate_gaps(&problem, &solution).0, 4);

        // Shrink a gap in c1
        // _#___
        // _##__
        solution.assign(1, 0, 2..3);
        assert_eq!(calculate_gaps(&problem, &solution).0, 4);

        // Close a gap in c0
        // _##__
        // ###__
        solution.assign(0, 0, 0..1);
        assert_eq!(calculate_gaps(&problem, &solution).0, 3);

        // Close a gap in c1
        // _##__
        // ###__
        solution.assign(1, 0, 0..1);
        assert_eq!(calculate_gaps(&problem, &solution).0, 2);

        // Close all gaps in c0
        // ###__
        // #####
        solution.assign(0, 0, 3..5);
        assert_eq!(calculate_gaps(&problem, &solution).0, 1);

        // Close all gaps in c1
        // ###__
        // #####
        solution.assign(1, 0, 3..5);
        assert_eq!(calculate_gaps(&problem, &solution).0, 0);
    }

    #[test]
    fn calculate_gap_unassign() {
        let mut problem = Problem::empty(1, 5);
        problem.reservations.arrival = (0..5).collect();
        problem.reservations.update();
        let mut solution = Solution::empty(&problem);

        // _____
        // #####
        solution.assign(0, 0, 0..1);
        solution.assign(0, 1, 1..2);
        solution.assign(0, 2, 2..3);
        solution.assign(0, 3, 3..4);
        solution.assign(0, 4, 4..5);
        assert_eq!(calculate_gaps(&problem, &solution).0, 0);

        // Create a gap
        // #####
        // ##_##
        solution.unassign(0, 2, 2..3);
        assert_eq!(calculate_gaps(&problem, &solution).0, 1);

        // Widen a gap
        // ##_##
        // ##__#
        solution.unassign(0, 3, 3..4);
        assert_eq!(calculate_gaps(&problem, &solution).0, 1);

        // Open a gap
        // ##_##
        // _#__#
        solution.unassign(0, 0, 0..1);
        assert_eq!(calculate_gaps(&problem, &solution).0, 2);

        // Close a gap
        // _#__#
        // ____#
        solution.unassign(0, 1, 1..2);
        assert_eq!(calculate_gaps(&problem, &solution).0, 1);
    }

    #[test]
    fn calculate_legionella_gap() {
        let size = 1;
        let mut problem = Problem::empty(size, size);
        problem.reservations.arrival = vec![42];
        let mut solution = Solution::empty(&problem);

        assert_eq!(calculate_gaps(&problem, &solution).2, 1);

        solution.assign(0, 0, 21..22);
        assert_eq!(calculate_gaps(&problem, &solution).2, 2);

        solution.assign(0, 0, 20..21);
        assert_eq!(calculate_gaps(&problem, &solution).2, 1);

        solution.assign(0, 0, 22..23);
        assert_eq!(calculate_gaps(&problem, &solution).2, 0);
    }

    #[test]
    fn calculate_fr_th_gap() {
        let size = 1;
        let mut problem = Problem::empty(size, size);
        problem.phase = 5;

        problem.reservations.arrival = vec![18];
        let mut solution = Solution::empty(&problem);
        solution.assign(0, 0, 0..2);

        solution.assign(0, 0, 16..18);
        assert_eq!(calculate_gaps(&problem, &solution).1, 2);

        solution.assign(0, 0, 2..3);
        assert_eq!(calculate_gaps(&problem, &solution).1, 1);

        solution.assign(0, 0, 15..16);
        assert_eq!(calculate_gaps(&problem, &solution).1, 0);
    }
}
