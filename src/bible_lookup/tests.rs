#[cfg(test)]
// TODO: Improve these to be more thorough
use super::*;

#[test]
fn test_extract_refs() {
    let refs = extract_refs(
        "I wanna look at [[John 3: 16 - 17]]
        and I wanna look at [[1Corinthians13]]",
    );

    assert_eq!(refs, vec!["John3:16-17", "1Corinthians13"]);
}

#[test]
fn test_fetch_ref() {
    let text_res = fetch_ref("John3:16-17");
    assert!(text_res.is_ok());
}

#[test]
fn test_extract_passage() {
    let text = fetch_ref("John3:16-17").unwrap();
    let mut json = serde_json::from_str(&text).unwrap();
    let passage = extract_passage(&mut json);
    assert!(passage.is_some());
}

#[test]
fn test_extract_passage_info() {
    let text = fetch_ref("John3:16-17").unwrap();
    let mut json = serde_json::from_str(&text).unwrap();
    let passage_info = extract_passage_info(&mut json);
    assert!(passage_info.is_some());
}

#[test]
fn test_refs_to_passages() {
    let passage_pairs = refs_to_passage_pairs(vec!["John3:16-17".into(), "1Corinthians13".into()]);
    let res: Vec<Option<(Info, Passage)>> = passage_pairs
        .into_iter()
        .filter(|pair| pair.is_none())
        .collect();
    assert!(res.is_empty());
}

#[test]
fn test_build_reply() {
    let text = fetch_ref("John3:16-18").unwrap();
    let mut json = serde_json::from_str(&text).unwrap();
    let passage = extract_passage(&mut json).unwrap();
    let passage_info = extract_passage_info(&mut json).unwrap();
    let reply = build_reply(&passage_info, &passage);
    assert!(!reply.is_empty());
}

#[test]
fn test_build_replies() {
    let refs = vec!["John3:16-17".into(), "1Corinthians13".into()];
    let pairs = refs_to_passage_pairs(refs);
    let replies = build_replies(pairs);
    assert!(!replies.contains("Could not find requested passage"));
}
