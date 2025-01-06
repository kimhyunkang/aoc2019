#![feature(unsigned_signed_diff)]
#![feature(try_blocks)]

use bitvec::boxed::BitBox;
use num_integer::gcd;
use ordered_float::OrderedFloat;

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let grid = Grid::parse(&input);
    let (pos, count) = grid.answer1().unwrap();
    println!("1: {}", count);
    let (r, c) = grid.vaporize(pos).nth(199).unwrap();
    println!("2: {}", c * 100 + r);
}

#[test]
fn test1() {
    let grid = Grid::parse(include_str!("test1.txt"));
    assert_eq!(grid.answer1().unwrap(), ((4, 3), 8));

    let grid = Grid::parse(include_str!("test2.txt"));
    assert_eq!(grid.answer1().unwrap(), ((8, 5), 33));

    let grid = Grid::parse(include_str!("test3.txt"));
    assert_eq!(grid.answer1().unwrap(), ((2, 1), 35));

    let grid = Grid::parse(include_str!("test4.txt"));
    assert_eq!(grid.answer1().unwrap(), ((3, 6), 41));

    let grid = Grid::parse(include_str!("test5.txt"));
    assert_eq!(grid.answer1().unwrap(), ((13, 11), 210));
}

#[test]
fn test_vaporize() {
    let grid = Grid::parse(include_str!("test_vaporize.txt"));
    let expected = vec![
        (1, 8),
        (0, 9),
        (1, 9),
        (0, 10),
        (2, 9),
        (1, 11),
        (1, 12),
        (2, 11),
        (1, 15),
        (2, 12),
        (2, 13),
        (2, 14),
        (2, 15),
        (3, 12),
        (4, 16),
        (4, 15),
        (4, 10),
        (4, 4),
        (4, 2),
        (3, 2),
        (2, 0),
        (2, 1),
        (1, 0),
        (1, 1),
        (2, 5),
        (0, 1),
        (1, 5),
        (1, 6),
        (0, 6),
        (0, 7),
        (0, 8),
        (1, 10),
        (0, 14),
        (1, 16),
        (3, 13),
        (3, 14),
    ];
    assert_eq!(grid.vaporize((3, 8)).collect::<Vec<_>>(), expected);
}

#[test]
fn test2() {
    let grid = Grid::parse(include_str!("test5.txt"));
    assert_eq!(grid.vaporize((13, 11)).nth(199), Some((2, 8)));
}

struct Vaporize {
    g: Grid,
    pos: (usize, usize),
}

impl Iterator for Vaporize {
    type Item = Vec<BitBox>;

    fn next(&mut self) -> Option<Vec<BitBox>> {
        let v = self.g.visible_asteroids(self.pos);
        let mut vaporized = false;
        for (g, v) in self.g.grid.iter_mut().zip(v.iter()) {
            if v.count_ones() > 0 {
                vaporized = true;
            }
            *g &= !(v.clone());
        }
        if vaporized { Some(v) } else { None }
    }
}

#[derive(Clone)]
struct Grid {
    dim: (usize, usize),
    grid: Vec<BitBox>,
}

impl Grid {
    fn parse(input: &str) -> Self {
        let lines = input
            .lines()
            .map(|line| line.as_bytes())
            .collect::<Vec<_>>();
        let dim = (lines.len(), lines[0].len());
        let grid: Vec<BitBox> = lines
            .iter()
            .map(|&line| line.iter().map(|&b| b == b'#').collect())
            .collect();
        Grid { dim, grid }
    }

    fn visible_asteroids(&self, (r0, c0): (usize, usize)) -> Vec<BitBox> {
        let mut grid = self.grid.clone();
        // Remove self from visibility
        grid[r0].set(c0, false);
        let (h, w) = self.dim;
        for r1 in (0..r0).rev().chain(r0..h) {
            for c1 in (0..c0).rev().chain(c0..w) {
                if (r1, c1) == (r0, c0) || !grid[r1][c1] {
                    continue;
                }
                for (r, c) in self.ray((r0, c0), (r1, c1)) {
                    grid[r].set(c, false);
                }
            }
        }
        grid
    }

    fn vaporize(&self, pos: (usize, usize)) -> impl Iterator<Item = (usize, usize)> {
        let (h, w) = self.dim;
        let iter = Vaporize {
            g: self.clone(),
            pos,
        };
        let (r0, c0) = pos;
        iter.flat_map(move |g| {
            let g_ref = &g;
            let mut asteroids = (0..h)
                .flat_map(move |r| {
                    (0..w).filter_map(move |c| if g_ref[r][c] { Some((r, c)) } else { None })
                })
                .collect::<Vec<_>>();

            //      y
            //   +------>
            //   |
            // x |
            //   |
            //   v
            asteroids.sort_by_key(|&(r1, c1)| {
                let x = r1.checked_signed_diff(r0).unwrap() as f32;
                let y = c1.checked_signed_diff(c0).unwrap() as f32;
                OrderedFloat(-f32::atan2(y, x))
            });

            asteroids
        })
    }

    fn count_visible(&self, pos: (usize, usize)) -> usize {
        let grid = self.visible_asteroids(pos);
        grid.into_iter().map(|row| row.count_ones()).sum::<usize>()
    }

    fn answer1(&self) -> Option<((usize, usize), usize)> {
        let (h, w) = self.dim;
        (0..h)
            .flat_map(|r| (0..w).map(move |c| (r, c)))
            .filter_map(|pos| {
                if self.grid[pos.0][pos.1] {
                    Some((pos, self.count_visible(pos)))
                } else {
                    None
                }
            })
            .max_by_key(|&(_, count)| count)
    }

    fn ray(
        &self,
        (r0, c0): (usize, usize),
        (r1, c1): (usize, usize),
    ) -> impl Iterator<Item = (usize, usize)> {
        let (h, w) = self.dim;
        let dr = r1.checked_signed_diff(r0).unwrap();
        let dc = c1.checked_signed_diff(c0).unwrap();
        let (dr, dc) = if dr == 0 || dc == 0 {
            (dr.signum(), dc.signum())
        } else {
            let g = gcd(dr.abs(), dc.abs());
            (dr / g, dc / g)
        };
        std::iter::successors(
            try { (r1.checked_add_signed(dr)?, c1.checked_add_signed(dc)?) },
            move |&(r, c)| try { (r.checked_add_signed(dr)?, c.checked_add_signed(dc)?) },
        )
        .take_while(move |&(r, c)| r < h && c < w)
    }
}
