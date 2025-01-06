use std::{
    collections::VecDeque,
    fmt,
    task::{Poll, ready},
};

use log::debug;

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let program = input
        .trim()
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect::<Vec<_>>();
    println!("1: {}", find_amp(program.clone(), [0, 1, 2, 3, 4], false));
    println!("2: {}", find_amp(program, [5, 6, 7, 8, 9], true));
}

#[test]
fn test1() {
    let program = vec![
        3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
    ];
    assert_eq!(find_amp(program, [0, 1, 2, 3, 4], false), 43210);

    let program = vec![
        3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23, 99,
        0, 0,
    ];
    assert_eq!(find_amp(program, [0, 1, 2, 3, 4], false), 54321);

    let program = vec![
        3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1, 33,
        31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
    ];
    assert_eq!(find_amp(program, [0, 1, 2, 3, 4], false), 65210);
}

#[test]
fn test2() {
    let program = vec![
        3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26, 27, 4, 27, 1001, 28, -1, 28,
        1005, 28, 6, 99, 0, 0, 5,
    ];
    assert_eq!(find_amp(program, [5, 6, 7, 8, 9], true), 139629729);

    let program = vec![
        3, 52, 1001, 52, -5, 52, 3, 53, 1, 52, 56, 54, 1007, 54, 5, 55, 1005, 55, 26, 1001, 54, -5,
        54, 1105, 1, 12, 1, 53, 54, 53, 1008, 54, 0, 55, 1001, 55, 1, 55, 2, 53, 55, 53, 4, 53,
        1001, 56, -1, 56, 1005, 56, 6, 99, 0, 0, 0, 0, 10,
    ];
    assert_eq!(find_amp(program, [5, 6, 7, 8, 9], true), 18216);
}

fn find_amp(program: Vec<isize>, phase: [isize; 5], feedback: bool) -> isize {
    let mut max = isize::MIN;
    for phase in permutation(phase) {
        let mut circuit = Circuit::load(program.clone(), phase, feedback);
        let signal = circuit.run().unwrap();
        if max < signal {
            max = signal;
        }
    }
    max
}

struct Permutation<T, const N: usize> {
    a: [T; N],
    c: [usize; N],
    i: usize,
    start: bool,
}

impl<T: Copy + fmt::Debug, const N: usize> Iterator for Permutation<T, N> {
    type Item = [T; N];

    fn next(&mut self) -> Option<[T; N]> {
        if self.start {
            self.start = false;
            return Some(self.a);
        }
        while self.i < N {
            if self.c[self.i] < self.i {
                if self.i % 2 == 0 {
                    self.a.swap(0, self.i);
                } else {
                    self.a.swap(self.c[self.i], self.i);
                }
                self.c[self.i] += 1;
                self.i = 1;
                return Some(self.a);
            } else {
                self.c[self.i] = 0;
                self.i += 1;
            }
        }
        None
    }
}

fn permutation<T: Copy, const N: usize>(a: [T; N]) -> Permutation<T, N> {
    Permutation {
        a,
        c: [0; N],
        i: 1,
        start: true,
    }
}

#[test]
fn test_permutation() {
    use std::collections::HashSet;

    let perms = permutation([0, 1, 2, 3, 4]).collect::<HashSet<_>>();
    assert_eq!(perms.len(), 5 * 4 * 3 * 2 * 1);
    for mut p in perms {
        p.sort();
        assert_eq!(p, [0, 1, 2, 3, 4]);
    }
}

struct Circuit {
    vms: [(VM, State); 5],
    feedback: bool,
    last_out: Option<isize>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Ready,
    Pending,
    Halt,
}

impl Circuit {
    fn load(code: Vec<isize>, phase: [isize; 5], feedback: bool) -> Self {
        let mut vms =
            core::array::from_fn(|i| (VM::init(code.clone(), vec![phase[i]]), State::Ready));
        vms[0].0.write_port(&[0]);
        Circuit {
            vms,
            feedback,
            last_out: None,
        }
    }

    fn poll_run(&mut self) -> Poll<()> {
        while let Some((vm, st)) = self.vms.iter_mut().find(|(_, st)| *st == State::Ready) {
            if vm.run().is_ready() {
                *st = State::Halt;
            } else {
                *st = State::Pending
            }
        }
        if self.vms.iter().all(|(_, st)| *st == State::Halt) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    fn pipe(&mut self) {
        for idx in 0..4 {
            let out0 = self.vms[idx].0.output.split_off(0);
            if out0.len() > 0 {
                let (vm1, st1) = &mut self.vms[idx + 1];
                vm1.write_port(&out0);
                *st1 = State::Ready;
            }
        }
        let mut out = self.vms[4].0.output.split_off(0);
        if self.feedback {
            if out.len() > 0 {
                let (vm0, st0) = &mut self.vms[0];
                vm0.write_port(&out);
                self.last_out = out.last().copied();
                *st0 = State::Ready;
            }
        } else {
            if let Some(signal) = out.pop() {
                self.last_out = Some(signal);
            }
        }
    }

    fn run(&mut self) -> Option<isize> {
        while !self.poll_run().is_ready() {
            self.pipe();
        }
        self.pipe();
        self.last_out
    }
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

    fn write_port(&mut self, buf: &[isize]) {
        self.input.extend(buf);
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
                    return Poll::Pending;
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
        Poll::Ready(Ok(()))
    }

    fn run(&mut self) -> Poll<()> {
        while let Ok(()) = ready!(self.step()) {
            ()
        }
        Poll::Ready(())
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

#[cfg(test)]
mod test {
    use super::VM;

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
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ];

        assert_eq!(test_run(program.clone(), vec![7]), vec![999]);
        assert_eq!(test_run(program.clone(), vec![8]), vec![1000]);
        assert_eq!(test_run(program.clone(), vec![9]), vec![1001]);
    }

    #[cfg(test)]
    fn test_run(code: Vec<isize>, input: Vec<isize>) -> Vec<isize> {
        let mut vm = VM::init(code, input);
        assert!(vm.run().is_ready());
        vm.output
    }
}
