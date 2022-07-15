use std::ops::Range;

use itertools::Itertools;

use crate::problem::Problem;

#[derive(Debug)]
pub struct Solution {
    // Mapping from reservation to cottage
    mapping: Vec<Option<usize>>,
    timetable: Vec<Option<usize>>,
    // gap_timetable[x] = y means: the gap at x is of length y
    gap_timetable: Vec<usize>,
    cottages: usize,
    timeslots: usize,
    unallocated: Vec<usize>,
}

impl Solution {
    pub fn empty(problem: &Problem) -> Self {
        let reservations = problem.reservations.reservations();
        let timeslots = problem.reservations.timeslots();
        let cottages = problem.cottages.cottages();

        Self {
            mapping: vec![None; reservations],
            timetable: vec![None; cottages * timeslots],
            gap_timetable: vec![timeslots; cottages * timeslots],
            cottages,
            timeslots,
            unallocated: (0..reservations).collect(),
        }
    }

    pub fn naive(problem: &Problem) -> Self {
        let reservations = problem.reservations.reservations();
        let timeslots = problem.reservations.timeslots();
        let cottages = problem.cottages.cottages();

        let mut out = Self {
            mapping: vec![None; reservations],
            timetable: vec![None; cottages * timeslots],
            gap_timetable: vec![timeslots; cottages * timeslots],
            cottages,
            timeslots,
            unallocated: (0..reservations).collect(),
        };

        for reservation in 0..reservations {
            out.assign(
                reservation,
                reservation,
                problem.reservations.range(reservation),
            );
        }

        out
    }

    pub fn unallocated(&self) -> &[usize] {
        &self.unallocated[..]
    }

    pub fn index<'a>(&'a self, v: &'a Vec<usize>) -> impl Iterator<Item = Option<usize>> + 'a {
        #[cfg(feature = "supersafe")]
        assert!(v.len() >= self.reservations);

        unsafe {
            self.mapping
                .iter()
                .map(|&cottage_id| cottage_id.map(|id| *v.get_unchecked(id)))
        }
    }

    pub fn mapping(&self) -> &[Option<usize>] {
        &self.mapping[..]
    }

    pub fn mapping_single(&self, reservation: usize) -> Option<usize> {
        #[cfg(feature = "supersafe")]
        assert!(self.mapping.get(reservation).is_some());

        unsafe { *self.mapping.get_unchecked(reservation) }
    }

    pub fn get_timetable_mut(&mut self, cottage: usize) -> &mut [Option<usize>] {
        let index = cottage * self.timeslots..(cottage + 1) * self.timeslots;

        #[cfg(feature = "supersafe")]
        assert!(self.timetable.get(index.clone()).is_some());

        unsafe { self.timetable.get_unchecked_mut(index) }
    }

    pub fn get_cottage(&self, reservation: usize) -> &Option<usize> {
        #[cfg(feature = "supersafe")]
        assert!(&self.mapping.get(reservation).is_some());
        unsafe { self.mapping.get_unchecked(reservation) }
    }

    pub fn get_timetable(&self, cottage: usize) -> &[Option<usize>] {
        let index = cottage * self.timeslots..(cottage + 1) * self.timeslots;

        #[cfg(feature = "supersafe")]
        assert!(self.timetable.get(index.clone()).is_some());

        unsafe { self.timetable.get_unchecked(index) }
    }

    pub fn timetable_iter(&self) -> impl Iterator<Item = &[Option<usize>]> {
        (0..self.cottages).map(|i| self.get_timetable(i))
    }

    pub fn is_free(&self, cottage: usize, range: Range<usize>) -> bool {
        self.get_timetable(cottage)[range]
            .iter()
            .all(|&x| x.is_none())
    }

    pub fn is_taken_by(
        &self,
        cottage: usize,
        range: Range<usize>,
    ) -> impl Iterator<Item = usize> + '_ {
        #[cfg(feature = "supersafe")]
        assert!(self.get_timetable(cottage).get(range.clone()).is_some());

        unsafe {
            self.get_timetable(cottage)
                .get_unchecked(range)
                .iter()
                .filter_map(|&x| x)
                .dedup()
        }
    }

    pub fn assign(&mut self, cottage: usize, reservation: usize, range: Range<usize>) {
        self.unallocated.retain(|x| *x != reservation);
        self.set_mapping(reservation, Some(cottage));
        self.set_timetable(cottage, Some(reservation), range.clone());
        self.gap_timetable_assign(cottage, range);
    }

    pub fn unassign(&mut self, cottage: usize, reservation: usize, range: Range<usize>) {
        // PANICS!
        self.unallocated.push(reservation);
        self.set_mapping(reservation, None);
        self.set_timetable(cottage, None, range.clone());
        self.gap_timetable_unassign(cottage, range);
    }

    fn set_mapping(&mut self, reservation: usize, target: Option<usize>) {
        #[cfg(feature = "supersafe")]
        assert!(self.mapping.get(reservation).is_some());

        unsafe {
            *self.mapping.get_unchecked_mut(reservation) = target;
        }
    }

    #[inline(always)]
    pub fn get_gap_timetable(&self, cottage: usize) -> &[usize] {
        let index = cottage * self.timeslots..(cottage + 1) * self.timeslots;

        #[cfg(feature = "supersafe")]
        assert!(self.gap_timetable.get(index.clone()).is_some());

        unsafe { self.gap_timetable.get_unchecked(index) }
    }

    fn get_gap_timetable_mut(&mut self, cottage: usize) -> &mut [usize] {
        let index = cottage * self.timeslots..(cottage + 1) * self.timeslots;

        #[cfg(feature = "supersafe")]
        assert!(self.gap_timetable.get(index.clone()).is_some());

        unsafe { self.gap_timetable.get_unchecked_mut(index) }
    }

    // Assign a reservation in the gap timetable
    fn gap_timetable_assign(&mut self, cottage: usize, range: Range<usize>) {
        let gap_timetable = self.get_gap_timetable_mut(cottage);
        let gap = gap_timetable.get(range.start).unwrap();

        #[cfg(feature = "supersafe")]
        assert!(gap_timetable.get(range.end..).is_some());

        let end = unsafe { gap_timetable.get_unchecked(range.end..) };

        // TODO: Pre-calculate this?
        let last_index = range.end + end.iter().take_while(|x| *x == gap).count();
        let first_index = last_index - gap;

        #[cfg(feature = "supersafe")]
        {
            assert!(gap_timetable.get(first_index..range.start).is_some());
            assert!(gap_timetable.get(range.end..last_index).is_some());
            assert!(gap_timetable.get(range.clone()).is_some());
        }

        unsafe {
            gap_timetable
                .get_unchecked_mut(first_index..range.start)
                .fill(range.start - first_index);
            gap_timetable
                .get_unchecked_mut(range.end..last_index)
                .fill(last_index - range.end);
            gap_timetable.get_unchecked_mut(range).fill(0)
        }
    }

    fn gap_timetable_unassign(&mut self, cottage: usize, range: Range<usize>) {
        let gap_timetable = self.get_gap_timetable_mut(cottage);

        let mut start = range.start;
        if let Some(previous_gap) = gap_timetable.get(range.start.wrapping_sub(1)) {
            start -= previous_gap;
        }

        let mut end = range.end;
        if let Some(next_gap) = gap_timetable.get(range.end) {
            end += next_gap;
        }

        #[cfg(feature = "supersafe")]
        assert!(gap_timetable.get(range.clone()).is_some());

        unsafe {
            gap_timetable
                .get_unchecked_mut(start..end)
                .fill(end - start);
        }
    }

    fn set_timetable(&mut self, cottage: usize, target: Option<usize>, range: Range<usize>) {
        let v = self.get_timetable_mut(cottage);

        #[cfg(feature = "supersafe")]
        assert!(v.get(range.clone()).is_some());

        unsafe {
            v.get_unchecked_mut(range).fill(target);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn timetable_independent() {
        let size = 5;
        let problem = Problem::empty(size, size);
        let mut solution = Solution::empty(&problem);
        solution.timetable[0] = Some(0);
        assert_eq!(solution.get_timetable(1)[0], None);
    }

    #[test]
    pub fn assign_then_unassign_0() {
        let size = 5;
        let mut problem = Problem::empty(size, size);
        problem.reservations.stay = vec![10; size];
        let mut solution = Solution::empty(&problem);

        // __________
        // #####_____
        solution.assign(0, 0, 0..5);
        assert_eq!(
            solution.get_timetable(0),
            [vec![Some(0); 5], vec![None; 5]].concat()
        );

        // #####_____
        // __###_____
        solution.unassign(0, 0, 0..3);
        assert_eq!(
            solution.get_timetable(0),
            [vec![None; 3], vec![Some(0); 2], vec![None; 5]].concat()
        );

        solution.unassign(0, 0, 3..5);
        assert_eq!(solution.get_timetable(0), vec![None; 10]);
    }

    #[test]
    pub fn assign_then_unassign_1() {
        let size = 5;
        let mut problem = Problem::empty(size, size);
        problem.reservations.stay = vec![1; size];
        let mut solution = Solution::empty(&problem);

        // _
        assert_eq!(solution.get_timetable(0), vec![None; 1]);

        // _
        // #
        solution.assign(0, 0, 0..1);
        assert_eq!(solution.get_timetable(0), vec![Some(0); 1]);

        // #
        // _
        solution.unassign(0, 0, 0..1);
        assert_eq!(solution.get_timetable(0), vec![None; 1]);
    }

    #[test]
    pub fn is_free() {
        let size = 5;
        let mut problem = Problem::empty(size, size);
        problem.reservations.stay = vec![10; 5];
        let mut solution = Solution::empty(&problem);
        assert!(solution.is_free(0, 0..5));
        assert!(solution.is_free(1, 0..5));

        solution.assign(0, 0, 0..5);
        assert!(!solution.is_free(0, 0..5));
        assert!(!solution.is_free(0, 4..5));
        assert!(!solution.is_free(0, 1..5));
        assert!(solution.is_free(0, 5..6));
        assert!(solution.is_free(1, 0..5));
    }

    #[test]
    pub fn is_taken_by() {
        let size = 5;
        let mut problem = Problem::empty(size, size);
        problem.reservations.stay = vec![10; 5];
        let mut solution = Solution::empty(&problem);

        assert!(solution.is_taken_by(0, 0..10).collect_vec().is_empty());

        solution.assign(0, 0, 0..1);
        solution.assign(0, 1, 1..2);
        solution.assign(0, 2, 7..8);

        assert_eq!(
            solution.is_taken_by(0, 0..10).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
    }
}
