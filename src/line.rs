//! Contains struct for keeping track of lines of user input and expected input
//! as well as generating new lines
use crossterm::{
    cursor, queue,
    style::{Color, PrintStyledContent, Stylize},
};
use std::io;

const COMPLETED: Color = gray(255);
const UNCOMPLETED: Color = gray(100);
const ERROR: Color = Color::Rgb { r: 230, g: 0, b: 0 };
const LINE_LEN: usize = 10;

/// ALL of the words possible
/// taken from <https://github.com/monkeytypegame/monkeytype/blob/master/frontend/static/languages/english.json>
const WORDS: &[&str] = include!("words.txt");

/// Return a color where the r, g, and b values are set to x
/// Effectively a grayscale color
const fn gray(x: u8) -> Color {
    Color::Rgb { r: x, g: x, b: x }
}

fn join<T>(x: T) -> String
where
    T: Iterator,
    T::Item: ToString,
{
    x.map(|x| x.to_string())
        .reduce(|a, b| format!("{} {}", a, b))
        .unwrap_or_default()
}

/// Get a random word from the list of words
fn next_word() -> &'static str {
    WORDS[rand::random::<usize>() % WORDS.len()]
}

/// Get a line comprised of {LINE_LEN} random words
fn next_line() -> String {
    join(std::iter::repeat_with(next_word).take(LINE_LEN))
}

/// A struct representing expected input and actual input
#[derive(Clone, Debug)]
pub struct Line {
    buffer: String,
    expected: String,
}

impl Default for Line {
    fn default() -> Self {
        Line::new()
    }
}

impl Line {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            expected: next_line(),
        }
    }

    /// Create a new Line using {LINE_LEN} words of a string
    /// Leaves remaining words in string
    pub fn from_quote(string: &mut String) -> Self {
        let mut it = string.split(' ');
        let res = Line {
            expected: join((&mut it).take(LINE_LEN)),
            ..Self::new()
        };
        *string = join(it);
        res
    }

    /// Create an empty line that has no expected input
    pub fn empty() -> Self {
        Self {
            expected: String::new(),
            ..Self::new()
        }
    }

    /// Get the x position for moving the cursor
    pub fn index(&self) -> usize {
        self.buffer.len()
    }

    /// Calculate the number of correctly completed words
    pub fn word_count(&self) -> u32 {
        let buffer: Vec<char> = self.buffer.chars().chain([' ']).collect();
        let expected: Vec<char> = self.expected.chars().collect();
        let mut word_correct = true;
        let mut count = 0;
        for i in 0..buffer.len() {
            if i >= expected.len() {
                if word_correct {
                    count += 1;
                }
                break;
            }
            if expected[i] == ' ' {
                if word_correct {
                    count += 1;
                }
                word_correct = true;
            }
            if buffer[i] != expected[i] {
                word_correct = false;
            }
        }
        count
    }

    /// remove one character if it exists
    pub fn backspace(&mut self) {
        self.buffer.pop();
    }

    /// Returns true if a word has been finshed
    pub fn add_char(&mut self, ch: char) {
        self.buffer.push(ch);
    }

    /// draw the line to provided stdout
    pub fn draw(&self, stdout: &mut io::Stdout) -> crossterm::Result<()> {
        let buffer: Vec<char> = self.buffer.chars().collect();
        let expected: Vec<char> = self.expected.chars().collect();
        for i in 0..buffer.len().max(expected.len()) {
            let ch = if i >= buffer.len() {
                expected[i].with(UNCOMPLETED)
            } else if i >= expected.len() {
                buffer[i].with(ERROR)
            } else {
                let color = if buffer[i] == expected[i] {
                    COMPLETED
                } else {
                    ERROR
                };
                if buffer[i] == ' ' && color == ERROR {
                    buffer[i].on(color)
                } else {
                    buffer[i].with(color)
                }
            };
            queue!(stdout, PrintStyledContent(ch))?;
        }
        queue!(stdout, cursor::MoveToNextLine(1))
    }

    /// return true if all of the expected input has been completed
    pub fn done(&self) -> bool {
        self.index() >= self.expected.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_test() {
        assert_eq!(
            join(1..=5),
            "1 2 3 4 5"
        );
        assert_eq!(
            join(["Here", "are", "some", "words"].iter()),
            "Here are some words"
        );
    }

    #[test]
    fn line_new_test() {
        for _ in 0..100 {
            let line = Line::new();
            assert_eq!(line.buffer, "");
            assert_ne!(line.expected, "");
        }
    }

    #[test]
    fn line_from_quote_test() {
        let mut s = "This is a quote".to_string();
        let s_clone = s.clone();
        let line = Line::from_quote(&mut s);
        assert_eq!(s, "");
        assert_eq!(line.expected, s_clone);
        let offset = 3;
        s = join(1..=(LINE_LEN + offset));
        let line = Line::from_quote(&mut s);
        assert_eq!(s, join((LINE_LEN+1)..=(LINE_LEN + offset)));
        assert_eq!(line.expected, join(1..=LINE_LEN));
    }

    #[test]
    fn line_empty_test() {
        let line = Line::empty();
        assert_eq!(line.buffer, "");
        assert_eq!(line.expected, "");
    }

    #[test]
    fn line_index_test() {
        let mut line = Line::new();
        line.buffer = "abc 12".to_string();
        assert_eq!(line.index(), 6);
        line.buffer = "123".to_string();
        assert_eq!(line.index(), 3);
        line.buffer = "This one is pretty long".to_string();
        assert_eq!(line.index(), 23);
    }

    #[test]
    fn line_word_count_test() {
        for (b, e, count) in [
            ("a b d", "a b c d", 2),
            ("a b c", "a b c d", 3),
            ("This is a quote!", "This is a quote!", 4),
            ("This is not a quote!", "This is a quote!", 2),
        ] {
            let line = Line {
                buffer: b.into(),
                expected: e.into(),
            };
            assert_eq!(line.word_count(), count);
        }
    }

    #[test]
    fn line_backspace_test() {
        Line::empty().backspace(); // shouldn't panic
        let mut line = Line::new();
        line.buffer = "abc".to_string();
        for _ in 0..3 {
            line.backspace();
        }
        assert_eq!(line.buffer.len(), 0);
    }

    #[test]
    fn line_add_char_test() {
        let mut line = Line::new();
        line.add_char('1');
        line.add_char('2');
        line.add_char('3');
        assert_eq!(line.buffer.len(), 3);
    }

    #[test]
    fn line_done_test() {
        for (b, e, done) in [
            ("a b d", "a b c d", false),
            ("a b c", "a b c d", false),
            ("This is a quote!", "This is a quote!", true),
            ("This is not a quote!", "This is a quote!", true),
            ("This is not a quote!", "This is a quote!", true),
            ("123", "1234", false),
        ] {
            let line = Line {
                buffer: b.into(),
                expected: e.into(),
            };
            assert_eq!(line.done(), done);
        }
    }
}
