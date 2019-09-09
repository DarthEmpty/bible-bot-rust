#![warn(clippy::all, clippy::pedantic)]
mod bible_lookup;
mod s3_access;

use failure::Error;
use orca::{data::Comment, App};
use s3_access::config::Config;

// TODO: Do some logging

fn create_app(config: &Config) -> App {
    let mut app = App::new(&config.app_name, &config.version, &config.author)
        .expect("Could not create Reddit instance");

    app.authorize_script(
        &config.client_id,
        &config.client_secret,
        &config.username,
        &config.password,
    )
    .expect("Could not authorize script");

    app
}

fn respond_to_comment(comment: &Comment, reddit: &App) -> Result<(), Error> {
    // TODO: Change extract refs to return Option<Vec> or Result<Vec, E>
    let refs = bible_lookup::extract_refs(&comment.body);

    // TODO: Handle result from extract refs

    let passage_pairs = bible_lookup::refs_to_passage_pairs(refs);
    let reply_body = bible_lookup::build_replies(passage_pairs);

    reddit.comment(&reply_body, &comment.name)
}

fn main() {
    let sub = env!("SUBREDDIT");
    let limit: i32 = env!("COMMENT_LIMIT").parse().unwrap_or(100);
    let bm_file = env!("BOOKMARK_FILE");

    let bucket = s3_access::connect_to_bucket().expect("Could not connect to bucket");

    let config = s3_access::load_config(&bucket).expect("Could not load config");
    let reddit = create_app(&config);

    let bookmark_name = s3_access::load_file(bm_file, &bucket).unwrap_or_default();
    let mut new_bookmark_name = String::default();

    let comments = reddit
        .get_recent_comments(sub, Some(limit), Some(&bookmark_name))
        .expect("Could not retrieve comments");
    
    comments.enumerate().for_each(|(i, c)| {
        if i == 0 {
            new_bookmark_name = c.name.clone();
        }

        respond_to_comment(&c, &reddit);
    });

    s3_access::save_file(bm_file, &new_bookmark_name, "text/plain", &bucket);
}
