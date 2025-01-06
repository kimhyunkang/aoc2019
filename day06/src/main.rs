#![feature(slice_as_array)]

use std::collections::HashMap;

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let orbits = Orbits::load(&parse(&input));
    println!("1: {}", orbits.total_count());
    println!("2: {}", orbits.dist("YOU", "SAN").unwrap());
}

#[test]
fn test1() {
    let orbits = Orbits::load(&parse(include_str!("test1.txt")));
    assert_eq!(orbits.total_count(), 42);
}

#[test]
fn test2() {
    let orbits = Orbits::load(&parse(include_str!("test2.txt")));
    assert_eq!(orbits.dist("YOU", "SAN"), Some(4));
}

fn parse<'r>(input: &'r str) -> Vec<[&'r str; 2]> {
    input
        .lines()
        .map(|line| *line.split(')').collect::<Vec<_>>().as_array::<2>().unwrap())
        .collect()
}

struct Orbits<'r> {
    parents: HashMap<&'r str, &'r str>,
}

impl<'r> Orbits<'r> {
    fn load(orbits: &[[&'r str; 2]]) -> Self {
        let mut parents = HashMap::new();
        for &[p, c] in orbits {
            parents.insert(c, p);
        }
        Self { parents }
    }

    fn count(&self, cache: &mut HashMap<&'r str, usize>, key: &'r str) -> usize {
        if let Some(&c) = cache.get(&key) {
            return c;
        }
        if let Some(&parent) = self.parents.get(&key) {
            let c = self.count(cache, parent) + 1;
            cache.insert(key, c);
            c
        } else {
            0
        }
    }

    fn total_count(&self) -> usize {
        let mut cache = HashMap::new();
        self.parents
            .keys()
            .map(|&key| self.count(&mut cache, key))
            .sum::<usize>()
    }

    fn dist(&self, a: &'r str, b: &'r str) -> Option<usize> {
        let mut dist_map: HashMap<&'r str, usize> = HashMap::new();
        dist_map.insert(a, 0);
        let mut dist = 0;
        let mut node = a;
        while let Some(&p) = self.parents.get(node) {
            dist += 1;
            dist_map.insert(p, dist);
            node = p;
        }

        let mut dist = 0;
        let mut node = b;
        loop {
            if let Some(&a_dist) = dist_map.get(node) {
                return (a_dist + dist).checked_sub(2);
            } else if let Some(&p) = self.parents.get(node) {
                dist += 1;
                node = p;
            } else {
                return None;
            }
        }
    }
}
