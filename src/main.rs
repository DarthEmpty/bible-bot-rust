mod bible_lookup;
mod s3_access;

use orca::{data::Comment, data::Listing, App};
use s3::{self, bucket::Bucket, credentials::Credentials, error::S3Result, region::Region};
use serde_json;
use std::collections::HashMap;
use std::fs::write;

fn save_read_comments(comments: Vec<String>) {
    const READ_COMMENTS_FILE: &str = "src/read_comments.json";
    let json = serde_json::to_string(&comments).expect("Could not serialize comments");
    write(READ_COMMENTS_FILE, json).expect("Could not save comments");
}

fn create_bucket() -> S3Result<Bucket> {
    const NAME: &str = "bible-bot";
    const REGION: Region = Region::EuWest2;

    Bucket::new(NAME, REGION, Credentials::default())
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
    let bucket = create_bucket().expect("Could not create bucket");

    let config = s3_access::load_config(bucket);
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
