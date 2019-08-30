use s3::{self, bucket::Bucket};
use serde_json;
use std::collections::HashMap;
use toml;

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

pub fn load_read_comments(bucket: &Bucket) -> Option<Vec<String>> {
  const READ_COMMENTS_FILE: &str = "read_comments.json";
  get_from_bucket(READ_COMMENTS_FILE, bucket).and_then(|s| serde_json::from_str(&s).ok())
}
