use std::{
    fmt,
    iter::Sum,
    ops::{Add, AddAssign, Sub},
};

use num_integer::gcd;
use once_cell::sync::Lazy;
use regex::Regex;

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let mut system = System::parse(&input);
    for _ in 0..1000 {
        system.step();
    }
    println!("1: {}", system.energy());
    println!("2: {}", system.cycle());
}

#[test]
fn test_energy() {
    let mut system = System::parse(include_str!("test1.txt"));
    for _ in 0..10 {
        system.step();
    }
    assert_eq!(system.energy(), 179);

    let mut system = System::parse(include_str!("test2.txt"));
    for _ in 0..100 {
        system.step();
    }
    assert_eq!(system.energy(), 1940);
}

#[test]
fn test2() {
    let system = System::parse(include_str!("test2.txt"));
    assert_eq!(system.cycle(), 4686774924);
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Vec3 {
    v: [isize; 3],
}

impl fmt::Debug for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<x={}, y={}, z={}>", self.v[0], self.v[1], self.v[2])
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self {
        Self {
            v: [
                self.v[0] + rhs.v[0],
                self.v[1] + rhs.v[1],
                self.v[2] + rhs.v[2],
            ],
        }
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self {
        Self {
            v: [
                self.v[0] - rhs.v[0],
                self.v[1] - rhs.v[1],
                self.v[2] - rhs.v[2],
            ],
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.v[0] += rhs.v[0];
        self.v[1] += rhs.v[1];
        self.v[2] += rhs.v[2];
    }
}

impl Sum for Vec3 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut s = Self::zero();
        for v in iter {
            s += v;
        }
        s
    }
}

impl Vec3 {
    fn zero() -> Self {
        Self { v: [0, 0, 0] }
    }

    fn norm(&self) -> usize {
        self.v.iter().map(|x| x.unsigned_abs()).sum::<usize>()
    }

    fn signum(&self) -> Self {
        Self {
            v: [self.v[0].signum(), self.v[1].signum(), self.v[2].signum()],
        }
    }

    fn parse_line(line: &str) -> Self {
        static PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"<x=(-?[0-9]+), y=(-?[0-9]+), z=(-?[0-9]+)>").unwrap());
        let caps = PATTERN.captures(line).unwrap();
        let (_, xyz) = caps.extract::<3>();
        Self {
            v: core::array::from_fn(|i| xyz[i].parse().unwrap()),
        }
    }
}

struct Moon {
    pos: Vec3,
    vel: Vec3,
}

impl Moon {
    fn gravity(&self, moons: &[Moon]) -> Vec3 {
        moons
            .iter()
            .map(|moon| (moon.pos - self.pos).signum())
            .sum::<Vec3>()
    }

    fn energy(&self) -> usize {
        self.pos.norm() * self.vel.norm()
    }
}

struct System {
    moons: Vec<Moon>,
}

impl fmt::Debug for System {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for moon in &self.moons {
            writeln!(f, "pos={:?}, vel={:?}", moon.pos, moon.vel)?;
        }
        Ok(())
    }
}

impl System {
    fn parse(input: &str) -> Self {
        let moons = input
            .lines()
            .map(|line| Moon {
                pos: Vec3::parse_line(line),
                vel: Vec3::zero(),
            })
            .collect();
        Self { moons }
    }

    fn step(&mut self) {
        let gravity = self
            .moons
            .iter()
            .map(|moon| moon.gravity(&self.moons))
            .collect::<Vec<_>>();
        for (moon, acc) in self.moons.iter_mut().zip(gravity) {
            moon.vel += acc;
        }
        for moon in &mut self.moons {
            moon.pos += moon.vel;
        }
    }

    fn energy(&self) -> usize {
        self.moons.iter().map(|moon| moon.energy()).sum::<usize>()
    }

    fn axis_cycle(&self, axis: usize) -> usize {
        let (pos0, vel0): (Vec<isize>, Vec<isize>) = self
            .moons
            .iter()
            .map(|moon| (moon.pos.v[axis], moon.vel.v[axis]))
            .unzip();
        let mut pos = pos0.clone();
        let mut vel = vel0.clone();
        let mut n = 0;
        loop {
            let acc = pos
                .iter()
                .map(|&p0| pos.iter().map(|&p1| (p1 - p0).signum()).sum::<isize>())
                .collect::<Vec<_>>();
            for i in 0..self.moons.len() {
                vel[i] += acc[i];
                pos[i] += vel[i];
            }
            n += 1;
            if pos == pos0 && vel == vel0 {
                break;
            }
        }
        n
    }

    fn cycle(&self) -> usize {
        let axis_cycle = (0..3)
            .into_iter()
            .map(|axis| self.axis_cycle(axis))
            .collect::<Vec<_>>();
        let mut cycle = axis_cycle[0];
        cycle = cycle / gcd(cycle, axis_cycle[1]) * axis_cycle[1];
        cycle = cycle / gcd(cycle, axis_cycle[2]) * axis_cycle[2];
        cycle
    }
}

#[test]
fn test_parse() {
    assert_eq!(Vec3::parse_line("<x=-1, y=0, z=2>"), Vec3 { v: [-1, 0, 2] });
}
