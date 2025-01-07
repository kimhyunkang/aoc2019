use std::{cmp::Ordering, collections::HashMap};

use once_cell::sync::Lazy;
use regex::Regex;

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let formula = Formula::parse(&input);
    println!("1: {}", formula.produce("FUEL", 1, "ORE").unwrap());
    let n = 1000000000000;
    println!("2: {}", binary_search(&formula, n));
}

#[test]
fn test1() {
    let formula = Formula::parse(include_str!("test1.txt"));
    assert_eq!(formula.produce("FUEL", 1, "ORE"), Some(31));

    let formula = Formula::parse(include_str!("test2.txt"));
    assert_eq!(formula.produce("FUEL", 1, "ORE"), Some(165));

    let formula = Formula::parse(include_str!("test3.txt"));
    assert_eq!(formula.produce("FUEL", 1, "ORE"), Some(13312));

    let formula = Formula::parse(include_str!("test4.txt"));
    assert_eq!(formula.produce("FUEL", 1, "ORE"), Some(180697));

    let formula = Formula::parse(include_str!("test5.txt"));
    assert_eq!(formula.produce("FUEL", 1, "ORE"), Some(2210736));
}

#[test]
fn test2() {
    let n = 1000000000000;

    let formula = Formula::parse(include_str!("test3.txt"));
    assert_eq!(binary_search(&formula, n), 82892753);

    let formula = Formula::parse(include_str!("test4.txt"));
    assert_eq!(binary_search(&formula, n), 5586022);

    let formula = Formula::parse(include_str!("test5.txt"));
    assert_eq!(binary_search(&formula, n), 460664);
}

fn binary_search<'r>(formula: &Formula<'r>, target: isize) -> isize {
    let mut lo = 0;
    let mut hi = 1;
    loop {
        let ore = formula.produce("FUEL", hi, "ORE").unwrap();
        match ore.cmp(&target) {
            Ordering::Less => {
                lo = hi;
                hi *= 2;
            }
            Ordering::Equal => {
                return hi;
            }
            Ordering::Greater => {
                break;
            }
        }
    }

    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        let ore = formula.produce("FUEL", mid, "ORE").unwrap();
        match ore.cmp(&target) {
            Ordering::Less => {
                lo = mid;
            }
            Ordering::Equal => {
                return mid;
            }
            Ordering::Greater => {
                hi = mid;
            }
        }
    }
    lo
}

struct Formula<'r> {
    formulas: HashMap<&'r str, (isize, Vec<(&'r str, isize)>)>,
}

fn find_requirement<'r>(
    requirements: &mut HashMap<&'r str, isize>,
    except: &'r str,
) -> Option<(&'r str, isize)> {
    if let Some((k, _)) = requirements.iter().find(|&(k, v)| *k != except && *v > 0) {
        requirements.remove_entry(*k)
    } else {
        None
    }
}

impl<'r> Formula<'r> {
    fn parse_line(input: &'r str) -> (&'r str, (isize, Vec<(&'r str, isize)>)) {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"([0-9]+) ([A-Z]+)").unwrap());
        let mut iter = input.split("=>");
        let inputs = iter.next().unwrap();
        let inputs = PATTERN
            .captures_iter(inputs)
            .map(|cap| {
                let (_, [n, name]) = cap.extract::<2>();
                (name, n.parse().unwrap())
            })
            .collect::<Vec<_>>();
        let output = iter.next().unwrap();
        let (_, [n, name]) = PATTERN.captures(output).unwrap().extract::<2>();
        (name, (n.parse().unwrap(), inputs))
    }

    fn parse(input: &'r str) -> Self {
        Self {
            formulas: input.lines().map(Self::parse_line).collect(),
        }
    }

    fn requirement(
        &self,
        chemical: &'r str,
        required: isize,
    ) -> Option<(isize, Vec<(&'r str, isize)>)> {
        let (quantity, mut requirements) = self.formulas.get(&chemical)?.clone();
        let factor = (required + quantity - 1) / quantity;
        for (_, q) in &mut requirements {
            *q *= factor;
        }
        Some((quantity * factor, requirements))
    }

    fn produce(&self, target: &str, target_quantity: isize, base: &str) -> Option<isize> {
        let mut targets = HashMap::new();
        targets.insert(target, target_quantity);

        while let Some((chemical, required)) = find_requirement(&mut targets, base) {
            let (produced, requirements) = self.requirement(chemical, required)?;
            targets.insert(chemical, required - produced);
            for (c, n) in requirements {
                let e = targets.entry(c).or_insert(0);
                *e += n;
            }
        }

        targets.get(&base).copied()
    }
}
