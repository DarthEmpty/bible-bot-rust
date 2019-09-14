use failure::Fail;
use reqwest;
use serde_json;

pub type BibleBotResult<T> = Result<T, BibleBotError>;

#[derive(Debug, Fail)]
pub enum BibleBotError {
    #[fail(display = "No references were found.")]
    NoRefs,

    #[fail(display = "Could not complete request: {}", _0)]
    Request(reqwest::Error),

    #[fail(display = "Could not parse JSON: {}", _0)]
    Parse(serde_json::Error),

    #[fail(display = "Passage could not be constructed as its type was neither a 'chapter' nor a 'verse'.")]
    BadPassageType,

    #[fail(display = "Could not find requested passage: {}.", _0)]
    PassageNotFound(&'static str),
}

impl From<reqwest::Error> for BibleBotError {
    fn from(err: reqwest::Error) -> Self {
        BibleBotError::Request(err)
    }
}

impl From<serde_json::Error> for BibleBotError {
    fn from(err: serde_json::Error) -> Self {
        BibleBotError::Parse(err)
    }
}
