#![feature(iter_intersperse)]
#![feature(get_many_mut)]
#![feature(try_blocks)]

use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use bitvec::{bitbox, boxed::BitBox};
use intcode::{VM, parse_program};
use log::{info, log_enabled};
use ndarray::Array2;

fn main() {
    env_logger::init();

    let input = std::fs::read_to_string("input.txt").unwrap();
    let mut program = parse_program(&input);
    let image = get_camera(program.clone());
    for line in image.trim().lines() {
        info!("{}", line);
    }
    let scaffold = Scaffold::parse(&image);
    println!("1: {}", scaffold.count_crosses());
    let walk = scaffold.walk();
    info!("walk: {}", serialize(&walk));
    let code = Code::compress(&walk);

    program[0] = 2;
    let mut vm = VM::init(program);
    for routine in code.serialize() {
        info!("program: {}", routine);
        let mut buf = Vec::with_capacity(routine.len() + 1);
        buf.extend(routine.as_bytes().into_iter().map(|&b| b as isize));
        buf.push(10);
        vm.write_port(&buf);
    }
    vm.write_port(&[b'n' as isize, 10]);

    let mut buf: Vec<u8> = Vec::new();
    let mut score = 0;
    for b in vm.run_ready() {
        match b.try_into() {
            Ok(b'\n') => {
                if !buf.is_empty() {
                    info!("{}", std::str::from_utf8(&buf).unwrap());
                    buf.clear();
                }
            }
            Ok(b) => {
                buf.push(b);
            }
            _ => {
                score = b;
            }
        }
    }
    println!("2: {}", score);
}

#[test]
fn test2() {
    let scaffold = Scaffold::parse(include_str!("test.txt"));
    let walk = scaffold.walk();
    assert_eq!(
        serialize(&walk),
        "R,8,R,8,R,4,R,4,R,8,L,6,L,2,R,4,R,4,R,8,R,8,R,8,L,6,L,2"
    );
    let code = Code::compress(&walk);
    assert_eq!(code.rebuild(), walk);
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

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
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

struct Code<'w> {
    routine: Vec<usize>,
    subroutines: Vec<&'w [Walk]>,
}

impl<'w> fmt::Debug for Code<'w> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "routine: {:?}", self.routine)?;
        for (idx, subroutine) in self.subroutines.iter().enumerate() {
            writeln!(f, "subroutine[{}]: {:?}", idx, serialize(subroutine))?;
        }
        Ok(())
    }
}

impl<'w> Code<'w> {
    fn compress(walk: &'w [Walk]) -> Self {
        let mut words = Vec::new();
        let mut word_idx = HashMap::new();
        for i in 0..walk.len() - 1 {
            for j in (i + 1)..walk.len() {
                let word = &walk[i..j];
                if serialize(word).len() <= 20 {
                    word_idx.entry(word).or_insert_with(|| {
                        let idx = words.len();
                        words.push(word);
                        idx
                    });
                }
            }
        }

        // Find cover using 3 or less words

        // Set up array for dynamic programming:
        // cover[i] = list of subsets with 3 or less words that cover walk[0..i]
        let mut cover: Vec<HashSet<BitBox>> = vec![HashSet::new(); walk.len() + 1];
        cover[0].insert(bitbox![0; words.len()]);
        for j in 1..=walk.len() {
            for i in 0..j - 1 {
                let word = &walk[i..j];
                if let Some(&idx) = word_idx.get(&word) {
                    let [ci, cj] = cover.get_many_mut([i, j]).unwrap();
                    for subset in ci.iter() {
                        if subset.count_ones() < 3 || subset[idx] {
                            let mut subset = subset.clone();
                            subset.set(idx, true);
                            cj.insert(subset);
                        }
                    }
                }
            }
        }

        for subset in cover.pop().unwrap() {
            let subroutines = subset.iter_ones().map(|idx| words[idx]).collect::<Vec<_>>();
            if log_enabled!(log::Level::Info) {
                for (i, subroutine) in subroutines.iter().enumerate() {
                    info!("subroutine[{}]: [{}]", i, serialize(subroutine));
                }
            }

            // compressed[i] = list of possible routines made out of the subroutines
            let mut compressed: Vec<Vec<Vec<usize>>> = vec![vec![]; walk.len() + 1];
            compressed[0].push(vec![]);
            // Rebuild the cover using the subset found
            for j in 1..=walk.len() {
                for i in 0..j - 1 {
                    let word = &walk[i..j];
                    if let Some(r_idx) = subroutines.iter().position(|&w| w == word) {
                        let [ci, cj] = compressed.get_many_mut([i, j]).unwrap();
                        for mut prefix in ci.iter().cloned() {
                            prefix.push(r_idx);
                            cj.push(prefix);
                        }
                    }
                }
            }

            for routine in compressed.pop().unwrap() {
                if routine.len() <= 10 {
                    return Self {
                        routine,
                        subroutines,
                    };
                }
            }
        }

        panic!("Program not found");
    }

    #[cfg(test)]
    fn rebuild(&self) -> Vec<Walk> {
        let mut walk = Vec::new();
        for &idx in &self.routine {
            walk.extend_from_slice(self.subroutines[idx]);
        }
        walk
    }

    fn serialize(&self) -> [String; 4] {
        static PROGRAM: [char; 3] = ['A', 'B', 'C'];
        let mut routines = core::array::from_fn(|_| String::new());
        routines[0] = self
            .routine
            .iter()
            .map(|&i| PROGRAM[i])
            .intersperse(',')
            .collect();
        for (i, routine) in self.subroutines.iter().enumerate() {
            routines[i + 1] = serialize(routine);
        }
        routines
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
            .find(|&(_, tile)| b"^v<>".contains(tile))
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
