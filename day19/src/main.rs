use intcode::{VM, parse_program};
use log::{info, log_enabled};
use ndarray::Array2;

fn main() {
    env_logger::init();

    let input = std::fs::read_to_string("input.txt").unwrap();
    let program = parse_program(&input);
    let q = VMQuery { program };
    let (image, count) = scan(&q, 50);
    if log_enabled!(log::Level::Info) {
        for y in 0..50 {
            let buf = (0..50)
                .map(|x| if image[(x, y)] { '#' } else { '.' })
                .collect::<String>();
            info!("{}", buf);
        }
    }
    println!("1: {}", count);
}

trait Query {
    fn query(&self, x: usize, y: usize) -> bool;
}

struct VMQuery {
    program: Vec<isize>,
}

impl Query for VMQuery {
    fn query(&self, x: usize, y: usize) -> bool {
        let mut vm = VM::init(self.program.clone());
        vm.write_port(&[x as isize, y as isize]);
        if !vm.run().is_ready() {
            panic!("VM should have shut down");
        }
        match vm.read_port().unwrap() {
            0 => false,
            1 => true,
            n => {
                panic!("Unexpected output {}", n);
            }
        }
    }
}

fn scan<Q: Query>(q: &Q, n: usize) -> (Array2<bool>, usize) {
    let mut count = 0;
    let mut field = Array2::from_elem((n, n), false);
    for y in 0..n {
        for x in 0..n {
            if q.query(x, y) {
                count += 1;
                field[(x, y)] = true;
            }
        }
    }
    (field, count)
}
