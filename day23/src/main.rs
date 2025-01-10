use std::collections::VecDeque;

use bitvec::{bitbox, boxed::BitBox};
use intcode::{VM, parse_program};

fn main() {
    env_logger::init();

    let input = std::fs::read_to_string("input.txt").unwrap();
    let program = parse_program(&input);

    let mut os = OS::boot(program.clone(), 50);
    let nat = os.run_until_nat().unwrap();
    println!("1: {}", nat[1]);

    let mut os = OS::boot(program, 50);
    println!("2: {}", os.run());
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Ready,
    Sleep,
    Halt,
}

#[derive(Clone)]
struct Process {
    vm: VM,
    st: State,
    queue: VecDeque<[isize; 2]>,
}

struct Scheduler {
    ready: BitBox,
}

impl Scheduler {
    fn init(n: usize) -> Self {
        Self {
            ready: bitbox![1; n],
        }
    }

    fn next(&mut self) -> Option<usize> {
        let pid = self.ready.iter_ones().next()?;
        self.ready.set(pid, false);
        Some(pid)
    }

    fn wake(&mut self, pid: usize) {
        self.ready.set(pid, true);
    }
}

struct Packet {
    pid: usize,
    data: [isize; 2],
}

struct OS {
    process: Vec<Process>,
    scheduler: Scheduler,
    queue: VecDeque<Packet>,
}

impl OS {
    fn boot(program: Vec<isize>, n: usize) -> Self {
        let mut process = vec![
            Process {
                vm: VM::init(program),
                st: State::Ready,
                queue: VecDeque::new()
            };
            n
        ];
        for (pid, proc) in process.iter_mut().enumerate() {
            // Init network addr
            proc.vm.write_port(&[pid as isize]);
        }
        Self {
            process,
            scheduler: Scheduler::init(n),
            queue: VecDeque::new(),
        }
    }

    fn run_until_nat(&mut self) -> Option<[isize; 2]> {
        while let Some(pid) = self.scheduler.next() {
            self.run_proc(pid);
            while let Some(packet) = self.queue.pop_front() {
                if packet.pid == 255 {
                    return Some(packet.data);
                }
                self.write_packet(packet);
            }
        }

        None
    }

    fn run(&mut self) -> isize {
        let mut nat = None;
        let mut nat_history = Vec::with_capacity(2);

        loop {
            while let Some(pid) = self.scheduler.next() {
                self.run_proc(pid);
                while let Some(packet) = self.queue.pop_front() {
                    if packet.pid == 255 {
                        log::info!("RECV: {:?}", packet.data);
                        nat = Some(packet.data);
                    } else {
                        self.write_packet(packet);
                    }
                }
            }

            assert!(self.idle());

            if let Some(data) = nat {
                log::info!("SEND: {:?}", data);
                if nat_history.len() >= 2 {
                    nat_history.remove(0);
                }
                nat_history.push(data[1]);
                match &nat_history[..] {
                    &[y0, y1] if y0 == y1 => {
                        return y0;
                    }
                    _ => (),
                }

                self.write_packet(Packet { pid: 0, data });
            } else {
                panic!("NAT packet empty");
            }
        }
    }

    fn idle(&self) -> bool {
        self.queue.is_empty() && self.process.iter().all(|proc| proc.queue.is_empty())
    }

    fn write_packet(&mut self, packet: Packet) {
        let proc = &mut self.process[packet.pid];
        proc.queue.push_back(packet.data);
        if proc.st != State::Halt {
            proc.st = State::Ready;
            self.scheduler.wake(packet.pid);
        }
    }

    fn run_proc(&mut self, pid: usize) {
        let proc = &mut self.process[pid];
        if proc.st == State::Halt {
            return;
        }

        if proc.queue.is_empty() {
            proc.vm.write_port(&[-1]);
        } else {
            while let Some(data) = proc.queue.pop_front() {
                proc.vm.write_port(&data[..]);
            }
        }

        if proc.vm.run().is_ready() {
            proc.st = State::Halt;
        } else {
            proc.st = State::Sleep;
        }
        let mut buf = [0; 3];
        while proc.vm.read_exact(&mut buf[..]).is_ready() {
            let [pid, x, y] = buf;
            let pid = pid.try_into().unwrap();
            self.queue.push_back(Packet { pid, data: [x, y] });
        }
    }
}
