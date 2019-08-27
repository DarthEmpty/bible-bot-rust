mod bible_lookup;

use orca::{data::Comment, data::Listing, App};
use std::collections::HashMap;
use std::fs::read_to_string;
use toml;

fn read_config() -> HashMap<String, String> {
    const CONFIG_FILE: &str = "src/config.toml";
    let toml = read_to_string(CONFIG_FILE).expect("No file found");
    toml::from_str(&toml).expect("Could not deserialize file")
}

fn create_app(config: HashMap<String, String>) -> App {
    let mut app = App::new(&config["app_name"], &config["version"], &config["author"])
        .expect("Could not create Reddit instance");

    app
        .authorize_script(&config["client_id"], &config["client_secret"], &config["username"], &config["password"])
        .expect("Could not authorize script");

    app
}

fn get_comments(reddit: &App) -> Listing<Comment> {
    const SUB: &str = "pythonforengineers";
    const COMMENT_LIMIT: i32 = 100;

    reddit
        .get_recent_comments(SUB, Some(COMMENT_LIMIT), None)
        .expect("Could not retrieve comments")
}

fn main() {
    let config = read_config();
    let reddit = create_app(config);

    let mut comments = get_comments(&reddit);
    println!("{:?}", comments.next());
}
