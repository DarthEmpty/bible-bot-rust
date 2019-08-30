mod bible_lookup;
mod s3_access;

use bounded_vec_deque::BoundedVecDeque;
use orca::{data::Comment, data::Listing, App};
use std::collections::HashMap;

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
    let bucket = s3_access::create_bucket().expect("Could not create bucket");

    let config = s3_access::load_config(&bucket).expect("Could not load config");
    let reddit = create_app(config);

    let comments = get_comments(&reddit);
    let mut past_comments = BoundedVecDeque::from_iter(s3_access::load_past_comments(&bucket).unwrap_or_default(), 500);

    past_comments.extend(comments.filter_map(|comment| { 
        let refs = bible_lookup::extract_refs(&comment.body);

        if refs.is_empty() {
            return None;
        }

        let passage_pairs = bible_lookup::refs_to_passage_pairs(refs);
        let reply_body = bible_lookup::build_replies(passage_pairs);

        reddit.comment(&reply_body, &comment.id).ok().map(|_| String::from(comment.id))
    }));
    
    s3_access::save_past_comments(Vec::from(past_comments.into_unbounded()), &bucket);    
}
