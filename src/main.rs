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
    number: Option<i32>,
}

enum TestMode {
    WordCount(i32),
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
            stdout: io::Stdout,
            terminal_size,
            previous_lines: vec![],
            line: Line::new(),
            next_line: Line::new(),
            test_mode: TestMode::WordCount(args.number.unwrap_or(30)),
            0,
        }
    }

    fn redraw(&mut self) -> crossterm::Result<()> {
        self.clear()?;
        todo!()
    }

    fn get_next_line(&mut self) {
        self.previous_lines.push(self.line);
        self.line = self.next_line;
        self.next_line = Line::new();
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
                }
                Event::Key(key) => match key.code {
                    KeyCode::Esc => {
                        self.running = false;
                    }
                    KeyCode::Backspace => {
                        self.line.backspace();
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
                    }
                }
            }
        }
        Ok(true)
    }

    fn run(&mut self) -> crossterm::Result<()> {
        self.redraw()?;
        while self.running {
            if self.kbin()? {
                self.redraw()?;
            }
            match self.test_mode {
                TestMode::WordCount(words) => if words > self.word_count {
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
