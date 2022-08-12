//! Used to get quotes from <https://api.quotable.io/random>
use serde::Deserialize;

/// Holds response from <https://api.quotable.io/random>
#[derive(Deserialize)]
struct Response {
    content: String,
}

/// Use reqwest to get quotes from <https://api.quotable.io/random>
pub fn random_quote() -> String {
    let err_prefix = "Could not get quote because";
    let url = "https://api.quotable.io/random";
    reqwest::blocking::get(url)
        .unwrap_or_else(|_| {
            eprintln!("{err_prefix} the url \"{url}\" cannot be fetched.");
            std::process::exit(1);
        })
        .json::<Response>()
        .unwrap_or_else(|_| {
            eprintln!("{err_prefix} the url \"{url}\" returned an unexpected result.");
            std::process::exit(1);
        })
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
