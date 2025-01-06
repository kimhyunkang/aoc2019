use std::collections::{HashSet, VecDeque};

use bitvec::bitbox;
use vm::{VM, parse_program};

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
        self.buf.extend(self.vm.read_port());
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

mod vm {
    use std::{
        collections::VecDeque,
        fmt,
        task::{Poll, ready},
    };

    use log::debug;

    pub fn parse_program(input: &str) -> Vec<isize> {
        input
            .trim()
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect()
    }

    pub struct VM {
        mem: Vec<isize>,
        pc: usize,
        relative_base: usize,
        input: VecDeque<isize>,
        output: Vec<isize>,
    }

    enum VMError {
        Halt,
    }

    enum Mode {
        Position,
        Immediate,
        Relative,
    }

    impl fmt::Display for Mode {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Mode::Position => write!(f, "P"),
                Mode::Immediate => write!(f, "I"),
                Mode::Relative => write!(f, "R"),
            }
        }
    }

    impl VM {
        pub fn init(code: Vec<isize>) -> Self {
            VM {
                mem: code,
                pc: 0,
                relative_base: 0,
                input: VecDeque::new(),
                output: Vec::new(),
            }
        }

        fn mode(n: usize) -> Result<Mode, ()> {
            match n {
                0 => Ok(Mode::Position),
                1 => Ok(Mode::Immediate),
                2 => Ok(Mode::Relative),
                _ => Err(()),
            }
        }

        fn decode(op: isize) -> Result<(Mode, Mode, Mode, usize), ()> {
            let mut op: usize = op.try_into().map_err(|_| ())?;
            let opcode = op % 100;
            op /= 100;
            let a = op % 10;
            op /= 10;
            let b = op % 10;
            op /= 10;
            let c = op % 10;
            Ok((Self::mode(a)?, Self::mode(b)?, Self::mode(c)?, opcode))
        }

        pub fn write_port(&mut self, buf: &[isize]) {
            self.input.extend(buf);
        }

        pub fn read_port(&mut self) -> Vec<isize> {
            let mut output = Vec::new();
            std::mem::swap(&mut output, &mut self.output);
            output
        }

        fn step(&mut self) -> Poll<Result<(), VMError>> {
            let n = self.mem[self.pc];
            let (mode1, mode2, mode3, op) = if let Ok(dec) = Self::decode(n) {
                dec
            } else {
                panic!("Invalid opcode {} at addr {}", n, self.pc);
            };

            if op == 99 {
                return Poll::Ready(Err(VMError::Halt));
            }
            match op {
                1 | 2 | 7 | 8 => {
                    debug!("{:?}", &self.mem[self.pc..self.pc + 4]);
                    debug!("{} {}{}{}", op, mode1, mode2, mode3);
                    let x = self.read(mode1, 1);
                    let y = self.read(mode2, 2);
                    let v = match op {
                        1 => x + y,
                        2 => x * y,
                        7 => {
                            if x < y {
                                1
                            } else {
                                0
                            }
                        }
                        8 => {
                            if x == y {
                                1
                            } else {
                                0
                            }
                        }
                        _ => unreachable!(),
                    };
                    self.write(mode3, 3, v);
                    self.pc += 4;
                }
                3 => {
                    debug!("{:?}", &self.mem[self.pc..self.pc + 2]);
                    debug!("{} {}", op, mode1);
                    let v = if let Some(v) = self.input.pop_front() {
                        v
                    } else {
                        return Poll::Pending;
                    };
                    debug!("Read input: {}", v);
                    self.write(mode1, 1, v);
                    self.pc += 2;
                }
                4 => {
                    debug!("{:?}", &self.mem[self.pc..self.pc + 2]);
                    debug!("{} {}", op, mode1);
                    let v = self.read(mode1, 1);
                    debug!("Write output: {}", v);
                    self.output.push(v);
                    self.pc += 2;
                }
                5 | 6 => {
                    debug!("{:?}", &self.mem[self.pc..self.pc + 3]);
                    debug!("{} {}{}", op, mode1, mode2);
                    let x = self.read(mode1, 1);
                    let addr = self.read(mode2, 2);
                    let jump = match op {
                        5 => x != 0,
                        6 => x == 0,
                        _ => unreachable!(),
                    };
                    if jump {
                        debug!("Jump to {}", addr);
                        self.pc = addr.try_into().unwrap();
                    } else {
                        self.pc += 3;
                    }
                }
                9 => {
                    debug!("{:?}", &self.mem[self.pc..self.pc + 2]);
                    debug!("{} {}", op, mode1);
                    let offset = self.read(mode1, 1);
                    self.relative_base =
                        if let Some(base) = self.relative_base.checked_add_signed(offset) {
                            base
                        } else {
                            panic!(
                                "Cannot set relative base {}+{} at addr {}",
                                self.relative_base,
                                offset,
                                self.pc + 1
                            );
                        };
                    self.pc += 2;
                }
                _ => unimplemented!("Unknown instruction {}", op),
            }
            Poll::Ready(Ok(()))
        }

        pub fn run(&mut self) -> Poll<()> {
            while let Ok(()) = ready!(self.step()) {
                ()
            }
            Poll::Ready(())
        }

        pub fn run_ready(mut self) -> Vec<isize> {
            if let Poll::Ready(()) = self.run() {
                self.output
            } else {
                panic!("Program is pending");
            }
        }

        fn get_ptr(&self, mode: Mode, offset: usize) -> usize {
            let addr = self.pc + offset;
            let ptr = self.read_at(addr);
            match mode {
                Mode::Immediate => unreachable!(),
                Mode::Position => {
                    if let Ok(ptr) = ptr.try_into() {
                        ptr
                    } else {
                        panic!("Trying to access addr {} at {}", ptr, addr)
                    }
                }
                Mode::Relative => {
                    if let Some(ptr) = self.relative_base.checked_add_signed(ptr) {
                        ptr
                    } else {
                        panic!(
                            "Trying to access addr {}+{} at {}",
                            self.relative_base, ptr, addr
                        )
                    }
                }
            }
        }

        fn read(&self, mode: Mode, offset: usize) -> isize {
            if let Mode::Immediate = mode {
                let ptr = self.read_at(self.pc + offset);
                debug!("Imm {}", ptr);
                ptr
            } else {
                let ptr = self.get_ptr(mode, offset);
                let val = self.read_at(ptr);
                debug!("Read[{}]: {}", ptr, val);
                val
            }
        }

        fn read_at(&self, addr: usize) -> isize {
            self.mem.get(addr).copied().unwrap_or(0)
        }

        fn write(&mut self, mode: Mode, offset: usize, val: isize) {
            if let Mode::Immediate = mode {
                panic!(
                    "Immediate write not supported: opcode {} at {}",
                    self.read_at(self.pc),
                    self.pc
                );
            } else {
                let ptr = self.get_ptr(mode, offset);
                self.write_at(ptr, val);
            }
        }

        fn write_at(&mut self, addr: usize, val: isize) {
            debug!("Write[{}]={}", addr, val);
            if self.mem.len() <= addr {
                self.mem.resize(addr + 1, 0);
            }
            self.mem[addr] = val;
        }
    }

    #[cfg(test)]
    mod test {
        use super::VM;

        #[test]
        fn test_cmp() {
            // Using position mode, consider whether the input is equal to 8; output 1 (if it is) or 0 (if it is not).
            assert_eq!(
                test_run(vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8], &[8]),
                vec![1]
            );
            assert_eq!(
                test_run(vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8], &[7]),
                vec![0]
            );

            // Using position mode, consider whether the input is less than 8; output 1 (if it is) or 0 (if it is not).
            assert_eq!(
                test_run(vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8], &[8]),
                vec![0]
            );
            assert_eq!(
                test_run(vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8], &[7]),
                vec![1]
            );

            // Using immediate mode, consider whether the input is equal to 8; output 1 (if it is) or 0 (if it is not).
            assert_eq!(test_run(vec![3, 3, 1108, -1, 8, 3, 4, 3, 99], &[8]), vec![
                1
            ]);
            assert_eq!(test_run(vec![3, 3, 1108, -1, 8, 3, 4, 3, 99], &[7]), vec![
                0
            ]);

            // Using immediate mode, consider whether the input is less than 8; output 1 (if it is) or 0 (if it is not).
            assert_eq!(test_run(vec![3, 3, 1107, -1, 8, 3, 4, 3, 99], &[8]), vec![
                0
            ]);
            assert_eq!(test_run(vec![3, 3, 1107, -1, 8, 3, 4, 3, 99], &[7]), vec![
                1
            ]);
        }

        // Here are some jump tests that take an input, then output 0 if the input was zero or 1 if the input was non-zero.
        #[test]
        fn test_jmp() {
            // using position mode
            assert_eq!(
                test_run(
                    vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9],
                    &[0]
                ),
                vec![0]
            );
            assert_eq!(
                test_run(
                    vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9],
                    &[2]
                ),
                vec![1]
            );

            // using immediate mode
            assert_eq!(
                test_run(vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1], &[0]),
                vec![0]
            );
            assert_eq!(
                test_run(vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1], &[2]),
                vec![1]
            );
        }

        #[test]
        fn test_relative() {
            let program = vec![
                109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
            ];
            assert_eq!(test_run(program.clone(), &[]), program);

            let program = vec![1102, 34915192, 34915192, 7, 4, 7, 99, 0];
            let out = test_run(program, &[]);
            assert_eq!(out[0].to_string().len(), 16);

            let n = 1125899906842624;
            let program = vec![104, n, 99];
            assert_eq!(test_run(program, &[]), vec![n]);
        }

        #[test]
        fn test() {
            // The example program uses an input instruction to ask for a single number. The program will then output 999 if the input value is below 8, output 1000 if the input value is equal to 8, or output 1001 if the input value is greater than 8.
            let program = vec![
                3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36,
                98, 0, 0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000,
                1, 20, 4, 20, 1105, 1, 46, 98, 99,
            ];

            assert_eq!(test_run(program.clone(), &[7]), vec![999]);
            assert_eq!(test_run(program.clone(), &[8]), vec![1000]);
            assert_eq!(test_run(program.clone(), &[9]), vec![1001]);
        }

        #[cfg(test)]
        fn test_run(code: Vec<isize>, input: &[isize]) -> Vec<isize> {
            let mut vm = VM::init(code);
            vm.write_port(input);
            vm.run_ready()
        }
    }
}
