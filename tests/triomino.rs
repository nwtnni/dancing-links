use dancing_links::solve::Row;
use dancing_links::tile;
use dancing_links::tile::Point;

#[derive(Clone, Debug)]
struct Triomino([Point; 3]);

macro_rules! triomino {
    ($($rest:tt)*) => {
        Triomino(tile!(3 $($rest)*))
    };
}

const TRIOMINOES: [Triomino; 6] = [
    triomino! {
        X X .
        X . .
        . . .
    },
    triomino! {
        X X .
        . X .
        . . .
    },
    triomino! {
        X . .
        X X .
        . . .
    },
    triomino! {
        . X .
        X X .
        . . .
    },
    triomino! {
        X . .
        X . .
        X . .
    },
    triomino! {
        X X X
        . . .
        . . .
    },
];

impl Row for Triomino {
    fn iter(&self) -> impl Iterator<Item = u16> {
        self.0
            .iter()
            // Imposes maximum width of 64 units
            .map(|point| point.i as u16 * 64 + point.j as u16)
    }
}

/// Return number of ways to tile `rows` by `cols` rectangular
/// grid using triominoes.
fn solutions(rows: usize, cols: usize) -> usize {
    use dancing_links::solve::Solver;

    let mut triominoes = Vec::new();

    for triomino in TRIOMINOES.iter() {
        for row in 0..rows {
            'outer: for col in 0..cols {
                let mut translated = triomino.clone();

                for (before, after) in triomino.0.iter().zip(&mut translated.0) {
                    let point = Point {
                        i: before.i + row,
                        j: before.j + col,
                    };

                    if point.i >= rows || point.j >= cols {
                        continue 'outer;
                    }

                    *after = point;
                }

                triominoes.push(translated);
            }
        }
    }

    Solver::new(&triominoes).solve_count()
}

#[test]
fn rectangle_2x9() {
    assert_eq!(solutions(2, 9), 41);
}
