use failure::Fail;

pub type BibleBotResult<T> = Result<T, BibleBotError>;

#[derive(Debug, Fail)]
pub enum BibleBotError {
    #[fail(display = "No references were found")]
    NoRefs,
}