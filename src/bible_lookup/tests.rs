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
fn test_to_json() {
    let text = fetch_ref("John3:16-17").unwrap();
    let json_res = to_json(&text);
    assert!(json_res.is_ok());
}

#[test]
fn test_extract_passage() {
    let text = fetch_ref("John3:16-17").unwrap();
    let mut json = to_json(&text).unwrap();
    let passage = extract_passage(&mut json);
    assert!(passage.is_some());
}

#[test]
fn test_extract_passage_info() {
    let text = fetch_ref("John3:16-17").unwrap();
    let mut json = to_json(&text).unwrap();

    println!("{:?}", json);

    let passage_info = extract_passage_info(&mut json);
    assert!(passage_info.is_some());
}

#[test]
fn test_refs_to_passages() {
    let passages = refs_to_passages(vec!["John3:16-17", "1Corinthians13"]);
    let res: Vec<Option<Passage>> = passages
        .into_iter()
        .filter(|passage| passage.is_none())
        .collect();
    assert!(res.is_empty());
}

#[test]
fn test_build_reply() {
    let text = fetch_ref("John3:16-17").unwrap();
    let mut json = to_json(&text).unwrap();
    let passage = extract_passage(&mut json).unwrap();
    let passage_info = extract_passage_info(&mut json).unwrap();
    let reply = build_reply(passage_info, passage);
    assert!(!reply.is_empty());
}
