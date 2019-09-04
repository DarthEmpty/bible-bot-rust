#[cfg(test)]
// TODO: Improve these to be more thorough
use super::*;

#[test]
fn test_load_config() {
    let bucket = create_bucket().unwrap();
    let config = load_config(&bucket);

    println!("{:?}", config);

    assert!(config.is_some());
}