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
