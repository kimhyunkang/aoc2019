#![feature(map_try_insert)]
#![feature(slice_as_array)]
#![feature(try_blocks)]

use label::Label;

use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet, VecDeque},
};

use ndarray::Array2;

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let maze = Maze::parse(&input);
    println!("1: {}", maze.shortest_path().unwrap());
    println!("2: {}", maze.recursive_path().unwrap());
}

#[derive(PartialEq, Eq, Debug)]
struct Conn {
    inner: (usize, usize),
    outer: (usize, usize),
}

struct Maze {
    grid: Array2<bool>,
    aa: (usize, usize),
    zz: (usize, usize),
    n_idx: HashMap<Label, Conn>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Portal {
    Inner(Label),
    Outer(Label),
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

        let outer = |(r, c)| r == 2 || r == h - 3 || c == 2 || c == w - 3;
        let mut n_idx = HashMap::new();
        let mut aa = None;
        let mut zz = None;
        for (name, ps) in name_idx {
            match (name.as_str(), &ps[..]) {
                ("AA", &[p0]) => {
                    aa = Some(p0);
                }
                ("ZZ", &[p0]) => {
                    zz = Some(p0);
                }
                (_, &[p0, p1]) => {
                    let portal = if outer(p0) {
                        Conn {
                            inner: p1,
                            outer: p0,
                        }
                    } else if outer(p1) {
                        Conn {
                            inner: p0,
                            outer: p1,
                        }
                    } else {
                        panic!("Invalid portal at {:?} and {:?}", p0, p1);
                    };
                    n_idx.insert(name, portal);
                }
                _ => {
                    panic!("Invalid portal instance: {:?}: {:?}", name.as_str(), ps)
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

    fn neighbors(&self, (r, c): (usize, usize)) -> impl Iterator<Item = (usize, usize)> {
        [
            try { (r.checked_sub(1)?, c) },
            try { (r, c.checked_sub(1)?) },
            Some((r + 1, c)),
            Some((r, c + 1)),
        ]
        .into_iter()
        .filter_map(move |pos| pos.filter(|&p| self.grid.get(p).copied().unwrap_or(false)))
    }

    fn shortest_path(&self) -> Option<usize> {
        let mut portals = HashMap::new();
        for &Conn { inner, outer } in self.n_idx.values() {
            portals.insert(inner, outer);
            portals.insert(outer, inner);
        }

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back((self.aa, 0));
        visited.insert(self.aa);

        while let Some((p0, d0)) = queue.pop_front() {
            if p0 == self.zz {
                return Some(d0);
            }
            for p1 in self.neighbors(p0) {
                if visited.insert(p1) {
                    queue.push_back((p1, d0 + 1));
                }
            }
            if let Some(&p1) = portals.get(&p0) {
                if visited.insert(p1) {
                    queue.push_back((p1, d0 + 1));
                }
            }
        }

        None
    }

    fn paths(
        &self,
        p_idx: &HashMap<(usize, usize), Portal>,
        start: (usize, usize),
    ) -> HashMap<Portal, usize> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut paths = HashMap::new();

        queue.push_back((start, 0));
        visited.insert(start);

        while let Some((p0, d0)) = queue.pop_front() {
            if let Some(&portal) = p_idx.get(&p0) {
                paths.insert(portal, d0);
            }

            for p1 in self.neighbors(p0) {
                if visited.insert(p1) {
                    queue.push_back((p1, d0 + 1));
                }
            }
        }

        paths
    }

    fn all_paths(&self) -> HashMap<Portal, HashMap<Portal, usize>> {
        let mut p_idx = HashMap::new();
        let aa = Portal::Outer("AA".into());
        let zz = Portal::Outer("ZZ".into());
        p_idx.insert(self.aa, aa);
        p_idx.insert(self.zz, zz);
        for (&name, &Conn { inner, outer }) in &self.n_idx {
            p_idx.insert(inner, Portal::Inner(name));
            p_idx.insert(outer, Portal::Outer(name));
        }

        let mut all_paths = HashMap::new();
        all_paths.insert(aa, self.paths(&p_idx, self.aa));
        for (&name, &Conn { inner, outer }) in &self.n_idx {
            all_paths.insert(Portal::Inner(name), self.paths(&p_idx, inner));
            all_paths.insert(Portal::Outer(name), self.paths(&p_idx, outer));
        }

        all_paths
    }

    fn recursive_path(&self) -> Option<usize> {
        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        struct Node {
            dist: Reverse<usize>,
            vertex: (Portal, usize),
        }

        let all_paths = self.all_paths();

        let mut queue = BinaryHeap::new();
        let mut dist = HashMap::new();

        let aa = Portal::Outer("AA".into());
        let zz = Portal::Outer("ZZ".into());
        let start = (aa, 0);
        queue.push(Node {
            dist: Reverse(0),
            vertex: start,
        });
        dist.insert(start, 0);

        while let Some(node) = queue.pop() {
            let Node {
                dist: Reverse(d0),
                vertex: (p0, depth),
            } = node;

            if p0 == zz && depth == 0 {
                return Some(d0);
            }

            for (&dest, &cost) in all_paths.get(&p0).unwrap() {
                if dest == aa {
                    continue;
                }
                let (p1, d1) = match dest {
                    Portal::Inner(l1) => ((Portal::Outer(l1), depth + 1), d0 + cost + 1),
                    Portal::Outer(l1) => {
                        if depth == 0 && dest == zz {
                            ((zz, 0), d0 + cost)
                        } else if depth > 0 && dest != zz {
                            ((Portal::Inner(l1), depth - 1), d0 + cost + 1)
                        } else {
                            continue;
                        }
                    }
                };
                let d1_so_far = dist.entry(p1).or_insert(usize::MAX);
                if d1 < *d1_so_far {
                    *d1_so_far = d1;
                    queue.push(Node {
                        dist: Reverse(d1),
                        vertex: p1,
                    });
                }
            }
        }

        None
    }
}

mod label {
    use std::{
        borrow::Borrow,
        fmt,
        hash::{Hash, Hasher},
    };

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
        let maze = Maze::parse(include_str!("test1.txt"));
        assert_eq!(maze.aa, (2, 9));
        assert_eq!(maze.zz, (16, 13));

        assert_eq!(
            maze.n_idx.get("BC"),
            Some(&Conn {
                inner: (6, 9),
                outer: (8, 2)
            })
        );
        assert_eq!(
            maze.n_idx.get("DE"),
            Some(&Conn {
                inner: (10, 6),
                outer: (13, 2)
            })
        );
        assert_eq!(
            maze.n_idx.get("FG"),
            Some(&Conn {
                inner: (12, 11),
                outer: (15, 2)
            })
        );
    }

    #[test]
    fn test1() {
        let maze = Maze::parse(include_str!("test1.txt"));
        assert_eq!(maze.shortest_path(), Some(23));

        let maze = Maze::parse(include_str!("test2.txt"));
        assert_eq!(maze.shortest_path(), Some(58));
    }

    #[test]
    fn test_all_paths() {
        let maze = Maze::parse(include_str!("test1.txt"));
        let paths = maze.all_paths();
        assert_eq!(
            paths
                .get(&Portal::Outer("AA".into()))
                .unwrap()
                .get(&Portal::Inner("BC".into()))
                .copied(),
            Some(4)
        );

        assert_eq!(
            paths
                .get(&Portal::Outer("BC".into()))
                .unwrap()
                .get(&Portal::Inner("DE".into()))
                .copied(),
            Some(6)
        );

        assert_eq!(
            paths
                .get(&Portal::Inner("FG".into()))
                .unwrap()
                .get(&Portal::Outer("ZZ".into()))
                .copied(),
            Some(6)
        );
    }

    #[test]
    fn test2() {
        let maze = Maze::parse(include_str!("test3.txt"));
        assert_eq!(maze.recursive_path(), Some(396));
    }
}
