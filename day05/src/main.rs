use intcode::VM;

fn main() {
    env_logger::init();

    let input = std::fs::read_to_string("input.txt").unwrap();
    let code: Vec<isize> = input
        .trim()
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();

    let mut vm = VM::init(code.clone());
    vm.write_port(&[1]);
    let mut output = vm.run_ready();
    let diag = output.pop().unwrap();
    assert!(output.into_iter().all(|n| n == 0));
    println!("1: {:?}", diag);

    let mut vm = VM::init(code.clone());
    vm.write_port(&[5]);
    let output = vm.run_ready();
    assert_eq!(output.len(), 1);
    println!("2: {:?}", output[0]);
}
