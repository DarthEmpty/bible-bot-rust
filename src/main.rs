#![warn(clippy::all, clippy::pedantic)]
mod bible_lookup;
mod s3_access;

use bounded_vec_deque::BoundedVecDeque;
use orca::{data::Comment, data::Listing, App};
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

fn get_comments(reddit: &App) -> Listing<Comment> {
    let sub = env!("SUBREDDIT");
    let comment_limit: i32 = env!("COMMENT_LIMIT").parse().unwrap_or(100);

    // TODO: Use the 'before' argument
    reddit
        .get_recent_comments(sub, Some(comment_limit), None)
        .expect("Could not retrieve comments")
}

fn respond_to_comment(comment: Comment, reddit: &App) -> Option<String> {
    // TODO: Change extract refs to return Option<Vec> or Result<Vec, E>
    let refs = bible_lookup::extract_refs(&comment.body);

    // TODO: Then this can go
    if refs.is_empty() {
        return None;
    }

    let passage_pairs = bible_lookup::refs_to_passage_pairs(refs);
    let reply_body = bible_lookup::build_replies(passage_pairs);

    match reddit.comment(&reply_body, &comment.name) {
        Ok(_) => Some(comment.id),
        // TODO: Silently fails
        Err(_) => None,
    }
}

fn main() {
    const ID_DEQUE_CAPACITY: usize = 500;

    let bucket = s3_access::create_bucket().expect("Could not create bucket");

    let config = s3_access::load_config(&bucket).expect("Could not load config");
    let reddit = create_app(&config);

    let comments = get_comments(&reddit);
    let read_comment_ids = s3_access::load_comment_ids(&bucket).unwrap_or_default();
    let mut id_queue = BoundedVecDeque::from_iter(read_comment_ids, ID_DEQUE_CAPACITY);
    let comment_ids_responded_to: Vec<_> = comments
        .filter_map(|comment| {
            if (&mut id_queue).contains(&comment.id) {
                return None;
            }
            respond_to_comment(comment, &reddit)
        })
        .collect();

    id_queue.extend(comment_ids_responded_to);

    s3_access::save_comment_ids(Vec::from(id_queue.into_unbounded()), &bucket)
        .expect("Could not save comment ids");
}
