use core::cell::Cell;
use core::iter;
use core::ops;

#[derive(Debug)]
pub struct Matrix {
    headers: Vec<Header>,
    nodes: Vec<Node>,
}

macro_rules! impl_walk {
    ($name:ident, $field:ident) => {
        fn $name(&self, start: Index) -> impl Iterator<Item = Index> + '_ {
            self.walk(|node| node.$field.get(), start)
        }
    };
}

macro_rules! impl_attach {
    ($attach:ident, $reattach:ident, $detach:ident, $a:ident, $b:ident) => {
        fn $attach(&self, a: Index, b: Index) {
            self[a].$b.set(b);
            self[b].$a.set(a);
        }

        fn $reattach(&self, i: Index) {
            let a = self[i].$a.get();
            let b = self[i].$b.get();
            self[a].$b.set(i);
            self[b].$a.set(i);
        }

        fn $detach(&self, i: Index) {
            let a = self[i].$a.get();
            let b = self[i].$b.get();
            self[a].$b.set(b);
            self[b].$a.set(a);
        }
    };
}

impl Matrix {
    fn new(column_count: u16) -> Self {
        let header_count = 1 + column_count;
        let mut headers = Vec::with_capacity(header_count as usize);

        headers.push(Header {
            size: Cell::new(0),
            node: Node::new(
                Row(0),
                Col(0),
                Index::dangling(),
                Index::dangling(),
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
                    Index::dangling(),
                    Index::dangling(),
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

    fn size(&self, col: Col) -> u32 {
        self.headers[col.0 as usize].size.get()
    }

    fn update_size(&self, col: Col, delta: i32) {
        let size = &self.headers[col.0 as usize].size;
        size.set(size.get().wrapping_add_signed(delta))
    }

    fn push(&mut self, node: Node) -> Index {
        let index = self.nodes.len();
        self.nodes.push(node);
        Index((self.headers.len() + index) as u32)
    }

    fn map(&self) -> ColMap<Index> {
        ColMap(
            (0..self.headers.len())
                .map(|i| Col(i as u16).into())
                .collect(),
        )
    }

    fn index_to_column(&self, index: Index) -> Col {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Index(u32);

impl Index {
    const GLOBAL: Self = Self(0);

    fn header(col: u16) -> Self {
        Self(1 + col as u32)
    }

    fn dangling() -> Self {
        Self(u32::MAX)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Row(u32);

struct ColMap<T>(Vec<T>);

impl<T> ColMap<T> {
    fn iter(&self) -> impl Iterator<Item = (Col, &T)> + '_ {
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
struct Col(u16);

impl From<Col> for Index {
    fn from(col: Col) -> Self {
        Self(col.0 as u32)
    }
}

#[derive(Debug)]
struct Header {
    size: Cell<u32>,
    node: Node,
}

#[derive(Clone, Debug)]
struct Node {
    row: Row,
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

    fn dangling(row: Row, col: Col) -> Self {
        Self {
            row,
            col,
            u: Cell::new(Index(u32::MAX)),
            d: Cell::new(Index(u32::MAX)),
            l: Cell::new(Index(u32::MAX)),
            r: Cell::new(Index(u32::MAX)),
        }
    }
}
