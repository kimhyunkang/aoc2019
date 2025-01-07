use std::{
    cell::RefCell,
    collections::VecDeque,
    io::{self, Write},
    rc::Rc,
};

use crossterm::{
    ExecutableCommand,
    cursor::MoveTo,
    event::{Event, KeyCode, KeyModifiers},
    execute, queue,
    terminal::{Clear, ClearType},
};
use day13::{Console, Game, Tile, vm::parse_program};

fn main() -> io::Result<()> {
    env_logger::init();

    crossterm::terminal::enable_raw_mode()?;
    io::stdout().execute(crossterm::cursor::Hide)?;

    let result = mainloop();

    io::stdout().execute(crossterm::cursor::Show)?;
    crossterm::terminal::disable_raw_mode()?;
    result
}

fn mainloop() -> io::Result<()> {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let mut program = parse_program(&input);
    program[0] = 2;
    let console = Rc::new(RefCell::new(TUI::init((21, 38))));
    console.borrow().clearscreen()?;
    let mut game = Game::init(console.clone(), program);
    while game.run().is_pending() {
        console.borrow_mut().flush()?;
        if let Some(input) = read_joystick()? {
            game.joystick_input(input);
        } else {
            break;
        }
    }
    Ok(())
}

fn read_joystick() -> io::Result<Option<isize>> {
    loop {
        if let Event::Key(key) = crossterm::event::read()? {
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                return Ok(None);
            }
            if !key.modifiers.is_empty() {
                continue;
            }
            match key.code {
                KeyCode::Char(' ') => return Ok(Some(0)),
                KeyCode::Left => return Ok(Some(-1)),
                KeyCode::Right => return Ok(Some(1)),
                _ => (),
            }
        }
    }
}

pub struct TUI {
    dim: (u16, u16),
    buf: VecDeque<((u16, u16), Tile)>,
    score: Option<isize>,
}

impl TUI {
    fn init(dim: (u16, u16)) -> Self {
        Self {
            dim,
            buf: VecDeque::new(),
            score: None,
        }
    }

    fn clearscreen(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        let (h, _) = self.dim;
        execute!(stdout, Clear(ClearType::All), MoveTo(0, h))?;
        write!(&mut stdout, "Score: {}", self.score.unwrap_or(0))
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        while let Some(((x, y), tile)) = self.buf.pop_front() {
            let ch = match tile {
                Tile::Empty => ' ',
                Tile::Wall => '#',
                Tile::Block => '$',
                Tile::Paddle => '=',
                Tile::Ball => 'O',
            };
            queue!(stdout, MoveTo(x, y))?;
            write!(&mut stdout, "{}", ch)?;
        }
        if let Some(score) = self.score.take() {
            let (h, _) = self.dim;
            queue!(stdout, MoveTo(0, h), Clear(ClearType::CurrentLine))?;
            write!(&mut stdout, "Score: {}", score)?;
        }
        stdout.flush()
    }
}

impl Console for TUI {
    fn draw(&mut self, pos: (u16, u16), tile: Tile) {
        self.buf.push_back((pos, tile));
    }

    fn set_score(&mut self, score: isize) {
        self.score = Some(score);
    }
}
