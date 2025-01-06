use core::fmt;
use std::collections::VecDeque;

use log::debug;

fn main() {
    env_logger::init();

    let input = std::fs::read_to_string("input.txt").unwrap();
    let code: Vec<isize> = input
        .trim()
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();

    let mut vm = VM::init(code.clone(), vec![1]);
    vm.run();
    let diag = vm.output.pop().unwrap();
    assert!(vm.output.into_iter().all(|n| n == 0));
    println!("1: {:?}", diag);

    let mut vm = VM::init(code.clone(), vec![5]);
    vm.run();
    assert_eq!(vm.output.len(), 1);
    println!("2: {:?}", vm.output[0]);
}

struct VM {
    mem: Vec<isize>,
    pc: usize,
    input: VecDeque<isize>,
    output: Vec<isize>,
}

enum VMError {
    Halt,
}

enum Mode {
    Position,
    Immediate,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mode::Position => write!(f, "P"),
            Mode::Immediate => write!(f, "I"),
        }
    }
}

impl VM {
    fn init(code: Vec<isize>, input: Vec<isize>) -> Self {
        VM {
            mem: code,
            pc: 0,
            input: input.into(),
            output: Vec::new(),
        }
    }

    fn mode(n: usize) -> Result<Mode, ()> {
        match n {
            0 => Ok(Mode::Position),
            1 => Ok(Mode::Immediate),
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

    fn step(&mut self) -> Result<(), VMError> {
        let n = self.mem[self.pc];
        let (mode1, mode2, mode3, op) = if let Ok(dec) = Self::decode(n) {
            dec
        } else {
            panic!("Invalid opcode {} at addr {}", n, self.pc);
        };

        if op == 99 {
            return Err(VMError::Halt);
        }
        match op {
            1 | 2 | 7 | 8 => {
                debug!("{:?}", &self.mem[self.pc..self.pc + 4]);
                debug!("{} {}{}{}", op, mode1, mode2, mode3);
                let x = self.read(mode1, self.pc + 1);
                let y = self.read(mode2, self.pc + 2);
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
                self.write(mode3, self.pc + 3, v);
                self.pc += 4;
            }
            3 => {
                debug!("{:?}", &self.mem[self.pc..self.pc + 2]);
                debug!("{} {}", op, mode1);
                let v = if let Some(v) = self.input.pop_front() {
                    v
                } else {
                    panic!("End of input")
                };
                debug!("Read input: {}", v);
                self.write(mode1, self.pc + 1, v);
                self.pc += 2;
            }
            4 => {
                debug!("{:?}", &self.mem[self.pc..self.pc + 2]);
                debug!("{} {}", op, mode1);
                let v = self.read(mode1, self.pc + 1);
                debug!("Write output: {}", v);
                self.output.push(v);
                self.pc += 2;
            }
            5 | 6 => {
                debug!("{:?}", &self.mem[self.pc..self.pc + 3]);
                debug!("{} {}{}", op, mode1, mode2);
                let x = self.read(mode1, self.pc + 1);
                let addr = self.read(mode2, self.pc + 2);
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
            _ => unimplemented!("Unknown instruction {}", op),
        }
        Ok(())
    }

    fn run(&mut self) {
        while let Ok(()) = self.step() {
            ()
        }
    }

    fn read(&self, mode: Mode, addr: usize) -> isize {
        let ptr = self.read_at(addr);
        match mode {
            Mode::Immediate => {
                debug!("Imm {}", ptr);
                ptr
            }
            Mode::Position => {
                if let Ok(ptr) = ptr.try_into() {
                    let val = self.read_at(ptr);
                    debug!("Read[{}]: {}", ptr, val);
                    val
                } else {
                    panic!("Trying to read from addr {} at {}", ptr, addr)
                }
            }
        }
    }

    fn read_at(&self, addr: usize) -> isize {
        self.mem.get(addr).copied().unwrap_or(0)
    }

    fn write(&mut self, mode: Mode, addr: usize, val: isize) {
        let ptr = self.read_at(addr);
        match mode {
            Mode::Immediate => panic!(
                "Immediate write not supported: opcode {} at {}",
                self.read_at(self.pc),
                self.pc
            ),
            Mode::Position => {
                if let Ok(ptr) = ptr.try_into() {
                    self.write_at(ptr, val)
                } else {
                    panic!("Trying to write to addr {} at {}", ptr, addr)
                }
            }
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

#[test]
fn test_cmp() {
    // Using position mode, consider whether the input is equal to 8; output 1 (if it is) or 0 (if it is not).
    assert_eq!(
        test_run(vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8], vec![8]),
        vec![1]
    );
    assert_eq!(
        test_run(vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8], vec![7]),
        vec![0]
    );

    // Using position mode, consider whether the input is less than 8; output 1 (if it is) or 0 (if it is not).
    assert_eq!(
        test_run(vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8], vec![8]),
        vec![0]
    );
    assert_eq!(
        test_run(vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8], vec![7]),
        vec![1]
    );

    // Using immediate mode, consider whether the input is equal to 8; output 1 (if it is) or 0 (if it is not).
    assert_eq!(
        test_run(vec![3, 3, 1108, -1, 8, 3, 4, 3, 99], vec![8]),
        vec![1]
    );
    assert_eq!(
        test_run(vec![3, 3, 1108, -1, 8, 3, 4, 3, 99], vec![7]),
        vec![0]
    );

    // Using immediate mode, consider whether the input is less than 8; output 1 (if it is) or 0 (if it is not).
    assert_eq!(
        test_run(vec![3, 3, 1107, -1, 8, 3, 4, 3, 99], vec![8]),
        vec![0]
    );
    assert_eq!(
        test_run(vec![3, 3, 1107, -1, 8, 3, 4, 3, 99], vec![7]),
        vec![1]
    );
}

// Here are some jump tests that take an input, then output 0 if the input was zero or 1 if the input was non-zero.
#[test]
fn test_jmp() {
    // using position mode
    assert_eq!(
        test_run(
            vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9],
            vec![0]
        ),
        vec![0]
    );
    assert_eq!(
        test_run(
            vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9],
            vec![2]
        ),
        vec![1]
    );

    // using immediate mode
    assert_eq!(
        test_run(vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1], vec![
            0
        ]),
        vec![0]
    );
    assert_eq!(
        test_run(vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1], vec![
            2
        ]),
        vec![1]
    );
}

#[test]
fn test() {
    // The example program uses an input instruction to ask for a single number. The program will then output 999 if the input value is below 8, output 1000 if the input value is equal to 8, or output 1001 if the input value is greater than 8.
    let program = vec![
        3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0, 0,
        1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4, 20,
        1105, 1, 46, 98, 99,
    ];

    assert_eq!(test_run(program.clone(), vec![7]), vec![999]);
    assert_eq!(test_run(program.clone(), vec![8]), vec![1000]);
    assert_eq!(test_run(program.clone(), vec![9]), vec![1001]);
}

#[cfg(test)]
fn test_run(code: Vec<isize>, input: Vec<isize>) -> Vec<isize> {
    let mut vm = VM::init(code, input);
    vm.run();
    vm.output
}
