mod bible_lookup;

use orca::{data::Comment, data::Listing, App};
use serde_json;
use std::collections::HashMap;
use std::fs::{read_to_string, write};
use toml;

fn load_config() -> HashMap<String, String> {
    const CONFIG_FILE: &str = "src/config.toml";
    let toml = read_to_string(CONFIG_FILE).expect("No file found");
    toml::from_str(&toml).expect("Could not deserialize file")
}

fn load_read_comments() -> Vec<String> {
    const READ_COMMENTS_FILE: &str = "src/read_comments.json";
    let json = read_to_string(READ_COMMENTS_FILE).expect("No file found");
    serde_json::from_str(&json).expect("Could not deserialize file")
}

fn save_read_comments(comments: Vec<String>) {
    const READ_COMMENTS_FILE: &str = "src/read_comments.json";
    let json = serde_json::to_string(&comments).expect("Could not serealize comments");
    write(READ_COMMENTS_FILE, json).expect("Could not save comments");
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
    let config = load_config();
    let reddit = create_app(config);

    let comments = get_comments(&reddit);

    let read_comments: Vec<String> = comments.filter_map(|comment| { 
        let refs = bible_lookup::extract_refs(&comment.body);

        if refs.is_empty() {
            return None;
        }

        let passage_pairs = bible_lookup::refs_to_passage_pairs(refs);
        let reply_body = bible_lookup::build_replies(passage_pairs);

        reddit.comment(&reply_body, &comment.id).ok().map(|_| comment.id)
    })
    .collect();
    
    save_read_comments(read_comments);    
}
