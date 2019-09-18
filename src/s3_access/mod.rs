pub mod config;
pub mod err;
mod constants;
mod tests;

use config::Config;
use err::{S3AccessError, S3AccessResult};
use s3::{self, bucket::Bucket, credentials::Credentials, error::S3Result};
use toml;

pub fn connect_to_bucket() -> S3Result<Bucket> {
    Bucket::new(constants::NAME, constants::REGION, Credentials::default())
}

pub fn load_file(filename: &str, bucket: &Bucket) -> S3AccessResult<String> {
    let (data, code) = bucket.get_object(filename).unwrap_or_default();
    match code {
        200 => Ok(String::from_utf8(data).unwrap_or_default()),
        _ => Err(S3AccessError::Load(filename.into())),
    }
}

pub fn save_file(
    filename: &str,
    content: &str,
    content_type: &str,
    bucket: &Bucket,
) -> S3AccessResult<()> {
    let (_, code) = bucket
        .put_object(filename, content.as_bytes(), content_type)
        .unwrap_or_default();
    match code {
        200 => Ok(()),
        _ => Err(S3AccessError::Save(filename.into())),
    }
}

pub fn load_config(bucket: &Bucket) -> S3AccessResult<Config> {
    let s = load_file(constants::CONFIG_FILE, bucket)?;
    Ok(toml::from_str(&s)?)
}
