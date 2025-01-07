use intcode::VM;
use std::{fmt, task::Poll};

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
        let mut vms = core::array::from_fn(|_| (VM::init(code.clone()), State::Ready));
        for i in 0..5 {
            vms[i].0.write_port(&[phase[i]]);
        }
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
            let out0 = self.vms[idx].0.read_all();
            if out0.len() > 0 {
                let (vm1, st1) = &mut self.vms[idx + 1];
                vm1.write_port(&out0);
                *st1 = State::Ready;
            }
        }
        let mut out = self.vms[4].0.read_all();
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
