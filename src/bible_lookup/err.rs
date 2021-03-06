use failure::Fail;
use reqwest;
use serde_json;

pub type BibleLookupResult<T> = Result<T, BibleLookupError>;

#[derive(Debug, Fail)]
pub enum BibleLookupError {
    #[fail(display = "No references were found.")]
    NoRefs,

    #[fail(display = "Could not complete request: {}", _0)]
    Request(reqwest::Error),

    #[fail(display = "Could not parse JSON: {}", _0)]
    Parse(serde_json::Error),

    #[fail(display = "Passage could not be constructed as its type was neither a 'chapter' nor a 'verse'.")]
    BadPassageType,
}

impl From<reqwest::Error> for BibleLookupError {
    fn from(err: reqwest::Error) -> Self {
        BibleLookupError::Request(err)
    }
}

impl From<serde_json::Error> for BibleLookupError {
    fn from(err: serde_json::Error) -> Self {
        BibleLookupError::Parse(err)
    }
}
