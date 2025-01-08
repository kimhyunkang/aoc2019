#![feature(iter_intersperse)]
#![feature(try_blocks)]

use core::fmt;
use std::collections::HashSet;

use intcode::{VM, parse_program};
use ndarray::Array2;

fn main() {
    env_logger::init();

    let input = std::fs::read_to_string("input.txt").unwrap();
    let program = parse_program(&input);
    let image = get_camera(program);
    println!("{}", image.trim());
    let scaffold = Scaffold::parse(&image);
    println!("1: {}", scaffold.count_crosses());
    let walk = scaffold.walk();
    println!("2: {}", serialize(&walk));
    compress(&walk);
}

#[test]
fn test2() {
    let scaffold = Scaffold::parse(include_str!("test.txt"));
    assert_eq!(
        serialize(&scaffold.walk()),
        "R,8,R,8,R,4,R,4,R,8,L,6,L,2,R,4,R,4,R,8,R,8,R,8,L,6,L,2"
    );
}

fn get_camera(program: Vec<isize>) -> String {
    let mut vm = VM::init(program);
    assert!(vm.run().is_ready());
    String::from_utf8(
        vm.read_all()
            .into_iter()
            .map(|n| {
                let b: u8 = n.try_into().unwrap();
                b
            })
            .collect(),
    )
    .unwrap()
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum Walk {
    R,
    L,
    F(usize),
}

impl fmt::Display for Walk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Walk::R => write!(f, "R",),
            Walk::L => write!(f, "L",),
            Walk::F(n) => write!(f, "{}", *n),
        }
    }
}

fn serialize(walk: &[Walk]) -> String {
    walk.iter()
        .map(|s| s.to_string())
        .intersperse(",".to_string())
        .collect::<String>()
}

fn compress(walk: &[Walk]) {
    let mut words = HashSet::new();
    for i in 0..walk.len() - 1 {
        for j in i + 1..walk.len() {
            let word = &walk[i..j];
            if serialize(word).len() <= 20 {
                words.insert(word);
            }
        }
    }
    let words = words.into_iter().collect::<Vec<_>>();
    for word in words {
        println!("{}", serialize(word));
    }
}

struct Scaffold {
    grid: Array2<u8>,
}

impl Scaffold {
    fn parse(image: &str) -> Self {
        let lines = image
            .lines()
            .map(|line| line.as_bytes())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>();
        let h = lines.len();
        let w = lines[0].len();
        Self {
            grid: Array2::from_shape_fn((h, w), |(r, c)| lines[r][c]),
        }
    }

    fn count_crosses(&self) -> usize {
        static DIR: [(isize, isize); 5] = [(-1, 0), (0, -1), (0, 0), (0, 1), (1, 0)];

        let mut sum = 0;
        let (h, w) = self.grid.dim();
        for r in 1..h - 1 {
            for c in 1..w - 1 {
                if DIR
                    .iter()
                    .map(|&(dr, dc)| {
                        (
                            r.checked_add_signed(dr).unwrap(),
                            c.checked_add_signed(dc).unwrap(),
                        )
                    })
                    .all(|pos| self.grid[pos] == b'#')
                {
                    sum += r * c;
                }
            }
        }
        sum
    }

    fn neighbor(&self, (r, c): (usize, usize), (dr, dc): (isize, isize)) -> Option<(usize, usize)> {
        let (h, w) = self.grid.dim();
        let r = r.checked_add_signed(dr)?;
        let c = c.checked_add_signed(dc)?;
        if r < h && c < w { Some((r, c)) } else { None }
    }

    fn walk(&self) -> Vec<Walk> {
        let mut plan = Vec::new();
        let (start, &dir) = self
            .grid
            .indexed_iter()
            .find(|&(pos, tile)| b"^v<>".contains(tile))
            .unwrap();
        let (mut dr, mut dc) = match dir {
            b'^' => (-1, 0),
            b'v' => (1, 0),
            b'<' => (0, -1),
            b'>' => (0, 1),
            _ => unreachable!(),
        };

        let mut pos = start;

        loop {
            let left = (-dc, dr);
            let right = (dc, -dr);
            if self.neighbor(pos, left).map(|p| self.grid[p]) == Some(b'#') {
                plan.push(Walk::L);
                (dr, dc) = left;
            } else if self.neighbor(pos, right).map(|p| self.grid[p]) == Some(b'#') {
                plan.push(Walk::R);
                (dr, dc) = right;
            }

            let mut forward = 0;
            while let Some(p) = self.neighbor(pos, (dr, dc)) {
                if self.grid[p] == b'#' {
                    forward += 1;
                    pos = p;
                } else {
                    break;
                }
            }
            if forward == 0 {
                break;
            } else {
                plan.push(Walk::F(forward));
            }
        }

        plan
    }
}
