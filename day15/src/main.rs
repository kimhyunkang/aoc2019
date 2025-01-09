use std::collections::{HashMap, VecDeque};

use intcode::{VM, parse_program};
use log::info;

fn main() {
    env_logger::init();

    let input = std::fs::read_to_string("input.txt").unwrap();
    let program = parse_program(&input);
    let (vm, dist) = find_oxygen(program).unwrap();
    println!("1: {}", dist);
    let max_dist = fill_oxygen(vm);
    println!("2: {}", max_dist);
}

//      x
//   +----->
//   |
// y |
//   |
//   v
fn find_oxygen(program: Vec<isize>) -> Option<(VM, usize)> {
    struct Entry {
        dist: usize,
        vm: VM,
    }

    // 1:N, 2:S, 3:W, 4:E
    static DIR: [(isize, isize); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

    let vm = VM::init(program);
    let mut dist_map = HashMap::new();
    dist_map.insert((0, 0), Entry { dist: 0, vm });
    let mut queue = VecDeque::new();
    queue.push_back((0, 0));

    while let Some(p0) = queue.pop_front() {
        let dist0 = dist_map[&p0].dist;
        let (x0, y0) = p0;
        for (dir, &(dx, dy)) in DIR.iter().enumerate() {
            let p1 = (x0 + dx, y0 + dy);
            info!("dir: {}, p1: {:?}", dir, p1);
            let dist1 = dist_map
                .get(&p1)
                .map(|entry| entry.dist)
                .unwrap_or(usize::MAX);
            if dist0 + 1 < dist1 {
                let mut vm = dist_map[&p0].vm.clone();
                vm.write_port(&[(dir + 1) as isize]);
                if vm.run().is_ready() {
                    panic!("VM halted");
                }
                match vm.read_port() {
                    Some(0) => {
                        info!("Wall");
                        // Wall
                        continue;
                    }
                    Some(1) => {
                        info!("Success");
                        // Move successful
                        dist_map.insert(p1, Entry {
                            dist: dist0 + 1,
                            vm,
                        });
                        queue.push_back(p1);
                    }
                    Some(2) => {
                        info!("Found");
                        // Found
                        return Some((vm, dist0 + 1));
                    }
                    Some(n) => {
                        panic!("VM returned {}", n);
                    }
                    None => {
                        panic!("VM did not return output");
                    }
                }
            }
        }
    }
    None
}

fn fill_oxygen(vm: VM) -> usize {
    struct Entry {
        dist: usize,
        vm: VM,
    }

    // 1:N, 2:S, 3:W, 4:E
    static DIR: [(isize, isize); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

    let mut dist_map = HashMap::new();
    dist_map.insert((0, 0), Entry { dist: 0, vm });
    let mut queue = VecDeque::new();
    queue.push_back((0, 0));

    let mut max_dist = 0;
    while let Some(p0) = queue.pop_front() {
        let dist0 = dist_map[&p0].dist;
        let (x0, y0) = p0;
        for (dir, &(dx, dy)) in DIR.iter().enumerate() {
            let p1 = (x0 + dx, y0 + dy);
            info!("dir: {}, p1: {:?}", dir, p1);
            let dist1 = dist_map
                .get(&p1)
                .map(|entry| entry.dist)
                .unwrap_or(usize::MAX);
            if dist0 + 1 < dist1 {
                let mut vm = dist_map[&p0].vm.clone();
                vm.write_port(&[(dir + 1) as isize]);
                if vm.run().is_ready() {
                    panic!("VM halted");
                }
                match vm.read_port() {
                    Some(0) => {
                        info!("Wall");
                        // Wall
                        continue;
                    }
                    Some(1 | 2) => {
                        info!("Success");
                        // Move successful
                        if max_dist < dist0 + 1 {
                            max_dist = dist0 + 1;
                        }
                        dist_map.insert(p1, Entry {
                            dist: dist0 + 1,
                            vm,
                        });
                        queue.push_back(p1);
                    }
                    Some(n) => {
                        panic!("VM returned {}", n);
                    }
                    None => {
                        panic!("VM did not return output");
                    }
                }
            }
        }
    }
    max_dist
}
