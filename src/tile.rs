use core::cmp::Ordering;

// Invariant: `self.0` is sorted.
#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Set<const LEN: usize>(Vec<Tile<LEN>>);

impl<const LEN: usize> Set<LEN> {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, tile: Tile<LEN>) {
        self.0.push(tile);
        self.0.sort();
    }

    pub fn iter(&self) -> impl Iterator<Item = &Tile<LEN>> {
        self.0.iter()
    }

    pub fn reflect_x(&self) -> Self {
        self.transform_clamp(SPoint::reflect_x)
    }

    pub fn reflect_y(&self) -> Self {
        self.transform_clamp(SPoint::reflect_y)
    }

    pub fn rotate_90(&self) -> Self {
        self.transform_clamp(SPoint::rotate_90)
    }

    pub fn rotate_180(&self) -> Self {
        self.transform_clamp(SPoint::rotate_180)
    }

    pub fn rotate_270(&self) -> Self {
        self.transform_clamp(SPoint::rotate_270)
    }

    pub fn canonicalize(&self) -> Self {
        [self.clone(), self.reflect_x(), self.reflect_y()]
            .into_iter()
            .flat_map(|set| [set.rotate_90(), set.rotate_180(), set.rotate_270(), set])
            .min()
            .unwrap_or_default()
    }

    fn transform_clamp<F: FnMut(&SPoint) -> SPoint>(&self, mut apply: F) -> Self {
        Self::clamp(
            &self
                .0
                .iter()
                .map(|tile| tile.transform(&mut apply))
                .collect::<Vec<_>>(),
        )
    }

    fn clamp(tiles: &[[SPoint; LEN]]) -> Self {
        let min_i = tiles
            .iter()
            .flatten()
            .map(|point| point.i)
            .min()
            .unwrap_or(0);

        let min_j = tiles
            .iter()
            .flatten()
            .map(|point| point.j)
            .min()
            .unwrap_or(0);

        let mut tiles = tiles
            .iter()
            .map(|tile| {
                core::array::from_fn(|index| Point::from(tile[index].translate(-min_i, -min_j)))
            })
            .map(Tile::new)
            .collect::<Vec<_>>();

        tiles.sort();
        Self(tiles)
    }
}

impl<const LEN: usize> FromIterator<Tile<LEN>> for Set<LEN> {
    fn from_iter<T: IntoIterator<Item = Tile<LEN>>>(iter: T) -> Self {
        let mut tiles = Vec::from_iter(iter);
        tiles.sort();
        Self(tiles)
    }
}

// Invariant: `self.0` is sorted.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tile<const LEN: usize>([Point; LEN]);

impl<const LEN: usize> Tile<LEN> {
    pub const fn new(mut points: [Point; LEN]) -> Self {
        // Manual bubble sort to preserve `const` compatibility :(
        'outer: loop {
            let swap;
            let mut i = 0;

            while i + 1 < points.len() {
                match points[i].cmp(&points[i + 1]) {
                    Ordering::Less => i += 1,
                    Ordering::Equal | Ordering::Greater => {
                        swap = points[i];
                        points[i] = points[i + 1];
                        points[i + 1] = swap;
                        continue 'outer;
                    }
                }
            }

            break;
        }

        Self(points)
    }

    pub fn transformations(&self) -> impl Iterator<Item = Self> {
        [*self, self.reflect_x(), self.reflect_y()]
            .into_iter()
            .flat_map(|tile| [tile, tile.rotate_90(), tile.rotate_180(), tile.rotate_270()])
    }

    pub fn reflect_x(&self) -> Self {
        self.transform_clamp(SPoint::reflect_x)
    }

    pub fn reflect_y(&self) -> Self {
        self.transform_clamp(SPoint::reflect_y)
    }

    pub fn rotate_90(&self) -> Self {
        self.transform_clamp(SPoint::rotate_90)
    }

    pub fn rotate_180(&self) -> Self {
        self.transform_clamp(SPoint::rotate_180)
    }

    pub fn rotate_270(&self) -> Self {
        self.transform_clamp(SPoint::rotate_270)
    }

    fn transform_clamp<F: FnMut(&SPoint) -> SPoint>(&self, apply: F) -> Self {
        Self::clamp(self.transform(apply))
    }

    fn transform<F: FnMut(&SPoint) -> SPoint>(&self, mut apply: F) -> [SPoint; LEN] {
        core::array::from_fn(|index| apply(&SPoint::from(self.0[index])))
    }

    fn clamp(tile: [SPoint; LEN]) -> Tile<LEN> {
        let min_i = tile.iter().map(|point| point.i).min().unwrap_or(0);
        let min_j = tile.iter().map(|point| point.j).min().unwrap_or(0);
        let mut tile =
            core::array::from_fn(|index| Point::from(tile[index].translate(-min_i, -min_j)));
        tile.sort();
        Tile(tile)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    pub i: u8,
    pub j: u8,
}

impl Point {
    const fn cmp(&self, other: &Self) -> Ordering {
        if self.i > other.i {
            return Ordering::Greater;
        } else if self.i < other.i {
            return Ordering::Less;
        }

        if self.j > other.j {
            return Ordering::Greater;
        } else if self.j < other.j {
            return Ordering::Less;
        }

        Ordering::Equal
    }
}

// Intermediate representation to simplify 2D transformations.
#[derive(Copy, Clone, Debug)]
struct SPoint {
    i: i8,
    j: i8,
}

impl SPoint {
    fn translate(&self, di: i8, dj: i8) -> Self {
        Self {
            i: self.i + di,
            j: self.j + dj,
        }
    }

    fn reflect_x(&self) -> Self {
        Self {
            i: self.i,
            j: -self.j,
        }
    }

    fn reflect_y(&self) -> Self {
        Self {
            i: -self.i,
            j: self.j,
        }
    }

    fn rotate_90(&self) -> Self {
        Self {
            i: -self.j,
            j: self.i,
        }
    }

    fn rotate_180(&self) -> Self {
        Self {
            i: -self.i,
            j: -self.j,
        }
    }

    fn rotate_270(&self) -> Self {
        Self {
            i: self.j,
            j: -self.i,
        }
    }
}

impl From<Point> for SPoint {
    fn from(Point { i, j }: Point) -> Self {
        Self {
            i: i as _,
            j: j as _,
        }
    }
}

impl From<SPoint> for Point {
    fn from(SPoint { i, j }: SPoint) -> Self {
        Self {
            i: i as _,
            j: j as _,
        }
    }
}

impl<const LEN: usize> AsRef<[Point; LEN]> for Tile<LEN> {
    fn as_ref(&self) -> &[Point; LEN] {
        &self.0
    }
}

impl<const LEN: usize> AsMut<[Point; LEN]> for Tile<LEN> {
    fn as_mut(&mut self) -> &mut [Point; LEN] {
        &mut self.0
    }
}

#[macro_export]
macro_rules! tile {
    ($width:tt $index:tt: [$($acc:expr),*]) => {
        $crate::Tile::new([$($acc),*])
    };

    ($width:tt $index:tt: [$($acc:expr),*] X $($rest:tt)*) => {
        $crate::tile!($width ($index + 1): [$($acc ,)* $crate::tile::Point { i: $index / $width, j: $index % $width }] $($rest)*)
    };

    ($width:tt $index:tt: [$($acc:expr),*] . $($rest:tt)*) => {
        $crate::tile!($width ($index + 1): [$($acc),*] $($rest)*)
    };

    ($width:tt $($rest:tt)*) => {
        $crate::tile!($width 0: [] $($rest)*)
    };
}
