//! # References
//!
//! - https://www.cs.brandeis.edu/~storer/JimPuzzles/ZPAGES/zzzPentominoes.html
//! - https://www.fishlet.com/2022/01/21/pentomino/

use core::ops::ControlFlow;
use std::collections::BTreeSet;
use std::collections::HashMap;

use dancing_links::solve::Row;
use dancing_links::solve::Solver;
use dancing_links::tile;
use dancing_links::tile::Point;
use dancing_links::Tile;

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
struct Pentomino {
    id: char,
    tile: Tile<5>,
}

macro_rules! pentomino {
    ($id:tt: $($rest:tt)*) => {
        Pentomino {
            id: $id,
            tile: tile!(5 $($rest)*),
        }
    };
}

// Note: using Conway's labeling scheme
// to simplify conversion from label to
// numeric ID.
const PENTOMINOES: [Pentomino; 12] = [
    pentomino! { 'O':
        X X X X X
    },
    pentomino! { 'P':
        X X . . .
        X X . . .
        X . . . .
    },
    pentomino! { 'Q':
        X X X X .
        . . . X .
        . . . . .
        . . . . .
    },
    pentomino! { 'R':
        . X X . .
        X X . . .
        . X . . .
    },
    pentomino! { 'S':
        X . . . .
        X X . . .
        . X . . .
        . X . . .
    },
    pentomino! { 'T':
        X X X . .
        . X . . .
        . X . . .
    },
    pentomino! { 'U':
        X . X . .
        X X X . .
    },
    pentomino! { 'V':
        . . X . .
        . . X . .
        X X X . .
    },
    pentomino! { 'W':
        . . X . .
        . X X . .
        X X . . .
    },
    pentomino! { 'X':
        . X . . .
        X X X . .
        . X . . .
    },
    pentomino! { 'Y':
        . X . . .
        X X . . .
        . X . . .
        . X . . .
    },
    pentomino! { 'Z':
        X X . . .
        . X . . .
        . X X . .
    },
];

#[test]
fn transform() {
    for pentomino in PENTOMINOES {
        // https://en.wikipedia.org/wiki/Pentomino#Symmetry
        let expected = match pentomino.id {
            'X' => 1,
            'O' => 2,
            'T' | 'U' | 'V' | 'W' | 'Z' => 4,
            'L' | 'Q' | 'R' | 'S' | 'P' | 'Y' => 8,
            id => unreachable!("Unexpected pentomino ID: {}", id),
        };

        let actual = pentomino
            .tile
            .transformations()
            .collect::<BTreeSet<_>>()
            .len();

        assert_eq!(
            expected, actual,
            "Transformation mismatch for {}",
            pentomino.id,
        );
    }
}

impl Pentomino {
    fn encode_id(&self) -> u16 {
        (self.id as u8 - b'O') as u16
    }
}

impl Row for Pentomino {
    fn iter(&self) -> impl Iterator<Item = u16> {
        self.tile
            .as_ref()
            .iter()
            // Imposes maximum width of 32 units
            .map(|point| point.i as u16 * 32 + point.j as u16)
            // Encode tile ID in upper 4 bits
            // Note: offset by 1 to avoid collision with (0, 0) point encoding
            .chain(core::iter::once((1 + self.encode_id()) << 12))
    }
}

#[test]
fn rectangle_6x10() {
    assert_eq!(rectangle(6, 10).len(), 2_339);
}

#[test]
fn rectangle_5x12() {
    assert_eq!(rectangle(5, 12).len(), 1_010);
}

#[test]
fn rectangle_4x15() {
    assert_eq!(rectangle(4, 15).len(), 368);
}

#[test]
fn rectangle_3x20() {
    assert_eq!(rectangle(3, 20).len(), 2);
}

#[test]
fn scott() {
    assert_eq!(
        solve(8, 8, |point| !((3..5).contains(&point.i)
            && (3..5).contains(&point.j)))
        .len(),
        65
    );
}

fn rectangle(rows: u8, cols: u8) -> BTreeSet<tile::Set<5>> {
    solve(rows, cols, |_| true)
}

fn solve<F: FnMut(Point) -> bool>(rows: u8, cols: u8, filter: F) -> BTreeSet<tile::Set<5>> {
    let pentominoes = pack(rows, cols, filter);

    let mut count = 0;
    let mut seen = BTreeSet::<tile::Set<5>>::new();

    let solver = Solver::new(&pentominoes);

    solver.solve(|solution| {
        let tiles = solution
            .iter()
            .map(|index| pentominoes[*index].tile)
            .collect::<tile::Set<5>>()
            .canonicalize();

        if !seen.insert(tiles) {
            return ControlFlow::Continue(());
        }

        count += 1;
        core::ops::ControlFlow::<(), _>::Continue(())
    });

    seen
}

fn pack<F: FnMut(Point) -> bool>(rows: u8, cols: u8, mut filter: F) -> Vec<Pentomino> {
    let mut pentominoes = Vec::new();

    for pentomino in transformations().into_iter() {
        for row in 0..rows {
            'outer: for col in 0..cols {
                let mut translated = pentomino;

                for (before, after) in pentomino.tile.as_ref().iter().zip(translated.tile.as_mut())
                {
                    let point = Point {
                        i: before.i + row,
                        j: before.j + col,
                    };

                    if point.i >= rows || point.j >= cols || !filter(point) {
                        continue 'outer;
                    }

                    *after = point;
                }

                pentominoes.push(translated);
            }
        }
    }

    pentominoes
}

fn transformations() -> BTreeSet<Pentomino> {
    PENTOMINOES
        .iter()
        .flat_map(|&Pentomino { tile, id }| {
            tile.transformations()
                .map(move |tile| Pentomino { id, tile })
        })
        .collect()
}

#[expect(unused)]
fn debug(rows: u8, cols: u8, set: &tile::Set<5>) {
    let mut grid = HashMap::new();

    for (i, tile) in set.iter().enumerate() {
        for point in tile.as_ref() {
            grid.insert(point, i);
        }
    }

    for i in 0..rows {
        for j in 0..cols {
            eprint!("\x1b[48;5;{}m ", grid[&Point { i, j }]);
        }
        eprintln!("\x1b[49m");
    }
}
