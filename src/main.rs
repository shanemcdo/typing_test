mod line;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    queue,
    style::{Print, Stylize},
    terminal,
};
use line::Line;
use std::io::{self, prelude::*};
use std::time::Duration;
use std::time::Instant;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "typing_test",
    usage = "typing_test [flags]",
    about = r#"A program to test your typing speed
  Controls:
    Esc - Exit test
    Tab - Restart test
    Letters - Enter input into the test
    Backspace - Undo input from the test
"#
)]
struct Args {
    /// The number of words to type before a test ends
    #[structopt(short, long)]
    number: Option<u32>,

    /// How long the test should run in seconds
    #[structopt(short, long)]
    time: Option<u64>,
}

/// Struct that indicates when to stop the typing test
enum TestMode {
    WordCount(u32),
    TimeLimit(u64),
}

impl std::fmt::Display for TestMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            TestMode::WordCount(wc) => write!(formatter, "{} words", wc),
            TestMode::TimeLimit(seconds) => write!(formatter, "{} seconds", seconds),
        }
    }
}

/// holds info about current typing test
struct TypingTest {
    running: bool,
    started: bool,
    show_final_score: bool,
    stdout: io::Stdout,
    previous_line: Line,
    line: Line,
    next_line: Line,
    test_mode: TestMode,
    _word_count: u32,
    instant: Option<Instant>,
}

impl TypingTest {
    fn new(args: Args) -> Self {
        Self {
            running: true,
            started: false,
            show_final_score: true,
            stdout: io::stdout(),
            previous_line: Line::empty(),
            line: Line::new(),
            next_line: Line::new(),
            test_mode: if let Some(seconds) = args.time {
                TestMode::TimeLimit(seconds)
            } else {
                TestMode::WordCount(args.number.unwrap_or(30))
            },
            _word_count: 0,
            instant: None,
        }
    }

    /// calculate word count
    fn word_count(&self) -> u32 {
        self._word_count + self.line.word_count()
    }

    /// Draw line containing words completed, time passed, wpm, and test mode
    fn draw_score(&mut self) -> crossterm::Result<()> {
        let time = self
            .instant
            .map(|x| x.elapsed().as_secs_f32())
            .unwrap_or(0f32);
        let wc = self.word_count();
        let wpm = wc as f32 / (time / 60f32);
        let mode = &self.test_mode;
        queue!(
            self.stdout,
            Print(format!(
                "{}: {}  {}: {}s  {}: {}  {}: {}",
                "Words".red().bold(),
                wc,
                "Time".green().bold(),
                time,
                "wpm".blue().bold(),
                wpm,
                "Mode".yellow().bold(),
                mode
            )),
            cursor::MoveToNextLine(1)
        )
    }

    /// Redraw the entire screen
    fn redraw(&mut self) -> crossterm::Result<()> {
        self.clear()?;
        self.draw_score()?;
        self.previous_line.draw(&mut self.stdout)?;
        self.line.draw(&mut self.stdout)?;
        self.next_line.draw(&mut self.stdout)?;
        let x = self.line.index as u16;
        queue!(self.stdout, cursor::MoveTo(x, 2))?;
        self.stdout.flush()
    }

    /// Move cursor to the next line and get next needed lines
    fn get_next_line(&mut self) {
        self._word_count += self.line.word_count();
        std::mem::swap(&mut self.line, &mut self.next_line);
        self.previous_line = std::mem::take(&mut self.next_line);
    }

    /// clear the screen
    fn clear(&mut self) -> crossterm::Result<()> {
        queue!(
            self.stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
        )
    }

    /// Handle keyboard input
    fn kbin(&mut self) -> crossterm::Result<()> {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => self.quit(),
                    KeyCode::Backspace => self.line.backspace(),
                    KeyCode::Tab => self.reset(),
                    KeyCode::Char(ch) => {
                        if !self.started {
                            self.started = true;
                            self.instant = Some(Instant::now());
                        }
                        if ch == ' ' && self.line.done() {
                            self.get_next_line();
                        } else {
                            self.line.add_char(ch);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Quit the test early
    fn quit(&mut self) {
        self.running = false;
        self.show_final_score = false;
    }

    /// Restart the test
    fn reset(&mut self) {
        self.started = false;
        self.previous_line = Line::empty();
        self.line = Line::new();
        self.next_line = Line::new();
        self._word_count = 0;
        self.instant = None;
    }

    /// Start the test application
    fn run(&mut self) -> crossterm::Result<()> {
        terminal::enable_raw_mode()?;
        self.redraw()?;
        while self.running {
            self.kbin()?;
            self.redraw()?;
            match self.test_mode {
                TestMode::WordCount(words) => {
                    if self.word_count() >= words {
                        break;
                    }
                }
                TestMode::TimeLimit(seconds) => {
                    if let Some(instant) = self.instant {
                        if instant.elapsed().as_secs() >= seconds {
                            break;
                        }
                    }
                }
            }
        }
        self.clear()?;
        terminal::disable_raw_mode()?;
        if self.show_final_score {
            if let Some(instant) = self.instant {
                let elapsed = instant.elapsed().as_secs_f32();
                let wc = self.word_count();
                println!("You typed {} words {} seconds", wc, elapsed);
                println!("Thats {} wpm", wc as f32 / (elapsed / 60f32));
            }
        }
        Ok(())
    }
}

fn main() -> crossterm::Result<()> {
    let args = Args::from_args();
    TypingTest::new(args).run()
}
