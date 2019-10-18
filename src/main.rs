#![warn(clippy::all, clippy::pedantic)]
mod bible_lookup;
mod err;
mod s3_access;

use err::{BibleBotError, BibleBotResult};
use failure;
use log::{error, info, warn};
use log_panics;
use orca::{data::Comment, App};
use s3::bucket::Bucket;
use s3_access::config::Config;
use simplelog::{self, LevelFilter, WriteLogger};
use std::{
    fs::{write, OpenOptions},
    thread::sleep,
    time::Duration,
};

fn setup_logging(filename: &str) {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)
        .unwrap();

    WriteLogger::init(LevelFilter::Info, simplelog::Config::default(), file).unwrap();

    log_panics::init();
}

fn create_app(config: &Config) -> BibleBotResult<App> {
    let mut app = App::new(&config.app_name, &config.version, &config.author)?;

    app.authorize_script(
        &config.client_id,
        &config.client_secret,
        &config.username,
        &config.password,
    )?;

    Ok(app)
}

fn try_and_retry_response(
    comment: &Comment,
    body: &str,
    reddit: &App,
    tries: usize,
) -> BibleBotResult<()> {
    match reddit.comment(body, &comment.name) {
        Err(e) => {
            if tries == 0 {
                Err(BibleBotError::from(e))
            } else {
                warn!("Failed to respond to {}, retrying...", &comment.name);
                try_and_retry_response(comment, body, reddit, tries - 1)
            }
        }
        _ => Ok(()),
    }
}

fn respond_to_comment(comment: &Comment, reddit: &App) -> BibleBotResult<()> {
    let refs = bible_lookup::extract_refs(&comment.body)?;

    let passage_pairs = bible_lookup::lookup_refs(refs);
    let reply_body = bible_lookup::build_replies(&passage_pairs);

    try_and_retry_response(comment, &reply_body, reddit, 5)
}

fn try_and_retry_save(
    filename: &str,
    content: &str,
    content_type: &str,
    bucket: &Bucket,
    tries: usize,
) -> BibleBotResult<()> {
    match s3_access::save_file(filename, content, content_type, bucket) {
        Err(e) => {
            if tries == 0 {
                Err(BibleBotError::from(e))
            } else {
                warn!("Failed to save {}, retrying...", filename);
                try_and_retry_save(filename, content, content_type, bucket, tries - 1)
            }
        }
        _ => Ok(()),
    }
}

fn save_bookmark(filename: &str, content: &str, bucket: &Bucket) {
    if let Err(BibleBotError::Storage(e)) =
        try_and_retry_save(filename, content, "text/plain", &bucket, 5)
    {
        error!("{}", e);
        info!("Writing bookmark to local {}...", filename);
        if let Err(e) = write(filename, content) {
            error!("{}", e);
        };
    };
}

fn pulse(sub: &str, comment_limit: i32, bookmark_file: &str, bucket: &Bucket, reddit: &App) {
    info!("Loading last read comment (bookmark)...");
    let bookmark_name = if let Ok(name) = s3_access::load_file(bookmark_file, &bucket) {
        info!("Bookmark found: {}", name);
        name
    } else {
        warn!("No bookmark found.");
        String::default()
    };

    info!("Loading most recent comments from {}...", sub);
    let comments = reddit
        .get_recent_comments(sub, Some(comment_limit), Some(&bookmark_name))
        .expect("Could not retrieve comments");
    info!("Responding to comments...");
    comments.enumerate().for_each(|(i, c)| {
        if i == 0 {
            info!("New bookmark found: {}", c.name);
            save_bookmark(bookmark_file, &c.name, bucket)
        }

        match respond_to_comment(&c, &reddit) {
            Err(BibleBotError::Lookup(e)) => warn!("{}: {}", c.name, e),
            Err(BibleBotError::RedditResponse(e)) => error!("{}: {}", c.name, e),
            _ => info!("{}: Successfully responded to!", c.name),
        }
    });

    info!("----- Done! -----")
}

fn main() {
    let sub: &str = env!("SUBREDDITS");
    let limit: i32 = env!("COMMENT_LIMIT").parse().unwrap_or(100);
    let bm_file: &str = env!("BOOKMARK_FILE");
    let log_filename: &str = env!("LOG_FILE");
    setup_logging(log_filename);

    info!("+++++ INITIALISING +++++");
    info!("Connecting to S3 bucket...");
    let bucket = s3_access::connect_to_bucket().expect("Could not connect to bucket");

    info!("Creating instance of 'Bible Bot'...");
    let config = s3_access::load_config(&bucket).expect("Could not load config");
    let reddit = create_app(&config).expect("Could not create App instance");

    info!("+++++ START MAIN LOOP +++++");

    loop {
        pulse(sub, limit, bm_file, &bucket, &reddit);
        sleep(Duration::from_millis(500));
    }
}
