use std::io;

use day25::*;
use intcode::parse_program;

fn main() -> io::Result<()> {
    env_logger::init();

    let input = std::fs::read_to_string("input.txt").unwrap();
    let program = parse_program(&input);
    let graph = Graph::scan(program)?;
    graph.search();
    Ok(())
}
