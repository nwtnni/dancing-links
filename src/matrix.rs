use core::cell::Cell;
use core::fmt::Display;
use core::iter;
use core::ops;
use std::collections::HashSet;

pub(crate) struct Matrix {
    headers: Vec<Header>,
    nodes: Vec<Node>,
}

macro_rules! impl_walk {
    ($name:ident, $field:ident) => {
        pub(crate) fn $name(&self, start: Index) -> impl Iterator<Item = Index> + '_ {
            self.walk(|node| node.$field.get(), start)
        }
    };
}

macro_rules! impl_attach {
    ($attach:ident, $reattach:ident, $detach:ident, $a:ident, $b:ident) => {
        pub(crate) fn $attach(&self, a: Index, b: Index) {
            self[a].$b.set(b);
            self[b].$a.set(a);
        }

        pub(crate) fn $reattach(&self, i: Index) {
            let a = self[i].$a.get();
            let b = self[i].$b.get();
            self[a].$b.set(i);
            self[b].$a.set(i);
        }

        pub(crate) fn $detach(&self, i: Index) {
            let a = self[i].$a.get();
            let b = self[i].$b.get();
            self[a].$b.set(b);
            self[b].$a.set(a);
        }
    };
}

impl Matrix {
    pub(crate) fn new(column_count: u16) -> Self {
        let header_count = 1 + column_count;
        let mut headers = Vec::with_capacity(header_count as usize);

        headers.push(Header {
            size: Cell::new(0),
            node: Node::new(
                Row(0),
                Col(0),
                Index::DANGLING,
                Index::DANGLING,
                Index::header(column_count),
                Index::header(0),
            ),
        });

        for i in 0..column_count {
            headers.push(Header {
                size: Cell::new(0),
                node: Node::new(
                    Row(0),
                    Col(i + 1),
                    Index::DANGLING,
                    Index::DANGLING,
                    i.checked_sub(1).map(Index::header).unwrap_or(Index::GLOBAL),
                    match i + 1 {
                        j if j == column_count => Index::GLOBAL,
                        j => Index::header(j),
                    },
                ),
            })
        }

        Self {
            headers,
            nodes: Vec::new(),
        }
    }

    pub(crate) fn size(&self, col: Col) -> u32 {
        self.headers[col.0 as usize].size.get()
    }

    pub(crate) fn update_size(&self, col: Col, delta: i32) {
        let size = &self.headers[col.0 as usize].size;
        size.set(size.get().wrapping_add_signed(delta))
    }

    pub(crate) fn column(&self, col: u16) -> Col {
        Col(col)
    }

    pub(crate) fn push(&mut self, node: Node) -> Index {
        let index = self.nodes.len();
        self.nodes.push(node);
        Index((self.headers.len() + index) as u32)
    }

    pub(crate) fn map(&self) -> ColMap<Index> {
        ColMap(
            (0..self.headers.len())
                .map(|i| Col(i as u16).into())
                .collect(),
        )
    }

    pub(crate) fn index_to_column(&self, index: Index) -> Col {
        self[index].col
    }

    impl_attach!(attach_vertical, reattach_vertical, detach_vertical, u, d);
    impl_attach!(
        attach_horizontal,
        reattach_horizontal,
        detach_horizontal,
        l,
        r
    );

    impl_walk!(walk_up, u);
    impl_walk!(walk_down, d);
    impl_walk!(walk_left, l);
    impl_walk!(walk_right, r);

    fn walk(&self, select: fn(&Node) -> Index, start: Index) -> impl Iterator<Item = Index> + '_ {
        let mut next = start;
        iter::from_fn(move || {
            next = select(&self[next]);
            match next == start {
                true => None,
                false => Some(next),
            }
        })
    }
}

impl ops::Index<Index> for Matrix {
    type Output = Node;
    fn index(&self, index: Index) -> &Self::Output {
        let index = index.0 as usize;
        match index.checked_sub(self.headers.len()) {
            None => &self.headers[index].node,
            Some(index) => &self.nodes[index],
        }
    }
}

impl core::fmt::Debug for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cols = self.headers.len() - 1;
        let rows = self
            .nodes
            .iter()
            .map(|node| node.row.0)
            .max()
            .unwrap_or(u32::MAX);

        let set = self
            .walk_right(Index::GLOBAL)
            .flat_map(|i| {
                self.walk_down(i)
                    .take_while(|index| *index != Index::DANGLING)
                    .map(move |j| &self[j])
                    .map(|node| (node.row.0, node.col.0))
            })
            .collect::<HashSet<_>>();

        for i in 0..rows {
            for j in 1..=cols {
                let char = match set.contains(&(i, j as u16)) {
                    true => "X",
                    false => ".",
                };

                write!(f, "{}", char)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Index(u32);

impl Index {
    pub(crate) const GLOBAL: Self = Self(0);
    pub(crate) const DANGLING: Self = Self(u32::MAX);

    fn header(col: u16) -> Self {
        Self(1 + col as u32)
    }

    pub(crate) fn prev(&self) -> Self {
        Self(self.0 - 1)
    }
}

impl Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Row(u32);

impl Row {
    pub(crate) fn new(row: u32) -> Self {
        Self(row)
    }
}

impl From<Row> for usize {
    fn from(value: Row) -> Self {
        value.0 as Self
    }
}

impl Display for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub(crate) struct ColMap<T>(Vec<T>);

impl<T> ColMap<T> {
    pub(crate) fn iter(&self) -> impl Iterator<Item = (Col, &T)> + '_ {
        self.0
            .iter()
            .enumerate()
            .map(|(col, item)| (Col(col as u16), item))
    }
}

impl<T> ops::Index<Col> for ColMap<T> {
    type Output = T;
    fn index(&self, index: Col) -> &Self::Output {
        &self.0[index.0 as usize]
    }
}

impl<T> ops::IndexMut<Col> for ColMap<T> {
    fn index_mut(&mut self, index: Col) -> &mut Self::Output {
        &mut self.0[index.0 as usize]
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Col(u16);

impl From<Col> for Index {
    fn from(col: Col) -> Self {
        Self(col.0 as u32)
    }
}

impl From<Col> for u16 {
    fn from(col: Col) -> Self {
        col.0
    }
}

impl Display for Col {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug)]
struct Header {
    size: Cell<u32>,
    node: Node,
}

#[derive(Clone, Debug)]
pub(crate) struct Node {
    pub(crate) row: Row,
    col: Col,

    u: Cell<Index>,
    d: Cell<Index>,
    l: Cell<Index>,
    r: Cell<Index>,
}

impl Node {
    fn new(row: Row, col: Col, u: Index, d: Index, l: Index, r: Index) -> Self {
        Self {
            row,
            col,
            u: Cell::new(u),
            d: Cell::new(d),
            l: Cell::new(l),
            r: Cell::new(r),
        }
    }

    pub(crate) fn dangling(row: Row, col: Col) -> Self {
        Self {
            row,
            col,
            u: Cell::new(Index::DANGLING),
            d: Cell::new(Index::DANGLING),
            l: Cell::new(Index::DANGLING),
            r: Cell::new(Index::DANGLING),
        }
    }
}
