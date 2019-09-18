use failure::Fail;
use toml;

pub type S3AccessResult<T> = Result<T, S3AccessError>;

#[derive(Debug, Fail)]
pub enum S3AccessError {
    #[fail(display = "Unable to load file from bucket: {}", _0)]
    Load(String),

    #[fail(display = "Unable to save file to bucket: {}", _0)]
    Save(String),

    #[fail(display = "Could not parse TOML: {}", _0)]
    Parse(toml::de::Error),
}

impl From<toml::de::Error> for S3AccessError {
    fn from(err: toml::de::Error) -> Self {
        S3AccessError::Parse(err)
    }
}