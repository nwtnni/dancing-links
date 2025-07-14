#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    pub i: usize,
    pub j: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tile<const LEN: usize>([Point; LEN]);

impl<const LEN: usize> Tile<LEN> {
    pub const fn new(points: [Point; LEN]) -> Self {
        Self(points)
    }

    pub fn transformations(&self) -> impl Iterator<Item = Self> {
        [
            *self,
            self.reflect_x(),
            self.reflect_y(),
            self.rotate_90(),
            self.rotate_180(),
            self.rotate_270(),
        ]
        .into_iter()
    }

    pub fn reflect_x(&self) -> Self {
        self.transform(SPoint::reflect_x)
    }

    pub fn reflect_y(&self) -> Self {
        self.transform(SPoint::reflect_y)
    }

    pub fn rotate_90(&self) -> Self {
        self.transform(SPoint::rotate_90)
    }

    pub fn rotate_180(&self) -> Self {
        self.transform(SPoint::rotate_180)
    }

    pub fn rotate_270(&self) -> Self {
        self.transform(SPoint::rotate_270)
    }

    fn transform<F: FnMut(&SPoint) -> SPoint>(&self, mut apply: F) -> Self {
        clamp(core::array::from_fn(|index| {
            apply(&SPoint::from(self.0[index]))
        }))
    }
}

fn clamp<const LEN: usize>(tile: [SPoint; LEN]) -> Tile<LEN> {
    let min_i = tile.iter().map(|point| point.i).min().unwrap_or(0);
    let min_j = tile.iter().map(|point| point.j).min().unwrap_or(0);
    let mut tile = core::array::from_fn(|index| Point::from(tile[index].translate(-min_i, -min_j)));
    tile.sort();
    Tile(tile)
}

#[derive(Copy, Clone, Debug)]
struct SPoint {
    i: isize,
    j: isize,
}

impl SPoint {
    fn translate(&self, di: isize, dj: isize) -> Self {
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
            i: i as isize,
            j: j as isize,
        }
    }
}

impl From<SPoint> for Point {
    fn from(SPoint { i, j }: SPoint) -> Self {
        Self {
            i: i as usize,
            j: j as usize,
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
