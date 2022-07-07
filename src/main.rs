use std::io::{self, prelude::*, BufRead};
use structopt::StructOpt;
use std::time::Duration;
use crossterm::{
    queue,
    cursor,
    terminal,
    tty::IsTty,
    event::{self, Event, KeyCode},
    style::{
        Color,
        Print,
        Stylize,
        PrintStyledContent,
    },
};

macro_rules! gray {
    ($x:expr) => (Color::Rgb{r: $x, g: $x, b: $x})
}

const COMPLETED: Color = gray!(255);
const UNCOMPLETED: Color = gray!(100);
const ERROR: Color = Color::Rgb{r: 230, g: 0, b: 0};

const WORDS: &[&str] = &[
    "apple",
    "banana",
    "names",
    "know",
    "computer",
    "science",
    "knowledge",
    "fight",
    "hug",
    "love",
    "boyfriend",
];

fn next_word() -> &str {
    WORDS[rand() % WORDS.len()]
}

fn next_line(word_count: usize) -> &str {
    let mut next_words = vec![];
    for _ in 0..word_count {
        next_words.push(next_word());
    }
    next_words.into_iter()
        .reduce(|a, b| format!("{} {}", a, b))
        .unwrap_or("")
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "typing_test",
    usage = "typing_test",
    about = r#"A typing test based on rust"#
)]
struct Args {
    // /// Directly add an item to the todo list
    // #[structopt(short, long)]
    // add: Option<String>,
}

struct TypingTest {
    running: bool,
    stdout: io::Stdout,
    terminal_size: (u16, u16),
    buffer: String,
    line: &str,
    next_line: &str,
    index: usize,
}

impl TypingTest {
    fn new() -> Self {
        let terminal_size = terminal::size().expect("Could not get terminal size");
        Self {
            running: true,
            stdout: io::Stdout,
            terminal_size,
            buffer: String,
            line: next_line(10),
            next_line: next_line(10),
            index: 0,
        }
    }

    fn redraw(&mut self) -> crossterm::Result<()> {
        self.clear()?;
        queue!(
            self.stdout,
            "{}\n{}{}",
            self.line.with(UNCOMPLETED)
            self.next_line.with(UNCOMPLETED)
            cursor::MoveTo(
                self.index,
                0
            )
        )?;
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
                        return todo!();
                    }
                    KeyCode::Char(ch) => {
                        self.buffer.push(ch);
                        return todo!();
                    }
                }
            }
        }
        Ok(true)
    }

    fn run(&mut self) -> crossterm::Result<()> {
        self.redraw()?
        while self.running {
            if self.kbin()? {
                self.redraw()?;
            }
        }
        self.clear()
    }
}

fn main() -> crossterm::Result<()> {
    terminal::enable_raw_mode()?;
    TypingTest::new().run()?;
    terminal::disable_raw_mode()?;
    Ok(())
}
