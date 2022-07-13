use serde::Deserialize;

#[derive(Deserialize)]
struct Response {
    content: String,
}

pub fn random_quote() -> String {
    reqwest::blocking::get("https://api.quotable.io/random")
        .expect("Couldn't get url")
        .json::<Response>()
        .expect("Couldn't decode text")
        .content
}
