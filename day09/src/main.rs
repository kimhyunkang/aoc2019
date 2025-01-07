use intcode::{VM, parse_program};

fn main() {
    env_logger::init();

    let input = std::fs::read_to_string("input.txt").unwrap();
    let program = parse_program(&input);

    let mut vm1 = VM::init(program.clone());
    vm1.write_port(&[1]);
    let output = vm1.run_ready();

    println!("1: {}", output[0]);

    let mut vm2 = VM::init(program);
    vm2.write_port(&[2]);
    let output = vm2.run_ready();

    println!("2: {}", output[0]);
}
