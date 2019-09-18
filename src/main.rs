#![warn(clippy::all, clippy::pedantic)]
mod bible_lookup;
mod err;
mod s3_access;

use err::{BibleBotResult, BibleBotError};
use failure;
use log::{info, warn, error};
use orca::{data::Comment, App};
use s3_access::config::Config;

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

fn try_and_retry_response(comment: &Comment, body: &str, reddit: &App, tries: usize) -> BibleBotResult<()> {
    match reddit.comment(body, &comment.name) {
        Err(res) => {
            if tries == 0 {
                Err(BibleBotError::from(res))
            } else {
                warn!("Failed to respond to {}, retrying...", &comment.name);
                try_and_retry_response(comment, body, reddit, tries - 1)
            }
        }
        _ => Ok(())
    }
}

fn respond_to_comment(comment: &Comment, reddit: &App) -> BibleBotResult<()> {
    let refs = bible_lookup::extract_refs(&comment.body)?;

    let passage_pairs = bible_lookup::refs_to_passage_pairs(refs);
    let reply_body = bible_lookup::build_replies(passage_pairs);

    try_and_retry_response(comment, &reply_body, reddit, 5)
}

fn main() {
    let sub = env!("SUBREDDIT");
    let limit: i32 = env!("COMMENT_LIMIT").parse().unwrap_or(100);
    let bm_file = env!("BOOKMARK_FILE");

    info!("Connecting to S3 bucket...");
    let bucket = s3_access::connect_to_bucket().expect("Could not connect to bucket");

    info!("Creating instance of 'Bible Bot'...");
    let config = s3_access::load_config(&bucket).expect("Could not load config");
    let reddit = create_app(&config);

    info!("Loading last read comment (bookmark)...");
    let bookmark_name = if let Ok(name) = s3_access::load_file(bm_file, &bucket) {
        info!("Bookmark found: {}", name);
        name
    } else {
        warn!("No bookmark found.");
        String::default()
    };

    let mut new_bookmark_name = String::default();

    info!("Loading most recent comments from {}...", sub);
    let comments = reddit
        .get_recent_comments(sub, Some(limit), Some(&bookmark_name))
        .expect("Could not retrieve comments");
    
    info!("Responding to comments...");
    comments.enumerate().for_each(|(i, c)| {
        if i == 0 {
            info!("New bookmark found: {}", c.name);
            new_bookmark_name = c.name.clone();
        }

        match respond_to_comment(&c, &reddit) {
            Err(BibleBotError::Lookup(e)) => warn!("{}: {}", c.name, e),
            Err(BibleBotError::RedditResponse(e)) => error!("{}: {}", c.name, e),
            Ok(_) => info!("{}: Successfully responded to!", c.name),
        }
    });

    s3_access::save_file(bm_file, &new_bookmark_name, "text/plain", &bucket);
}
