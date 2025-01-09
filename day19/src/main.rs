#![feature(try_blocks)]

use intcode::{VM, parse_program};
use log::{info, log_enabled};
use ndarray::Array2;

fn main() {
    env_logger::init();

    let input = std::fs::read_to_string("input.txt").unwrap();
    let program = parse_program(&input);
    let q = VMQuery { program };
    let (image, count) = scan(&q, 50);
    if log_enabled!(log::Level::Info) {
        for y in 0..50 {
            let buf = (0..50)
                .map(|x| if image[(x, y)] { '#' } else { '.' })
                .collect::<String>();
            info!("{}", buf);
        }
    }
    println!("1: {}", count);

    let (x, y) = search(&image, &q, 100);
    println!("2: {}", x * 10000 + y);
}

trait Query {
    fn query(&self, x: usize, y: usize) -> bool;
}

struct VMQuery {
    program: Vec<isize>,
}

impl Query for VMQuery {
    fn query(&self, x: usize, y: usize) -> bool {
        let mut vm = VM::init(self.program.clone());
        vm.write_port(&[x as isize, y as isize]);
        if !vm.run().is_ready() {
            panic!("VM should have shut down");
        }
        match vm.read_port().unwrap() {
            0 => false,
            1 => true,
            n => {
                panic!("Unexpected output {}", n);
            }
        }
    }
}

fn scan<Q: Query>(q: &Q, n: usize) -> (Array2<bool>, usize) {
    let mut count = 0;
    let mut field = Array2::from_elem((n, n), false);
    for y in 0..n {
        for x in 0..n {
            if q.query(x, y) {
                count += 1;
                field[(x, y)] = true;
            }
        }
    }
    (field, count)
}

fn slope(grid: &Array2<bool>) -> f32 {
    let n = grid.dim().0;
    let xs: Option<(usize, usize)> = try {
        (
            (0..n).find(|&x| grid[(x, n - 1)])?,
            (0..n).rev().find(|&x| grid[(x, n - 1)])?,
        )
    };
    let ys: Option<(usize, usize)> = try {
        (
            (0..n).find(|&y| grid[(n - 1, y)])?,
            (0..n).rev().find(|&y| grid[(n - 1, y)])?,
        )
    };
    if xs.is_none() && ys.is_none() {
        panic!("Slope not found")
    }
    let x = match xs {
        Some((min_x, max_x)) => (min_x + max_x) as f32 / 2.0,
        None => (n - 1) as f32,
    };
    let y = match ys {
        Some((min_y, max_y)) => (min_y + max_y) as f32 / 2.0,
        None => (n - 1) as f32,
    };
    y / x
}

// Finds the first element for which the predicate returns false
fn partition_point<F>(init: usize, mut f: F) -> usize
where
    F: FnMut(usize) -> bool,
{
    let mut lo = init;
    let mut hi = init;
    if f(init) {
        hi *= 2;
        while f(hi) {
            hi *= 2;
        }
    } else {
        lo /= 2;
        while lo > 0 && !f(lo) {
            lo /= 2;
        }
    }
    if lo == 0 {
        return lo;
    }

    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        if f(mid) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    hi
}

fn search<Q: Query>(init_image: &Array2<bool>, q: &Q, size: usize) -> (usize, usize) {
    let ratio = slope(init_image);

    let y_start = |x: usize| {
        let slope_y = (x as f32 * ratio).round() as usize;
        assert!(q.query(x, slope_y), "{:?}", (x, slope_y));
        // The first y that query returns true
        partition_point(slope_y, |y| !q.query(x, y))
    };
    let y_end = |x: usize| {
        let slope_y = (x as f32 * ratio).round() as usize;
        assert!(q.query(x, slope_y), "{:?}", (x, slope_y));
        // The first y that query returns false
        partition_point(slope_y, |y| q.query(x, y))
    };

    let (h, _) = init_image.dim();
    let min_x = partition_point(h - 1, |x| {
        let ye = y_end(x);
        let ys = y_start(x + size - 1);
        ye.checked_sub(ys).unwrap_or(0) < size
    });
    let min_y = y_start(min_x + size - 1);
    (min_x, min_y)
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestQuery {
        grid: Array2<bool>,
    }

    impl TestQuery {
        fn parse(input: &str) -> Self {
            let lines = input
                .lines()
                .map(|line| line.as_bytes())
                .collect::<Vec<_>>();
            let h = lines.len();
            let w = lines[0].len();
            let grid = Array2::from_shape_fn((w, h), |(x, y)| lines[y][x] == b'#');
            Self { grid }
        }
    }

    impl Query for TestQuery {
        fn query(&self, x: usize, y: usize) -> bool {
            self.grid[(x, y)]
        }
    }

    #[test]
    fn test2() {
        let q = TestQuery::parse(include_str!("test.txt"));
        let (image, _) = scan(&q, 5);
        assert_eq!(search(&image, &q, 6), (15, 12));
    }

    #[test]
    fn test_partition_point() {
        let x = partition_point(20, |n| n < 4);
        assert_eq!(x, 4);

        let x = partition_point(20, |n| n <= 100);
        assert_eq!(x, 101);
    }
}
