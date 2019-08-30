use s3::{self, bucket::Bucket, credentials::Credentials, error::S3Result, region::Region};
use serde_json;
use std::collections::HashMap;
use toml;

pub fn create_bucket() -> S3Result<Bucket> {
    const NAME: &str = "bible-bot";
    const REGION: Region = Region::EuWest2;

    Bucket::new(NAME, REGION, Credentials::default())
}

fn get_from_bucket(filename: &str, bucket: &Bucket) -> Option<String> {
  let (data, code) = bucket.get_object(filename).unwrap_or_default();
  match code {
    200 => Some(String::from_utf8(data).unwrap_or_default()),
    _ => None,
  }
}

pub fn load_config(bucket: &Bucket) -> Option<HashMap<String, String>> {
  const CONFIG_FILE: &str = "config.toml";
  get_from_bucket(CONFIG_FILE, bucket).and_then(|s| toml::from_str(&s).ok())
}

pub fn load_past_comments(bucket: &Bucket) -> Option<Vec<String>> {
  const FILE: &str = "past_comments.json";
  get_from_bucket(FILE, bucket).and_then(|s| serde_json::from_str(&s).ok())
}
