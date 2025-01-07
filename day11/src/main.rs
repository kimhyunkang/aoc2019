use std::collections::{HashSet, VecDeque};

use bitvec::bitbox;
use intcode::{VM, parse_program};

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let program = parse_program(&input);
    let mut robot = Robot::init1(program.clone());
    robot.run();
    println!("1: {}", robot.trail.unwrap().len());
    let mut robot = Robot::init2(program);
    robot.run();
    println!("2:");
    robot.render();
}

struct Robot {
    vm: VM,
    white: HashSet<(isize, isize)>,
    trail: Option<HashSet<(isize, isize)>>,
    buf: VecDeque<isize>,
    dir: (isize, isize),
    pos: (isize, isize),
}

impl Robot {
    fn init1(program: Vec<isize>) -> Self {
        Robot {
            vm: VM::init(program),
            white: HashSet::new(),
            trail: Some(HashSet::new()),
            buf: VecDeque::new(),
            dir: (0, 1),
            pos: (0, 0),
        }
    }

    fn init2(program: Vec<isize>) -> Self {
        let mut white = HashSet::new();
        white.insert((0, 0));
        Robot {
            vm: VM::init(program),
            white,
            trail: None,
            buf: VecDeque::new(),
            dir: (0, 1),
            pos: (0, 0),
        }
    }

    fn run(&mut self) {
        while !self.vm.run().is_ready() {
            self.process_output();
            let white = if self.white.contains(&self.pos) { 1 } else { 0 };
            self.vm.write_port(&[white]);
        }
    }

    fn process_output(&mut self) {
        self.buf.extend(self.vm.read_all());
        while 2 <= self.buf.len() {
            match self.buf.pop_front().unwrap() {
                0 => {
                    self.white.remove(&self.pos);
                }
                1 => {
                    self.white.insert(self.pos);
                }
                c => panic!("Invalid color {}", c),
            }
            if let Some(trail) = self.trail.as_mut() {
                trail.insert(self.pos);
            }
            let (dx, dy) = self.dir;
            self.dir = match self.buf.pop_front().unwrap() {
                0 => (-dy, dx),
                1 => (dy, -dx),
                d => panic!("Invalid direction {}", d),
            };
            self.move_forward();
        }
    }

    fn move_forward(&mut self) {
        let (dx, dy) = self.dir;
        let (x, y) = self.pos;
        self.pos = (x + dx, y + dy);
    }

    fn render(&self) {
        let (mut min_x, mut max_x) = (isize::MAX, isize::MIN);
        let (mut min_y, mut max_y) = (isize::MAX, isize::MIN);
        for &(x, y) in &self.white {
            if x < min_x {
                min_x = x;
            }
            if max_x < x {
                max_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if max_y < y {
                max_y = y;
            }
        }
        let h = max_y.abs_diff(min_y) + 1;
        let w = max_x.abs_diff(min_x) + 1;
        let mut grid = vec![bitbox![0; w]; h];
        for &(x, y) in &self.white {
            let c = x.abs_diff(min_x);
            let r = y.abs_diff(max_y);
            grid[r].set(c, true);
        }
        for r in 0..h {
            for c in 0..w {
                if grid[r][c] {
                    print!("#");
                } else {
                    print!(".");
                }
            }
            println!();
        }
    }
}
