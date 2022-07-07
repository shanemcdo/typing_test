mod line;

use std::time::Instant;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    queue, terminal,
};
use line::Line;
use std::io::{self, prelude::*};
use std::time::Duration;
use structopt::StructOpt;

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
    started: bool,
    stdout: io::Stdout,
    terminal_size: (u16, u16),
    previous_line: Line,
    line: Line,
    next_line: Line,
    test_mode: TestMode,
    word_count: u32,
    instant: Instant,
}

impl TypingTest {
    fn new(args: Args) -> Self {
        let terminal_size = terminal::size().expect("Could not get terminal size");
        Self {
            running: true,
            started: false,
            stdout: io::stdout(),
            terminal_size,
            previous_line: Line::empty(),
            line: Line::new(),
            next_line: Line::new(),
            test_mode: TestMode::WordCount(args.number.unwrap_or(30)),
            word_count: 0,
            instant: Instant::now(),
        }
    }

    fn redraw(&mut self) -> crossterm::Result<()> {
        self.clear()?;
        self.previous_line.draw(&mut self.stdout)?;
        self.line.draw(&mut self.stdout)?;
        self.next_line.draw(&mut self.stdout)?;
        let x = self.line.index as u16;
        queue!(self.stdout, cursor::MoveTo(x, 1))?;
        self.stdout.flush()?;
        Ok(())
    }

    fn get_next_line(&mut self) {
        std::mem::swap(&mut self.line, &mut self.next_line);
        self.previous_line = std::mem::take(&mut self.next_line);
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
                        if !self.started {
                            self.started = true;
                            self.instant = Instant::now();
                        }
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
                },
                _ => {}
            }
        }
        Ok(false)
    }

    fn run(&mut self) -> crossterm::Result<()> {
        terminal::enable_raw_mode()?;
        self.redraw()?;
        while self.running {
            if self.kbin()? {
                self.redraw()?;
            }
            match self.test_mode {
                TestMode::WordCount(words) => {
                    if self.word_count >= words {
                        break;
                    }
                }
            }
        }
        let elapsed = self.instant.elapsed().as_secs_f32();
        self.clear()?;
        terminal::disable_raw_mode()?;
        println!("You typed {} words {} seconds", self.word_count, elapsed);
        println!("Thats {} wpm", self.word_count as f32 / (elapsed / 60f32));
        Ok(())
    }
}

fn main() -> crossterm::Result<()> {
    let args = Args::from_args();
    TypingTest::new(args).run()
}
