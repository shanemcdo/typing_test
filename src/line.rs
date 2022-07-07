use std::io;
use crossterm::{
    queue,
    cursor,
    terminal,
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
    // TODO get rand to work
    WORDS[rand() % WORDS.len()]
}

fn next_line() -> &str {
    let mut next_words = vec![];
    for _ in 0..10 {
        next_words.push(next_word());
    }
    next_words.into_iter()
        .reduce(|a, b| format!("{} {}", a, b))
        .unwrap_or("")
}

#[derive(Clone, Debug)]
pub struct Line {
    buffer: String,
    expected: &str,
    index: usize,
}

impl Line {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            expected: next_line(),
            index: 0,
        }
    }

    pub fn backspace(&mut self) -> bool{
        if self.index > 0 {
            self.index -= 1;
            true
        } else {
            false
        }
    }

    pub fn draw(&self, stdout: &mut io::Stdout) -> crossterm::Result<()>{
        for i in 0..self.buffer.len().max(self.expected.len()) {
            if i > self.buffer.len() {
                queue!(
                    stdout,
                    PrintStyledContent(self.expected[i].with(UNCOMPLETED)
                )?;
            } else if i > self.expected.len() {
                queue!(
                    stdout,
                    PrintStyledContent(self.buffer[i].with(ERROR)
                )?;
            } else {
                let actual = self.buffer[i];
                let expected = self.expected[i];
                let color = if actual == expected {
                    COMPLETED
                } else {
                    ERROR
                };
                queue!(
                    stdout,
                    PrintStyledContent(self.buffer[i].with(color))
                )?;
            }
        }
        Ok(())
    }

    pub fn done(&self) -> bool {
        self.index > self.buffer.len()
    }
}
