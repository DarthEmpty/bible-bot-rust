#![warn(clippy::all, clippy::pedantic)]
mod bible_lookup;
mod err;
mod s3_access;

use err::{BibleBotError, BibleBotResult};
use failure;
use log::{debug, error, info, warn};
use log_panics;
use orca::{data::Comment, App};
use s3::bucket::Bucket;
use s3_access::config::Config;
use simplelog::{self, LevelFilter, WriteLogger};
use std::{
    cell::Cell,
    fs::{write, OpenOptions},
    thread::sleep,
    time::Duration,
};

struct AppHandler {
    app: Cell<App>,
    config: Config,
}

impl AppHandler {
    fn build_and_authorise_app(config: &Config) -> BibleBotResult<App> {
        let mut app = App::new(&config.app_name, &config.version, &config.author)?;

        app.authorize_script(
            &config.client_id,
            &config.client_secret,
            &config.username,
            &config.password,
        )?;

        Ok(app)
    }

    pub fn new(config: Config) -> BibleBotResult<Self> {
        let app = Self::build_and_authorise_app(&config)?;
        Ok(Self{ app: Cell::new(app), config })
    }

    pub fn rebuild_app(&self) -> BibleBotResult<()> {
        let new_app = Self::build_and_authorise_app(&self.config)?;
        let old_app = self.app.replace(new_app);
        drop(old_app);
        Ok(())
    }
}

fn setup_logging(filename: &str) {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)
        .unwrap();

    WriteLogger::init(LevelFilter::Info, simplelog::Config::default(), file).unwrap();

    log_panics::init();
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

fn check_and_respond(sub: &str, comment_limit: i32, bookmark_name: &str, mut reddit: AppHandler) -> (AppHandler, Option<String>) {  
    debug!("Loading most recent comments from {}...", sub);
    let comments = reddit.app.get_mut()
        .get_recent_comments(sub, Some(comment_limit), Some(&bookmark_name))
        .expect("Could not retrieve comments");
    debug!("Responding to comments...");
    let mut new_bookmark = String::default();
    comments.enumerate().for_each(|(i, c)| {
        if i == 0 {
            new_bookmark = c.name.clone();
        }

        match respond_to_comment(&c, &reddit.app.get_mut()) {
            Err(BibleBotError::Lookup(e)) => warn!("{}: {}", c.name, e),
            Err(BibleBotError::RedditResponse(e)) => {
                warn!("Reauthorising script and trying again...");
                if reddit.rebuild_app().is_ok() && respond_to_comment(&c, &reddit.app.get_mut()).is_ok() {
                    info!("{}: Response successful after reauthorising", c.name)
                } else {
                    error!("{}: {}", c.name, e)
                }
            },
            _ => info!("{}: Successfully responded to!", c.name),
        }
    });

    if new_bookmark.is_empty() {
        (reddit, None)
    } else {
        (reddit, Some(new_bookmark))
    }
}

fn main() {
    // Read environment variables and set up logger
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
    let mut reddit = AppHandler::new(config).expect("Could not create App instance");

    info!("Loading last read comment (bookmark)...");
    let mut bm = if let Ok(name) = s3_access::load_file(bm_file, &bucket) {
        info!("Bookmark found: {}", name);
        name
    } else {
        info!("No bookmark found.");
        String::default()
    };

    info!("+++++ START MAIN LOOP +++++");

    loop {
        let bm_res = check_and_respond(sub, limit, &bm, reddit);
        reddit = bm_res.0;
        if let Some(new_bm) = bm_res.1 {
            if bm != new_bm {
                save_bookmark(bm_file, &new_bm, &bucket);
                bm = new_bm;
            }
        }
        
        debug!("----- Done! -----");
        sleep(Duration::from_millis(500));
    }
}
