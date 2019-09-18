use crate::bible_lookup::err::BibleLookupError;
use failure::{self, Fail};

pub type BibleBotResult<T> = Result<T, BibleBotError>;

#[derive(Debug, Fail)]
pub enum BibleBotError {
    #[fail(display = "{}", _0)]
    Lookup(BibleLookupError),

    #[fail(display = "Could not respond to Reddit comment: {}", _0)]
    RedditResponse(failure::Error)
}

impl From<BibleLookupError> for BibleBotError {
    fn from(err: BibleLookupError) -> Self {
        BibleBotError::Lookup(err)
    }
}

impl From<failure::Error> for BibleBotError {
    fn from(err: failure::Error) -> Self {
        BibleBotError::RedditResponse(err)
    }
}