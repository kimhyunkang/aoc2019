#![feature(slice_as_array)]
#![feature(try_blocks)]

use label::Label;

use std::collections::HashMap;

use ndarray::Array2;

fn main() {
    println!("Hello, world!");
}

struct Maze {
    grid: Array2<bool>,
    aa: (usize, usize),
    zz: (usize, usize),
    n_idx: HashMap<Label, [(usize, usize); 2]>,
}

impl Maze {
    fn parse(input: &str) -> Self {
        let lines = input
            .lines()
            .map(|line| line.as_bytes())
            .collect::<Vec<_>>();
        let h = lines.len();
        let w = lines[0].len();
        let grid = Array2::from_shape_fn((h, w), |(r, c)| lines[r][c] == b'.');

        let lines_ref = &lines;
        let neighbors = |r: usize, c: usize| {
            [
                try { (r.checked_sub(1)?, c) },
                try { (r, c.checked_sub(1)?) },
                Some((r, c)),
                Some((r + 1, c)),
                Some((r, c + 1)),
            ]
            .into_iter()
            .filter_map(move |pos| {
                pos.and_then(|(r1, c1)| {
                    if h <= r1 || w <= c1 {
                        return None;
                    }
                    let b = lines_ref[r1][c1];
                    match b {
                        b'A'..=b'Z' | b'.' => Some((r1, c1)),
                        _ => None,
                    }
                })
            })
        };

        let mut name_idx: HashMap<Label, Vec<(usize, usize)>> = HashMap::new();
        for r in 0..h {
            for c in 0..w {
                if !(b'A'..=b'Z').contains(&lines[r][c]) {
                    continue;
                }
                let ns = neighbors(r, c).collect::<Vec<_>>();
                match ns.len() {
                    2 => continue,
                    3 => {
                        let exit = ns
                            .iter()
                            .copied()
                            .find(|&(r, c)| lines[r][c] == b'.')
                            .unwrap();
                        let name = ns
                            .iter()
                            .filter_map(|&(r, c)| {
                                let b = lines[r][c];
                                if b == b'.' { None } else { Some(b as char) }
                            })
                            .collect::<String>();
                        name_idx
                            .entry(name.as_str().into())
                            .or_insert_with(Vec::new)
                            .push(exit);
                    }
                    _ => panic!("Unexpected portal detection: {:?}", ns),
                }
            }
        }

        let mut n_idx = HashMap::new();
        let mut aa = None;
        let mut zz = None;
        for (portal, ps) in name_idx {
            match (portal.as_str(), &ps[..]) {
                ("AA", &[p0]) => {
                    aa = Some(p0);
                }
                ("ZZ", &[p0]) => {
                    zz = Some(p0);
                }
                (_, &[p0, p1]) => {
                    n_idx.insert(portal, [p0, p1]);
                }
                _ => {
                    panic!("Invalid portal instance: {:?}: {:?}", portal.as_str(), ps)
                }
            }
        }

        Self {
            grid,
            n_idx,
            aa: aa.unwrap(),
            zz: zz.unwrap(),
        }
    }
}

mod label {
    use std::{
        borrow::Borrow,
        fmt,
        hash::{Hash, Hasher},
    };

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Label([u8; 2]);

    impl Label {
        pub fn from_slice(slice: &[u8]) -> Self {
            Label(slice.as_array().copied().unwrap())
        }

        pub fn as_str(&self) -> &str {
            std::str::from_utf8(&self.0[..]).unwrap()
        }
    }

    impl From<&str> for Label {
        fn from(slice: &str) -> Self {
            Label::from_slice(slice.as_bytes())
        }
    }

    impl Borrow<str> for Label {
        fn borrow(&self) -> &str {
            self.as_str()
        }
    }

    impl Hash for Label {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.as_str().hash(state)
        }
    }

    impl fmt::Debug for Label {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "\"{}\"", self.as_str())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_label() {
        let bc = Label::from_slice(b"BC");
        assert_eq!(bc.as_str(), "BC");
    }

    #[test]
    fn test_parse() {
        let maze = Maze::parse(include_str!("test.txt"));
        assert_eq!(maze.aa, (2, 9));
        assert_eq!(maze.zz, (16, 13));
        dbg!(&maze.n_idx);
        assert_eq!(maze.n_idx.get("BC"), Some(&[(6, 9), (8, 2)]));
        assert_eq!(maze.n_idx.get("DE"), Some(&[(10, 6), (13, 2)]));
        assert_eq!(maze.n_idx.get("FG"), Some(&[(12, 11), (15, 2)]));
    }
}
