#![feature(try_blocks)]

use std::collections::HashSet;

use ndarray::Array2;

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let st0 = Space::parse(&input);
    println!("1: {}", answer1(st0));
    let mut st1 = RecursiveSpace::parse(&input);
    for _ in 0..200 {
        st1 = st1.step();
    }
    println!("2: {}", st1.count());
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Space(Array2<bool>);

impl Space {
    fn parse(input: &str) -> Self {
        let lines = input.lines().map(str::as_bytes).collect::<Vec<_>>();
        let h = lines.len();
        let w = lines[0].len();
        Space(Array2::from_shape_fn((h, w), |(r, c)| lines[r][c] == b'#'))
    }

    fn neighbors(&self, (r, c): (usize, usize)) -> usize {
        let neighbors = [
            try { (r.checked_sub(1)?, c) },
            try { (r, c.checked_sub(1)?) },
            Some((r + 1, c)),
            Some((r, c + 1)),
        ]
        .into_iter()
        .filter_map(|pos| pos);
        neighbors
            .filter(|&pos| self.0.get(pos).copied().unwrap_or(false))
            .count()
    }

    fn step(&self) -> Self {
        Space(Array2::from_shape_fn(self.0.dim(), |pos| {
            let n = self.neighbors(pos);
            if self.0[pos] {
                n == 1
            } else {
                n == 1 || n == 2
            }
        }))
    }

    fn biodiversity(&self) -> usize {
        let (_, w) = self.0.dim();
        let mut diversity = 0;
        for ((r, c), &cell) in self.0.indexed_iter() {
            if cell {
                diversity |= 1 << (r * w + c);
            }
        }
        diversity
    }
}

struct RecursiveSpace {
    level: Vec<Array2<bool>>,
}

struct Border<T> {
    n: T,
    s: T,
    e: T,
    w: T,
}

impl RecursiveSpace {
    fn parse(input: &str) -> Self {
        let lines = input.lines().map(str::as_bytes).collect::<Vec<_>>();
        let h = lines.len();
        let w = lines[0].len();
        RecursiveSpace {
            level: vec![Array2::from_shape_fn((h, w), |(r, c)| lines[r][c] == b'#')],
        }
    }

    fn outer_border(outer: &Array2<bool>) -> Border<bool> {
        Border {
            n: outer[(1, 2)],
            s: outer[(3, 2)],
            w: outer[(2, 1)],
            e: outer[(2, 3)],
        }
    }

    fn inner_border(inner: &Array2<bool>) -> Border<usize> {
        Border {
            n: (0..5).filter(|&c| inner[(0, c)]).count(),
            s: (0..5).filter(|&c| inner[(4, c)]).count(),
            w: (0..5).filter(|&r| inner[(r, 0)]).count(),
            e: (0..5).filter(|&r| inner[(r, 4)]).count(),
        }
    }

    fn inner_level(&self) -> Option<Array2<bool>> {
        let border = Self::outer_border(&self.level[0]);
        if border.n || border.s || border.w || border.e {
            Some(Array2::from_shape_fn((5, 5), |(r, c)| {
                let mut n = 0;
                if r == 0 && border.n {
                    n += 1;
                }
                if r == 4 && border.s {
                    n += 1;
                }
                if c == 0 && border.w {
                    n += 1;
                }
                if c == 4 && border.e {
                    n += 1;
                }
                n == 1 || n == 2
            }))
        } else {
            None
        }
    }

    fn outer_level(&self) -> Option<Array2<bool>> {
        static RANGE: [usize; 2] = [1, 2];
        let border = Self::inner_border(self.level.last().unwrap());
        if RANGE.contains(&border.n)
            || RANGE.contains(&border.s)
            || RANGE.contains(&border.w)
            || RANGE.contains(&border.e)
        {
            let mut grid = Array2::from_elem((5, 5), false);
            if RANGE.contains(&border.n) {
                grid[(1, 2)] = true;
            }
            if RANGE.contains(&border.s) {
                grid[(3, 2)] = true;
            }
            if RANGE.contains(&border.w) {
                grid[(2, 1)] = true;
            }
            if RANGE.contains(&border.e) {
                grid[(2, 3)] = true;
            }
            Some(grid)
        } else {
            None
        }
    }

    fn neighbors(level: &Array2<bool>, (r, c): (usize, usize)) -> usize {
        [
            try { (r.checked_sub(1)?, c) },
            try { (r, c.checked_sub(1)?) },
            Some((r + 1, c)),
            Some((r, c + 1)),
        ]
        .into_iter()
        .flatten()
        .filter(|&pos| level.get(pos).copied().unwrap_or(false))
        .count()
    }

    fn next_level(&self, idx: usize) -> Array2<bool> {
        let inner = idx
            .checked_sub(1)
            .and_then(|i| self.level.get(i))
            .map(Self::inner_border);
        let outer = self.level.get(idx + 1).map(Self::outer_border);
        Array2::from_shape_fn((5, 5), |(r, c)| {
            let mut n = Self::neighbors(&self.level[idx], (r, c));
            if r == 0 && outer.as_ref().map(|b| b.n).unwrap_or(false) {
                n += 1;
            }
            if r == 4 && outer.as_ref().map(|b| b.s).unwrap_or(false) {
                n += 1;
            }
            if c == 0 && outer.as_ref().map(|b| b.w).unwrap_or(false) {
                n += 1;
            }
            if c == 4 && outer.as_ref().map(|b| b.e).unwrap_or(false) {
                n += 1;
            }
            match (r, c) {
                (1, 2) => {
                    n += inner.as_ref().map(|b| b.n).unwrap_or(0);
                }
                (3, 2) => {
                    n += inner.as_ref().map(|b| b.s).unwrap_or(0);
                }
                (2, 1) => {
                    n += inner.as_ref().map(|b| b.w).unwrap_or(0);
                }
                (2, 3) => {
                    n += inner.as_ref().map(|b| b.e).unwrap_or(0);
                }
                _ => (),
            }
            if (r, c) == (2, 2) {
                false
            } else if self.level[idx][(r, c)] {
                n == 1
            } else {
                n == 1 || n == 2
            }
        })
    }

    fn step(&self) -> Self {
        let level = self
            .inner_level()
            .into_iter()
            .chain((0..self.level.len()).map(|i| self.next_level(i)))
            .chain(self.outer_level())
            .collect();
        RecursiveSpace { level }
    }

    fn count(&self) -> usize {
        self.level
            .iter()
            .map(|grid| grid.iter().filter(|&&cell| cell).count())
            .sum::<usize>()
    }
}

fn answer1(s0: Space) -> usize {
    let mut visited = HashSet::new();
    let mut s = s0;

    while visited.insert(s.clone()) {
        s = s.step();
    }

    s.biodiversity()
}

#[test]
fn test_step() {
    let stage0 = Space::parse(include_str!("stage0.txt"));
    let stage1 = Space::parse(include_str!("stage1.txt"));
    let stage2 = Space::parse(include_str!("stage2.txt"));

    assert_eq!(stage0.step(), stage1);
    assert_eq!(stage1.step(), stage2);
}

#[test]
fn test1() {
    let st0 = Space::parse(include_str!("stage0.txt"));
    assert_eq!(answer1(st0), 2129920);
}

#[test]
fn test2() {
    let mut st = RecursiveSpace::parse(include_str!("stage0.txt"));
    for _ in 0..10 {
        st = st.step();
    }
    assert_eq!(st.count(), 99);
}
