#![feature(unsigned_signed_diff)]

use std::{
    cell::RefCell,
    collections::VecDeque,
    io::{self, Write},
    rc::Rc,
    time::Duration,
};

use crossterm::{
    cursor::MoveTo,
    event::{Event, KeyCode, KeyModifiers},
    execute, queue,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use day13::{Console, Game, Tile};
use intcode::parse_program;

fn main() -> io::Result<()> {
    env_logger::init();

    crossterm::terminal::enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, crossterm::cursor::Hide)?;

    let result = mainloop();

    execute!(io::stdout(), crossterm::cursor::Show, LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    result
}

fn mainloop() -> io::Result<()> {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let mut program = parse_program(&input);
    program[0] = 2;
    let console = Rc::new(RefCell::new(TUI::init((21, 38))));
    console.borrow_mut().clearscreen()?;
    console.borrow_mut().flush()?;
    let mut game = Game::init(console.clone(), program);

    while game.run().is_pending() {
        console.borrow_mut().flush()?;
        let autoplay = console.borrow().autoplay;
        let joystick = read_joystick(autoplay)?;
        match joystick {
            Joystick::Auto => {
                console.borrow_mut().toggle_autoplay()?;
                continue;
            }
            Joystick::Exit => {
                return Ok(());
            }
            Joystick::Timeout => {
                if autoplay {
                    let input = console.borrow().auto_joystick();
                    game.joystick_input(input);
                }
            }
            Joystick::Input(input) => {
                game.joystick_input(input);
            }
        }
    }
    Ok(())
}

enum Joystick {
    Input(isize),
    Exit,
    Auto,
    Timeout,
}

fn read_joystick(timeout: bool) -> io::Result<Joystick> {
    loop {
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = crossterm::event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    return Ok(Joystick::Exit);
                }
                if !key.modifiers.is_empty() {
                    continue;
                }
                match key.code {
                    KeyCode::Char(' ') => return Ok(Joystick::Input(0)),
                    KeyCode::Char('a') => return Ok(Joystick::Auto),
                    KeyCode::Left => return Ok(Joystick::Input(-1)),
                    KeyCode::Right => return Ok(Joystick::Input(1)),
                    KeyCode::Enter => return Ok(Joystick::Auto),
                    _ => (),
                }
            }
        } else if timeout {
            return Ok(Joystick::Timeout);
        }
    }
}

pub struct TUI {
    dim: (u16, u16),
    buf: VecDeque<((u16, u16), Tile)>,
    autoplay: bool,
    ball: Option<(u16, u16)>,
    paddle: Option<(u16, u16)>,
    score: Option<isize>,
}

impl TUI {
    fn init(dim: (u16, u16)) -> Self {
        Self {
            dim,
            buf: VecDeque::new(),
            autoplay: false,
            ball: None,
            paddle: None,
            score: None,
        }
    }

    fn clearscreen(&mut self) -> io::Result<()> {
        execute!(io::stdout(), Clear(ClearType::All))?;
        self.set_score(0);
        self.autoplay = false;
        Ok(())
    }

    fn toggle_autoplay(&mut self) -> io::Result<()> {
        self.autoplay = !self.autoplay;
        let mut stdout = io::stdout();
        self.show_autoplay(&mut stdout)
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
            self.show_autoplay(&mut stdout)?;
        }
        stdout.flush()
    }

    fn show_autoplay(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        let (h, w) = self.dim;
        queue!(stdout, MoveTo(w - 13, h))?;
        if self.autoplay {
            write!(stdout, "Autoplay: On ")
        } else {
            write!(stdout, "Autoplay: Off")
        }
    }

    fn auto_joystick(&self) -> isize {
        match (self.ball, self.paddle) {
            (Some((bx, _)), Some((px, _))) => bx.checked_signed_diff(px).unwrap() as isize,
            _ => 0,
        }
    }
}

impl Console for TUI {
    fn draw(&mut self, pos: (u16, u16), tile: Tile) {
        match tile {
            Tile::Ball => {
                self.ball = Some(pos);
            }
            Tile::Paddle => {
                self.paddle = Some(pos);
            }
            _ => (),
        }
        self.buf.push_back((pos, tile));
    }

    fn set_score(&mut self, score: isize) {
        self.score = Some(score);
    }
}
