use crate::bible_lookup::err::BibleLookupError;
use crate::s3_access::err::S3AccessError;
use failure::{self, Fail};

pub type BibleBotResult<T> = Result<T, BibleBotError>;

#[derive(Debug, Fail)]
pub enum BibleBotError {
    #[fail(display = "{}", _0)]
    Lookup(BibleLookupError),

    #[fail(display = "Could not respond to Reddit comment: {}", _0)]
    RedditResponse(failure::Error),

    #[fail(display = "{}", _0)]
    Storage(S3AccessError),
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

impl From<S3AccessError> for BibleBotError {
    fn from(err: S3AccessError) -> Self {
        BibleBotError::Storage(err)
    }
}