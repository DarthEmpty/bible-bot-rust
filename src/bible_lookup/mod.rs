mod passage;
mod tests;

use passage::*;
use reqwest;
use regex::Regex;
use serde_json::{self, Value};


fn extract_refs(text: &str) -> Vec<String> {
    // Matches with:
    // <book><chapter> (book may have digit as prefix)
    // <book><chapter>:<verse>
    // <book><chapter>:<verse>-<verse>
    const PATTERN_STRING: &str = r"\[\[(\d?[a-zA-Z]+\d+(?::\d+(?:-\d+)?)?)\]\]";
    
    let pattern = Regex::new(PATTERN_STRING).unwrap();
    let string = text.replace(" ", "").replace("\\", "");

    pattern
        .captures_iter(&string)
        .map(|cap| 
            String::from(cap.get(1).unwrap().as_str())
        )
        .collect()
}

fn fetch_ref(reference: &str) -> Result<String, reqwest::Error> {
    let url = format!("https://getbible.net/json?text={}", reference);
    let text: String = reqwest
        ::get(&url)?
        .text()?
        .replace("(", "")
        .replace(");", "");

    Ok(text)
}

fn to_json(text: &str) -> serde_json::Result<Value> {
    serde_json::from_str(text)
} 

fn extract_passage(json: &mut Value) -> Option<Passage> {
    match json["type"].as_str().unwrap_or_default() {
        "chapter" => Some(Passage::from(
            json["chapter"].take()
        )),
        "verse" => Some(Passage::from(
            json["book"][0]["chapter"].take()
        )),
        _ => None
    }
}

fn extract_passage_info(json: &mut Value) -> Option<PassageInfo> {
    match json["type"].as_str().unwrap_or_default() {
        "chapter" => Some(PassageInfo::new(
            json["book_name"].take(),
            json["chapter_nr"].take(),
            json["version"].take()
        )),
        "verse" => Some(PassageInfo::new(
            json["book"][0]["book_name"].take(),
            json["book"][0]["chapter_nr"].take(),
            json["version"].take()
        )),
        _ => None
    }
}

fn refs_to_passages(refs: Vec<&str>) -> Vec<Option<Passage>> {
    refs
        .into_iter()
        .map(|reference| {
            let text = fetch_ref(&reference).unwrap_or_default();
            let mut json = to_json(&text).unwrap_or_default();

            extract_passage(&mut json)
        })
        .collect()
}

fn build_reply(info: PassageInfo, passage: Passage) -> String {
    format!("{}\n\n{}", info.to_string(), passage.to_string())
}
