use core::ops::ControlFlow;
use std::collections::BTreeSet;
use std::collections::HashMap;

use crate::matrix;
use crate::matrix::Matrix;

pub struct Solver {
    matrix: Matrix,
}

pub trait Row {
    fn iter(&self) -> impl Iterator<Item = u16>;
}

impl Solver {
    pub fn new<R: Row>(rows: &[R]) -> Self {
        let dense_to_sparse = rows
            .iter()
            .flat_map(Row::iter)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let sparse_to_dense = dense_to_sparse
            .iter()
            .copied()
            .enumerate()
            .map(|(dense, sparse)| (sparse, dense as u16 + 1))
            .collect::<HashMap<_, _>>();

        let mut matrix = Matrix::new(dense_to_sparse.len() as u16);
        let mut prev = matrix.map();

        for (row, r) in rows
            .iter()
            .enumerate()
            .map(|(i, row)| (matrix::Row::new(i as u32), row))
        {
            let mut head = None;
            let mut tail = None;

            for sparse in r.iter() {
                let dense = sparse_to_dense[&sparse];
                let col = matrix.column(dense);

                matrix.update_size(col, 1);

                let index = matrix.push(matrix::Node::dangling(row, col));
                let up = prev[col];

                matrix.attach_vertical(up, index);

                if head.is_some() {
                    let left = index.prev();
                    matrix.attach_horizontal(left, index);
                }

                prev[col] = index;

                head.get_or_insert(index);
                tail = Some(index);
            }

            if let (Some(head), Some(tail)) = (head, tail) {
                matrix.attach_horizontal(tail, head);
            }
        }

        // Complete column cycles
        for (col, index) in prev.iter() {
            matrix.attach_vertical(*index, col.into());
        }

        Self { matrix }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.matrix.len()
    }

    pub fn solve_count(&self) -> usize {
        let mut solution = Vec::new();
        let mut count = 0;
        self.solve_inner(&mut solution, &mut |_| {
            count += 1;
            ControlFlow::<(), ()>::Continue(())
        });
        count
    }

    pub fn solve<T, F: FnMut(&mut [usize]) -> ControlFlow<T, ()>>(
        &self,
        mut inspect: F,
    ) -> Option<T> {
        let mut solution = Vec::new();
        let mut buffer = Vec::new();
        self.solve_inner(&mut solution, &mut |solution| {
            buffer.clear();
            buffer.extend(
                solution
                    .iter()
                    .map(|index| usize::from(self.matrix[*index].row)),
            );
            inspect(&mut buffer)
        })
    }

    fn solve_inner<T, F: FnMut(&[matrix::Index]) -> ControlFlow<T, ()>>(
        &self,
        solution: &mut Vec<matrix::Index>,
        inspect: &mut F,
    ) -> Option<T> {
        let Some(col) = self
            .matrix
            .walk_right(matrix::Index::GLOBAL)
            .map(|index| self.matrix.index_to_column(index))
            .min_by_key(|col| self.matrix.size(*col))
        else {
            match inspect(solution) {
                ControlFlow::Continue(()) => return None,
                ControlFlow::Break(out) => return Some(out),
            }
        };

        self.cover(col);

        for i in self.matrix.walk_down(col.into()) {
            solution.push(i);

            for j in self
                .matrix
                .walk_right(i)
                .map(|j| self.matrix.index_to_column(j))
            {
                self.cover(j);
            }

            if let Some(out) = self.solve_inner(solution, inspect) {
                return Some(out);
            }

            for j in self
                .matrix
                .walk_left(i)
                .map(|j| self.matrix.index_to_column(j))
            {
                self.uncover(j);
            }

            solution.pop();
        }

        self.uncover(col);
        None
    }

    fn cover(&self, col: matrix::Col) {
        let col = col.into();

        self.matrix.detach_horizontal(col);

        for i in self.matrix.walk_down(col) {
            for j in self.matrix.walk_right(i) {
                self.matrix.detach_vertical(j);

                let col = self.matrix.index_to_column(j);
                self.matrix.update_size(col, -1);
            }
        }
    }

    fn uncover(&self, col: matrix::Col) {
        let col = col.into();

        for i in self.matrix.walk_up(col) {
            for j in self.matrix.walk_left(i) {
                self.matrix.reattach_vertical(j);

                let col = self.matrix.index_to_column(j);
                self.matrix.update_size(col, 1);
            }
        }

        self.matrix.reattach_horizontal(col);
    }
}

#[test]
fn smoke() {
    struct Row(u8);

    impl crate::solve::Row for Row {
        fn iter(&self) -> impl Iterator<Item = u16> {
            (0..8).filter(|bit| (self.0 >> bit) & 1 > 0)
        }
    }

    let solver = Solver::new(&[
        Row(0b0110100),
        Row(0b1001001),
        Row(0b0100110),
        Row(0b0001001),
        Row(0b1000010),
        Row(0b1011000),
    ]);

    let mut seen = false;
    solver.solve(|rows| {
        rows.sort();
        assert!(!seen);
        assert_eq!(rows, &[0, 3, 4]);
        seen = true;
        core::ops::ControlFlow::<(), _>::Continue(())
    });
}
