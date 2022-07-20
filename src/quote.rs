//! Used to get quotes from <https://api.quotable.io/random>
use serde::Deserialize;

/// Holds response from <https://api.quotable.io/random>
#[derive(Deserialize)]
struct Response {
    content: String,
}

/// Use reqwest to get quotes from <https://api.quotable.io/random>
pub fn random_quote() -> String {
    reqwest::blocking::get("https://api.quotable.io/random")
        .expect("Couldn't get url")
        .json::<Response>()
        .expect("Couldn't decode text")
        .content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_quote_test() {
        for _ in 0..3 {
            assert_ne!(random_quote(), "");
        }
    }
}
