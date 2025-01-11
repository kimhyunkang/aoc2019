use std::io;

use day25::{convert_ascii, write_console};
use intcode::{VM, parse_program};

fn rl_error(e: rustyline::error::ReadlineError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, e)
}

fn main() -> io::Result<()> {
    env_logger::init();

    let input = std::fs::read_to_string("input.txt").unwrap();
    let program = parse_program(&input);

    let mut vm = VM::init(program);
    let mut rl = rustyline::DefaultEditor::new().map_err(rl_error)?;

    while !vm.run().is_ready() {
        write_console(&vm.read_all())?;
        let readline = rl.readline("> ").map_err(rl_error)?;
        vm.write_port(&convert_ascii(readline.as_bytes()));
        vm.write_port(&[b'\n' as isize]);
    }

    Ok(())
}
