mod line;

use line::Line;
use std::io::{self, prelude::*, BufRead};
use structopt::StructOpt;
use std::time::Duration;
use crossterm::{
    queue,
    cursor,
    terminal,
    event::{self, Event, KeyCode},
    style::{
        Color,
        Print,
        Stylize,
        PrintStyledContent,
    },
};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "typing_test",
    usage = "typing_test",
    about = r#"A program to test your typing speed"#
)]
struct Args {
    /// The number of words to type before a test ends
    #[structopt(short, long)]
    number: Option<u32>,
}

enum TestMode {
    WordCount(u32),
}

struct TypingTest {
    running: bool,
    stdout: io::Stdout,
    terminal_size: (u16, u16),
    previous_lines: Vec<Line>,
    line: Line,
    next_line: Line,
    test_mode: TestMode,
    word_count: u32,
}

impl TypingTest {
    fn new(args: Args) -> Self {
        let terminal_size = terminal::size().expect("Could not get terminal size");
        Self {
            running: true,
            stdout: io::stdout(),
            terminal_size,
            previous_lines: vec![],
            line: Line::new(),
            next_line: Line::new(),
            test_mode: TestMode::WordCount(args.number.unwrap_or(30)),
            word_count: 0,
        }
    }

    fn redraw(&mut self) -> crossterm::Result<()> {
        self.clear()?;
        for line in &self.previous_lines {
            line.draw(&mut self.stdout)?;
        }
        self.line.draw(&mut self.stdout)?;
        self.next_line.draw(&mut self.stdout)?;
        let x = self.line.index as u16;
        let y = self.previous_lines.len() as u16;
        queue!(
            self.stdout,
            cursor::MoveTo(x, y),
            Print("TEST!")
        )
    }

    fn get_next_line(&mut self) {
        std::mem::swap(&mut self.line, &mut self.next_line);
        let temp = std::mem::replace(&mut self.next_line, Line::new());
        self.previous_lines.push(temp);
    }

    fn clear(&mut self) -> crossterm::Result<()> {
        queue!(
            self.stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
        )
    }

    /// Handle keyboard input
    /// returns Ok(true) if needs to be reloaded
    fn kbin(&mut self) -> crossterm::Result<bool> {
        if event::poll(Duration::from_millis(50))? {
            let evnt = event::read()?;
            match evnt {
                Event::Resize(w, h) => {
                    self.terminal_size = (w, h);
                    return Ok(true);
                }
                Event::Key(key) => match key.code {
                    KeyCode::Esc => {
                        self.running = false;
                    }
                    KeyCode::Backspace => {
                        self.line.backspace();
                        return Ok(true);
                    }
                    KeyCode::Char(ch) => {
                        if ch == ' ' && self.line.done() {
                            self.get_next_line();
                        } else { 
                            self.line.add_char(ch);
                        }
                        if ch == ' ' {
                            self.word_count += 1;
                        }
                        return Ok(true);
                    }
                    _ => {}
                }
                _ => {}
            }
        }
        Ok(false)
    }

    fn run(&mut self) -> crossterm::Result<()> {
        self.redraw()?;
        while self.running {
            if self.kbin()? {
                self.redraw()?;
            }
            match self.test_mode {
                TestMode::WordCount(words) => if self.word_count > words {
                    break;
                }
            }
        }
        self.clear()
    }
}

fn main() -> crossterm::Result<()> {
    let args = Args::from_args();
    terminal::enable_raw_mode()?;
    TypingTest::new(args).run()?;
    terminal::disable_raw_mode()
}
