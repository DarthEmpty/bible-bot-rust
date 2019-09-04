use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub username: String,
    pub password: String,
    pub app_name: String,
    pub version: String,
    pub author: String,
}