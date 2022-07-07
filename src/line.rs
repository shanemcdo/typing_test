use std::io;
use crossterm::{
    queue,
    style::{
        Color,
        Print,
        Stylize,
        PrintStyledContent,
    },
};

const COMPLETED: Color = gray(255);
const UNCOMPLETED: Color = gray(100);
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

const fn gray(x: u8) -> Color {
    Color::Rgb{ r: x, g: x, b: x }
}

fn next_word() -> &str {
    WORDS[rand::random() % WORDS.len()]
}

fn next_line() -> &str {
    std::iter::repeat_with(next_word)
        .take(10)
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

    pub fn backspace(&mut self) -> bool {
        if self.index > 0 {
            self.index -= 1;
            true
        } else {
            false
        }
    }

    pub fn add_char(&mut self, ch: char) {
        self.buffer.push(ch);
    }

    pub fn draw(&self, stdout: &mut io::Stdout) -> crossterm::Result<()>{
        let buffer: Vec<char> = self.buffer.chars.collect();
        let expected: Vec<char> = self.expected.chars.collect();
        for i in 0..buffer.len().max(expected.len()) {
            if i > buffer.len() {
                queue!(
                    stdout,
                    PrintStyledContent(expected[i].with(UNCOMPLETED))
                )?;
            } else if i > expected.len() {
                queue!(
                    stdout,
                    PrintStyledContent(buffer[i].with(ERROR))
                )?;
            } else {
                let actual = buffer[i];
                let expected = expected[i];
                let color = if actual == expected {
                    COMPLETED
                } else {
                    ERROR
                };
                queue!(
                    stdout,
                    PrintStyledContent(buffer[i].with(color))
                )?;
            }
        }
        Ok(())
    }

    pub fn done(&self) -> bool {
        self.index > self.buffer.len()
    }
}
