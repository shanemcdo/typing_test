//! Main logic of a typing test application
mod line;
mod quote;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    queue,
    style::{Print, Stylize},
    terminal,
};
use line::Line;
use quote::random_quote;
use std::io::{self, prelude::*};
use std::time::Duration;
use std::time::Instant;
use structopt::StructOpt;

/// Used by structopt for parsing command-line arguments
#[derive(Debug, StructOpt)]
#[structopt(
    name = "typing_test",
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
    #[structopt(short, long, name = "WORDS")]
    number: Option<u32>,

    /// How long the test should run in seconds
    #[structopt(short, long, name = "SECONDS")]
    time: Option<u64>,

    /// Whether or not the test should run in Quote Mode
    #[structopt(short, long)]
    quote: bool,

    /// A custom quote to use
    #[structopt(short, long, name = "QUOTE")]
    custom_quote: Option<String>,
}

/// Enum that indicates when to stop the typing test
enum TestMode {
    /// Stop the test after a certain number of correct words typed
    WordCount(u32),
    /// Stop the test after a certain number of seconds elapsed
    TimeLimit(u64),
    /// Stop the test after finishing the quote
    QuoteMode {
        remaining: String,
        custom: Option<String>,
    },
}

impl std::fmt::Display for TestMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            TestMode::WordCount(wc) => write!(formatter, "{} words", wc),
            TestMode::TimeLimit(seconds) => write!(formatter, "{} seconds", seconds),
            TestMode::QuoteMode { .. } => write!(formatter, "quote"),
        }
    }
}

/// holds info about current typing test
struct TypingTest {
    running: bool,
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
        let mut test_mode = if let Some(seconds) = args.time {
            TestMode::TimeLimit(seconds)
        } else if args.quote {
            TestMode::QuoteMode {
                custom: args.custom_quote.clone(),
                remaining: args.custom_quote.unwrap_or_else(random_quote),
            }
        } else {
            TestMode::WordCount(args.number.unwrap_or(30))
        };
        let (line, next_line) = if let TestMode::QuoteMode { remaining, .. } = &mut test_mode {
            (Line::from_quote(remaining), Line::from_quote(remaining))
        } else {
            (Line::new(), Line::new())
        };
        Self {
            running: true,
            show_final_score: true,
            stdout: io::stdout(),
            previous_line: Line::empty(),
            line,
            next_line,
            test_mode,
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
                "{}: {}  {}: {:6.2}s  {}: {:6.2}  {}: {}",
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
        let x = self.line.index() as u16;
        queue!(self.stdout, cursor::MoveTo(x, 2))?;
        self.stdout.flush()
    }

    /// Move cursor to the next line and get next needed lines
    fn get_next_line(&mut self) {
        self._word_count += self.line.word_count();
        std::mem::swap(&mut self.line, &mut self.next_line);
        let new = if let TestMode::QuoteMode { remaining, .. } = &mut self.test_mode {
            Line::from_quote(remaining)
        } else {
            Line::new()
        };
        self.previous_line = std::mem::replace(&mut self.next_line, new);
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
                        if self.instant.is_none() {
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
        self.previous_line = Line::empty();
        self._word_count = 0;
        self.instant = None;
        if let TestMode::QuoteMode { remaining, custom } = &mut self.test_mode {
            if let Some(s) = custom {
                *remaining = s.clone();
            } else {
                *remaining = random_quote();
            }
            self.line = Line::from_quote(remaining);
            self.next_line = Line::from_quote(remaining);
        } else {
            self.line = Line::new();
            self.next_line = Line::new();
        }
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
                TestMode::QuoteMode { .. } => {
                    if self.line.done() && self.next_line.done() {
                        break;
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

/// Driver code that runs the application
fn main() -> crossterm::Result<()> {
    let mut args = Args::from_args();
    if args.custom_quote.is_some() {
        args.quote = true;
    }
    if args.time.is_some() && args.number.is_some()
        || args.time.is_some() && args.quote
        || args.number.is_some() && args.quote
    {
        println!("Invalid combination of flags. Please do not pass conflicting flags.");
        return Ok(());
    }
    TypingTest::new(args).run()
}
