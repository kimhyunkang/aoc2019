#![feature(array_chunks)]
#![feature(unsigned_signed_diff)]

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use day13::{Console, Game, Tile};
use intcode::parse_program;

fn main() {
    env_logger::init();

    let input = std::fs::read_to_string("input.txt").unwrap();
    let mut program = parse_program(&input);
    let counter = Rc::new(RefCell::new(CountConsole::new()));
    let mut game = Game::init(counter.clone(), program.clone());
    assert!(game.run().is_ready());
    let blocks = counter
        .borrow()
        .tiles
        .values()
        .filter(|&&t| t == Tile::Block)
        .count();
    println!("1: {}", blocks);

    let console = Rc::new(RefCell::new(AutoConsole::default()));
    program[0] = 2;
    let mut game = Game::init(console.clone(), program.clone());
    while !game.run().is_ready() {
        let input = console.borrow().auto_joystick();
        game.joystick_input(input);
    }
    println!("2: {}", console.borrow().score);
}

struct CountConsole {
    tiles: HashMap<(u16, u16), Tile>,
}

impl CountConsole {
    fn new() -> Self {
        Self {
            tiles: HashMap::new(),
        }
    }
}

impl Console for CountConsole {
    fn draw(&mut self, pos: (u16, u16), tile: Tile) {
        self.tiles.insert(pos, tile);
    }

    fn set_score(&mut self, _score: isize) {}
}

#[derive(Default)]
struct AutoConsole {
    ball_x: Option<u16>,
    paddle_x: Option<u16>,
    score: isize,
}

impl AutoConsole {
    fn auto_joystick(&self) -> isize {
        match (self.ball_x, self.paddle_x) {
            (Some(bx), Some(px)) => bx.checked_signed_diff(px).unwrap() as isize,
            _ => 0,
        }
    }
}

impl Console for AutoConsole {
    fn draw(&mut self, (x, _y): (u16, u16), tile: Tile) {
        match tile {
            Tile::Ball => {
                self.ball_x = Some(x);
            }
            Tile::Paddle => {
                self.paddle_x = Some(x);
            }
            _ => (),
        }
    }

    fn set_score(&mut self, score: isize) {
        self.score = score;
    }
}
