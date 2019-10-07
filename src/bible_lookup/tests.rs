#![allow(dead_code)]
#[cfg(test)]
use super::*;

static GOOD_EXAMPLE: &str = "John3:16-17";
static OTHER_GOOD_EXAMPLE: &str = "1Corinthians13";

#[test]
fn test_extract_refs_ok() {
    let refs = extract_refs(
        "I wanna look at [[John 3: 16 - 17]]
        and I wanna look at [[1 Corinthians 13]]",
    );

    assert_eq!(refs.unwrap(), vec!["John3:16-17", "1Corinthians13"]);
}

#[test]
fn test_extract_refs_err() {
    let no_refs = extract_refs(
        "You have no references here, Gandalf the Grey!
        [[Noteventhisone!!]]
        Or even this 'valid' one! [[Jude]]"
    );

    assert!(no_refs.is_err());
}

#[test]
fn test_fetch_ref() {
    let text_res = fetch_ref(GOOD_EXAMPLE);
    assert!(text_res.is_ok());
}

#[test]
fn test_passage_constructor_ok() {
    let text = fetch_ref(GOOD_EXAMPLE).unwrap();
    let json = serde_json::from_str(&text).unwrap();
    let passage = Passage::new(&json);
    assert!(passage.is_ok());
}

#[test]
fn test_info_constructor_ok() {
    let text = fetch_ref(GOOD_EXAMPLE).unwrap();
    let json = serde_json::from_str(&text).unwrap();
    let info = Info::new(&json);
    assert!(info.is_ok());
}

#[test]
fn test_lookup_refs() {
    let passage_map = lookup_refs(vec![GOOD_EXAMPLE.into(), OTHER_GOOD_EXAMPLE.into()]);
    assert_eq!(
        passage_map
            .values()
            .filter(|v| v.is_ok())
            .count(), 2
    );
}

#[test]
fn test_build_replies() {
    let refs = vec![GOOD_EXAMPLE.into(), OTHER_GOOD_EXAMPLE.into()];
    let map = lookup_refs(refs);
    let replies = build_replies(&map);
    assert!(!replies.contains("Error finding"));
}
