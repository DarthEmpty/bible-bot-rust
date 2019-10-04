#[cfg(test)]
use super::*;

#[test]
fn test_connect_to_bucket() {
    assert!(connect_to_bucket().is_ok());
}

#[test]
fn test_load_config() {
    let bucket = connect_to_bucket().unwrap();
    let config = load_config(&bucket);

    println!("{:?}", config);

    assert!(config.is_ok());
}