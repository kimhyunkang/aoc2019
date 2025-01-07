use intcode::VM;

use std::{cell::RefCell, rc::Rc, task::Poll};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[repr(u8)]
#[derive(FromPrimitive, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Empty = 0,
    Wall = 1,
    Block = 2,
    Paddle = 3,
    Ball = 4,
}

pub struct Game<C> {
    vm: VM,
    console: Rc<RefCell<C>>,
}

pub trait Console {
    fn draw(&mut self, pos: (u16, u16), tile: Tile);
    fn set_score(&mut self, score: isize);
}

impl<C: Console> Game<C> {
    pub fn init(console: Rc<RefCell<C>>, program: Vec<isize>) -> Self {
        Game {
            vm: VM::init(program),
            console,
        }
    }

    pub fn run(&mut self) -> Poll<()> {
        let poll = self.vm.run();
        let mut buf: [isize; 3] = [0; 3];
        while self.vm.read_exact(&mut buf[..]).is_ready() {
            let [x, y, c] = buf;
            if x == -1 && y == 0 {
                self.console.borrow_mut().set_score(c);
            } else {
                let pos: (u16, u16) = (x.try_into().unwrap(), y.try_into().unwrap());
                let t: Tile = Tile::from_isize(c).unwrap();
                self.console.borrow_mut().draw(pos, t);
            }
        }
        poll
    }

    pub fn joystick_input(&mut self, input: isize) {
        self.vm.write_port(&[input]);
    }
}
