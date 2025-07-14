#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub i: usize,
    pub j: usize,
}

#[macro_export]
macro_rules! tile {
    ($width:tt $index:tt: [$($acc:expr),*]) => {
        [$($acc),*]
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
