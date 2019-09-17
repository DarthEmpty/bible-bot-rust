pub mod config;
mod constants;
mod tests;

use config::Config;
use s3::{self, bucket::Bucket, credentials::Credentials, error::S3Result};
use toml;

pub fn connect_to_bucket() -> S3Result<Bucket> {
    Bucket::new(constants::NAME, constants::REGION, Credentials::default())
}

pub fn load_file(filename: &str, bucket: &Bucket) -> Option<String> {
    let (data, code) = bucket.get_object(filename).unwrap_or_default();
    match code {
        200 => Some(String::from_utf8(data).unwrap_or_default()),
        _ => None,
    }
}

pub fn save_file(
    filename: &str,
    content: &str,
    content_type: &str,
    bucket: &Bucket,
) -> Result<(), &'static str> {
    let (_, code) = bucket
        .put_object(filename, content.as_bytes(), content_type)
        .unwrap_or_default();
    match code {
        200 => Ok(()),
        _ => Err("Unsuccessful post to bucket"),
    }
}

pub fn load_config(bucket: &Bucket) -> Option<Config> {
    load_file(constants::CONFIG_FILE, bucket).and_then(|s| toml::from_str(&s).ok())
}
