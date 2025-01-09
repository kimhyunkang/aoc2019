#![feature(iter_intersperse)]
#![feature(try_blocks)]

use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, VecDeque},
    fmt,
};

use log::log_enabled;
use ndarray::Array2;

fn main() {
    env_logger::init();

    let input = std::fs::read_to_string("input.txt").unwrap();
    let maze = Maze::parse(&input);

    // Verify that the maze is cycle-free
    // Maze being cycle-free means there is only 1 path between any two keys, and will be blocked if a closed door is between them
    maze.verify_tree();
    maze.verify_entrance();

    println!("1: {}", maze.tsp().unwrap());
    println!("2: {}", tsp4(&maze.split()).unwrap());
}

#[derive(Clone, Copy)]
struct KeyPath {
    dist: usize,
    doors: KeySet,
}

struct Maze {
    grid: Array2<u8>,
    entrance: (usize, usize),
    keys: HashMap<u8, (usize, usize)>,
}

impl Maze {
    fn parse(input: &str) -> Self {
        let lines = input
            .lines()
            .map(|line| line.as_bytes())
            .collect::<Vec<_>>();
        let h = lines.len();
        let w = lines[0].len();
        let mut entrance = None;
        let mut keys = HashMap::new();
        let grid = Array2::from_shape_fn((h, w), |(r, c)| {
            let ch = lines[r][c];
            match ch {
                b'a'..=b'z' => {
                    keys.insert(ch, (r, c));
                }
                b'@' => {
                    entrance = Some((r, c));
                }
                b'.' | b'#' | b'A'..=b'Z' => (),
                _ => panic!("Invalid character {:?}", ch as char),
            };
            ch
        });
        Self {
            grid,
            entrance: entrance.unwrap(),
            keys,
        }
    }

    fn neighbors(&self, (r, c): (usize, usize)) -> impl Iterator<Item = (usize, usize)> {
        let (h, w) = self.grid.dim();
        [
            try { (r.checked_sub(1)?, c) },
            Some((r + 1, c)),
            try { (r, c.checked_sub(1)?) },
            Some((r, c + 1)),
        ]
        .into_iter()
        .filter_map(|pos| pos)
        .filter(move |&(r, c)| r < h && c < w)
    }

    // Verify that the maze does not have a cycle except the 3x3 square around the entrance
    fn verify_tree(&self) {
        let mut queue = VecDeque::new();
        let mut prev: Array2<Option<(usize, usize)>> = Array2::from_elem(self.grid.dim(), None);
        prev[self.entrance] = Some(self.entrance);
        queue.push_back(self.entrance);

        while let Some(p0) = queue.pop_front() {
            let parent = prev[p0].unwrap();
            for p1 in self.neighbors(p0) {
                if self.grid[p1] == b'#' || p1 == parent {
                    continue;
                } else if let Some((r1, c1)) = prev[p1] {
                    // Ignore the 3x3 cycle around the entrance
                    let (r0, c0) = self.entrance;
                    if r1.abs_diff(r0) > 1 || c1.abs_diff(c0) > 1 {
                        panic!("Cycle found at {:?}", (r1, c1));
                    }
                } else {
                    prev[p1] = Some(p0);
                    queue.push_back(p1);
                }
            }
        }
    }

    // Verify that the 3x3 square around the entrance is clear from doors or keys
    fn verify_entrance(&self) {
        let (r0, c0) = self.entrance;
        for r in r0 - 1..=r0 + 1 {
            for c in c0 - 1..=c0 + 1 {
                match self.grid[(r, c)] {
                    b'a'..=b'z' => {
                        panic!("Key {:?} found around the entrance", self.grid[(r, c)])
                    }
                    b'A'..=b'Z' => {
                        panic!("Door {:?} found around the entrance", self.grid[(r, c)])
                    }
                    _ => (),
                }
            }
        }
    }

    // Search path to all keys
    fn search_path(&self, start: (usize, usize)) -> HashMap<u8, KeyPath> {
        let mut queue = VecDeque::new();
        let mut path: Array2<Option<KeyPath>> = Array2::from_elem(self.grid.dim(), None);
        path[start] = Some(KeyPath {
            dist: 0,
            doors: KeySet::new(),
        });
        queue.push_back(start);
        let mut key_path = HashMap::new();

        while let Some(p0) = queue.pop_front() {
            let e0 = path[p0].unwrap();
            for p1 in self.neighbors(p0) {
                let ch = self.grid[p1];
                if ch == b'#' || path[p1].is_some() {
                    continue;
                }
                let mut doors = e0.doors;
                if let b'A'..=b'Z' = ch {
                    doors.insert_door(ch);
                }
                let entry = KeyPath {
                    dist: e0.dist + 1,
                    doors,
                };
                path[p1] = Some(entry);
                if let b'a'..=b'z' = ch {
                    key_path.insert(ch, entry);
                    // Do not explore further, as we're not interested in a path that includes another key
                } else {
                    queue.push_back(p1);
                }
            }
        }

        key_path
    }

    fn all_path(&self) -> HashMap<u8, HashMap<u8, KeyPath>> {
        let mut paths = HashMap::new();
        paths.insert(b'@', self.search_path(self.entrance));
        for (&key, &pos) in &self.keys {
            paths.insert(key, self.search_path(pos));
        }
        paths
    }

    fn tsp(&self) -> Option<usize> {
        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        struct Node {
            dist: Reverse<usize>,
            pos: u8,
            keys: KeySet,
        }

        let paths = self.all_path();

        if log_enabled!(log::Level::Info) {
            for (&k0, ps) in &paths {
                for (&k1, path) in ps {
                    log::info!(
                        "{}-{}: {} {:?}",
                        k0 as char,
                        k1 as char,
                        path.dist,
                        path.doors
                    );
                }
            }
        }

        let all_keys = KeySet::from_keys(self.keys.keys().copied());
        let mut queue = BinaryHeap::new();
        let mut dist = HashMap::new();
        queue.push(Node {
            dist: Reverse(0),
            pos: b'@',
            keys: KeySet::new(),
        });
        dist.insert((b'@', KeySet::new()), 0);

        while let Some(entry) = queue.pop() {
            let Reverse(d0) = entry.dist;
            if entry.keys == all_keys {
                return Some(d0);
            }
            for (&k1, path) in paths.get(&entry.pos).unwrap() {
                if !entry.keys.can_open(&path.doors) {
                    continue;
                }
                let mut keys = entry.keys;
                keys.insert_key(k1);
                let d1 = dist.get(&(k1, keys)).copied().unwrap_or(usize::MAX);
                if d0 + path.dist < d1 {
                    let d1 = d0 + path.dist;
                    dist.insert((k1, keys), d1);
                    queue.push(Node {
                        dist: Reverse(d1),
                        pos: k1,
                        keys,
                    });
                }
            }
        }

        None
    }

    // Split the maze into 4
    fn split(&self) -> [Self; 4] {
        let mut grid = self.grid.clone();
        let (r0, c0) = self.entrance;
        grid[(r0, c0)] = b'#';
        grid[(r0, c0 - 1)] = b'#';
        grid[(r0, c0 + 1)] = b'#';
        grid[(r0 - 1, c0)] = b'#';
        grid[(r0 + 1, c0)] = b'#';
        let entrance = [
            (r0 - 1, c0 - 1),
            (r0 - 1, c0 + 1),
            (r0 + 1, c0 - 1),
            (r0 + 1, c0 + 1),
        ];
        core::array::from_fn(|i| Self {
            grid: grid.clone(),
            entrance: entrance[i],
            keys: self.keys.clone(),
        })
    }
}

fn tsp4(maze: &[Maze; 4]) -> Option<usize> {
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    struct Node {
        dist: Reverse<usize>,
        pos: [u8; 4],
        keys: KeySet,
    }

    let paths: [_; 4] = core::array::from_fn(|i| maze[i].all_path());

    let all_keys = KeySet::from_keys(maze[0].keys.keys().copied());
    let mut queue = BinaryHeap::new();
    let mut dist = HashMap::new();
    queue.push(Node {
        dist: Reverse(0),
        pos: [b'@'; 4],
        keys: KeySet::new(),
    });
    dist.insert(([b'@'; 4], KeySet::new()), 0);

    while let Some(entry) = queue.pop() {
        let Reverse(d0) = entry.dist;
        if entry.keys == all_keys {
            return Some(d0);
        }
        for i in 0..4 {
            for (&k1, path) in paths[i].get(&entry.pos[i]).unwrap() {
                if !entry.keys.can_open(&path.doors) {
                    continue;
                }
                let mut keys = entry.keys;
                keys.insert_key(k1);
                let mut pos1 = entry.pos;
                pos1[i] = k1;
                let d1 = dist.get(&(pos1, keys)).copied().unwrap_or(usize::MAX);
                if d0 + path.dist < d1 {
                    let d1 = d0 + path.dist;
                    dist.insert((pos1, keys), d1);
                    queue.push(Node {
                        dist: Reverse(d1),
                        pos: pos1,
                        keys,
                    });
                }
            }
        }
    }

    None
}

// Represents a set of doors
// Using u32 because there are only 26 doors max
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct KeySet(u32);

impl KeySet {
    fn new() -> Self {
        Self(0)
    }

    fn insert_door(&mut self, door: u8) {
        self.0 |= 1 << (door - b'A');
    }

    fn insert_key(&mut self, key: u8) {
        self.0 |= 1 << (key - b'a');
    }

    // This keyset can open all doors in the set
    fn can_open(&self, doors: &Self) -> bool {
        ((!self.0) & doors.0) == 0
    }

    fn from_keys<I: Iterator<Item = u8>>(iter: I) -> Self {
        let mut keys = Self::new();
        for key in iter {
            keys.insert_key(key);
        }
        keys
    }
}

impl fmt::Debug for KeySet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for ch in (0..26)
            .filter(|&i| (self.0 & (1 << i)) != 0)
            .map(|i| (i + b'a') as char)
            .intersperse(',')
        {
            write!(f, "{}", ch)?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test0() {
        init();

        let maze = Maze::parse(include_str!("test0.txt"));
        assert_eq!(maze.tsp(), Some(8));
    }

    #[test]
    fn test1() {
        init();

        let maze = Maze::parse(include_str!("test1.txt"));
        assert_eq!(maze.tsp(), Some(86));
    }

    #[test]
    fn test2() {
        init();

        let maze = Maze::parse(include_str!("test2.txt"));
        assert_eq!(maze.tsp(), Some(132));
    }

    #[test]
    fn test3() {
        init();

        let maze = Maze::parse(include_str!("test3.txt"));
        assert_eq!(maze.tsp(), Some(136));
    }

    #[test]
    fn test4() {
        init();

        let maze = Maze::parse(include_str!("test4.txt"));
        assert_eq!(maze.tsp(), Some(81));
    }

    #[test]
    fn test_tsp4_0() {
        let maze = Maze::parse(include_str!("test5.txt"));
        assert_eq!(tsp4(&maze.split()), Some(8));
    }

    #[test]
    fn test_tsp4_1() {
        let maze = Maze::parse(include_str!("test6.txt"));
        assert_eq!(tsp4(&maze.split()), Some(24));
    }

    #[test]
    fn test_tsp4_2() {
        let maze = Maze::parse(include_str!("test7.txt"));
        assert_eq!(tsp4(&maze.split()), Some(32));
    }

    #[test]
    fn test_tsp4_3() {
        let maze = Maze::parse(include_str!("test8.txt"));
        assert_eq!(tsp4(&maze.split()), Some(72));
    }

    #[test]
    fn test_keyset() {
        let mut doors = KeySet::new();
        doors.insert_door(b'A');

        let mut keys = KeySet::new();
        assert_eq!(keys.can_open(&doors), false);

        keys.insert_key(b'a');
        assert_eq!(keys.can_open(&doors), true);

        keys.insert_key(b'b');
        assert_eq!(keys.can_open(&doors), true);

        doors.insert_door(b'C');
        assert_eq!(keys.can_open(&doors), false);
    }
}
