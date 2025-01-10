#![feature(try_blocks)]

use std::collections::HashSet;

use ndarray::Array2;

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let st0 = Space::parse(&input);
    println!("1: {}", answer1(st0));
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

impl RecursiveSpace {
    fn parse(input: &str) -> Self {
        let lines = input.lines().map(str::as_bytes).collect::<Vec<_>>();
        let h = lines.len();
        let w = lines[0].len();
        RecursiveSpace {
            level: vec![Array2::from_shape_fn((h, w), |(r, c)| lines[r][c] == b'#')],
        }
    }

    fn inner_level(level0: &Array2<bool>) -> Option<Array2<bool>> {
        let grid = Array2::from_shape_fn((5, 5), |(r, c)| {
            let mut n: usize = 0;
            if r == 0 && level0[(1, 2)] {
                n += 1;
            }
            if r == 4 && level0[(3, 2)] {
                n += 1;
            }
            if c == 0 && level0[(2, 1)] {
                n += 1;
            }
            if c == 4 && level0[(2, 3)] {
                n += 1;
            }
            n == 1 || n == 2
        });
        if grid.iter().any(|&cell| cell) {
            Some(grid)
        } else {
            None
        }
    }

    fn outer_level(inner: &Array2<bool>) -> Option<Array2<bool>> {
        let cell12 = [1, 2].contains(&(0..5).filter(|&c| inner[(0, c)]).count());
        let cell32 = [1, 2].contains(&(0..5).filter(|&c| inner[(4, c)]).count());
        let cell21 = [1, 2].contains(&(0..5).filter(|&r| inner[(r, 0)]).count());
        let cell23 = [1, 2].contains(&(0..5).filter(|&r| inner[(r, 4)]).count());
        if cell12 || cell32 || cell21 || cell23 {
            let mut grid = Array2::from_elem((5, 5), false);
            grid[(1, 2)] = cell12;
            grid[(3, 2)] = cell32;
            grid[(2, 1)] = cell21;
            grid[(2, 3)] = cell23;
            Some(grid)
        } else {
            None
        }
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
