#![feature(array_chunks)]

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use day13::{Console, Game, Tile, vm::parse_program};

fn main() {
    env_logger::init();

    let input = std::fs::read_to_string("input.txt").unwrap();
    let program = parse_program(&input);
    let console = Rc::new(RefCell::new(VirtualConsole::new()));
    let mut game = Game::init(console.clone(), program);
    assert!(game.run().is_ready());
    let blocks = console
        .borrow()
        .tiles
        .values()
        .filter(|&&t| t == Tile::Block)
        .count();
    println!("1: {}", blocks);
}

struct VirtualConsole {
    tiles: HashMap<(u16, u16), Tile>,
}

impl VirtualConsole {
    fn new() -> Self {
        Self {
            tiles: HashMap::new(),
        }
    }
}

impl Console for VirtualConsole {
    fn draw(&mut self, pos: (u16, u16), tile: Tile) {
        self.tiles.insert(pos, tile);
    }

    fn set_score(&mut self, score: isize) {}
}
