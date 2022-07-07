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
    previous_lines: Vec<Line>,
    line: Line,
    next_line: Line,
}

impl TypingTest {
    fn new() -> Self {
        let terminal_size = terminal::size().expect("Could not get terminal size");
        Self {
            running: true,
            stdout: io::Stdout,
            terminal_size,
            previous_lines: vec![],
            line: Line::new(),
            next_line: Line::new(),
        }
    }

    fn redraw(&mut self) -> crossterm::Result<()> {
        self.clear()?;
        todo!()
    }

    fn move_down(&mut self) {
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
                        return todo!();
                    }
                    KeyCode::Char(ch) => {
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
