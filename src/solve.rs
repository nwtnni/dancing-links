use core::ops::ControlFlow;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::matrix;
use crate::matrix::Matrix;

pub struct Solver {
    dense_to_sparse: Vec<u16>,
    pool: Matrix,
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
            .map(|(dense, sparse)| (sparse, dense as u16))
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
                let left = index.prev();
                let up = prev[col];

                matrix.attach_vertical(up, index);
                matrix.attach_horizontal(left, index);
                prev[col] = index;

                head.get_or_insert(index);
                tail = Some(index);
            }

            let (Some(head), Some(tail)) = (head, tail) else {
                continue;
            };

            matrix.attach_horizontal(tail, head);
        }

        // Complete column cycles
        for (col, index) in prev.iter() {
            matrix.attach_vertical(*index, col.into());
        }

        Self {
            dense_to_sparse,
            pool: matrix,
        }
    }

    pub(crate) fn solve<T, F: FnMut(&[matrix::Index]) -> ControlFlow<T, ()>>(
        &self,
        inspect: &mut F,
    ) -> Option<T> {
        let mut solution = Vec::new();
        self.solve_inner(&mut solution, inspect)
    }

    fn solve_inner<T, F: FnMut(&[matrix::Index]) -> ControlFlow<T, ()>>(
        &self,
        solution: &mut Vec<matrix::Index>,
        inspect: &mut F,
    ) -> Option<T> {
        let Some(col) = self
            .pool
            .walk_right(matrix::Index::GLOBAL)
            .map(|index| self.pool.index_to_column(index))
            .min_by_key(|col| self.pool.size(*col))
        else {
            match inspect(solution) {
                ControlFlow::Continue(()) => return None,
                ControlFlow::Break(out) => return Some(out),
            }
        };

        self.cover(col);

        for i in self.pool.walk_down(col.into()) {
            solution.push(i);

            for j in self
                .pool
                .walk_right(i)
                .map(|j| self.pool.index_to_column(j))
            {
                self.cover(j);
            }

            if let Some(out) = self.solve_inner(solution, inspect) {
                return Some(out);
            }

            for j in self.pool.walk_left(i).map(|j| self.pool.index_to_column(j)) {
                self.uncover(j);
            }

            solution.pop();
        }

        self.uncover(col);
        None
    }

    fn cover(&self, col: matrix::Col) {
        let col = col.into();

        self.pool.detach_horizontal(col);

        for i in self.pool.walk_down(col) {
            for j in self.pool.walk_right(i) {
                self.pool.detach_vertical(j);

                let col = self.pool.index_to_column(j);
                self.pool.update_size(col, -1);
            }
        }
    }

    fn uncover(&self, col: matrix::Col) {
        let col = col.into();

        for i in self.pool.walk_up(col) {
            for j in self.pool.walk_left(i) {
                self.pool.reattach_vertical(j);

                let col = self.pool.index_to_column(j);
                self.pool.update_size(col, 1);
            }
        }

        self.pool.reattach_horizontal(col);
    }
}
