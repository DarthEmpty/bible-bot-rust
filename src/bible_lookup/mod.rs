mod constants;
mod passage;
mod tests;

use passage::*;
use regex::Regex;
use reqwest;
use serde_json::{self, Value};

pub fn extract_refs(text: &str) -> Vec<String> {
    // Matches with:
    // <book><chapter> (book may have digit as prefix)
    // <book><chapter>:<verse>
    // <book><chapter>:<verse>-<verse>
    let pattern = Regex::new(constants::REFERENCE_PATTERN).unwrap();
    let string = text.replace(" ", "").replace("\\", "");

    pattern
        .captures_iter(&string)
        .map(|cap| String::from(cap.get(1).unwrap().as_str()))
        .collect()
}

fn fetch_ref(reference: &str) -> Result<String, reqwest::Error> {
    let url = format!("{}{}", constants::BIBLE_API_URL, reference);
    let text: String = reqwest::get(&url)?
        .text()?
        .replace("(", "")
        .replace(");", "");

    Ok(text)
}

// TODO: Is this best as a part of a constructor on your passage struct?
fn extract_passage(json: &Value) -> Option<Passage> {
    match json["type"].as_str().unwrap_or_default() {
        "chapter" => Some(Passage::from(json["chapter"].clone())),
        "verse" => Some(Passage::from(json["book"][0]["chapter"].clone())),
        _ => None,
    }
}


// TODO: Is this best on your passage struct?
fn extract_passage_info(json: &Value) -> Option<Info> {
    match json["type"].as_str().unwrap_or_default() {
        "chapter" => Some(Info::new(
            // TODO: Consider using .to_string() or some other strong typed value enum(?)
            &json["book_name"].clone(),
            &json["chapter_nr"].clone(),
            &json["version"].clone(),
        )),
        "verse" => Some(Info::new(
            &json["book"][0]["book_name"].clone(),
            &json["book"][0]["chapter_nr"].clone(),
            &json["version"].clone(),
        )),
        _ => None,
    }
}

pub fn refs_to_passage_pairs(refs: Vec<String>) -> Vec<Option<(Info, Passage)>> {
    refs.into_iter()
        .map(|reference| {
            let text = fetch_ref(&reference).unwrap_or_default();
            let json = serde_json::from_str(&text).unwrap_or_default();

            let passage_info = extract_passage_info(&json);
            let passage = extract_passage(&json);

            if (passage_info.is_none()) || (passage.is_none()) {
                return None;
            }
            Some((passage_info.unwrap(), passage.unwrap()))
        })
        .collect()
}

fn build_reply(info: &Info, passage: &Passage) -> String {
    format!("{}\n\n{}", info.to_string(), passage.to_string())
}

pub fn build_replies(passage_pairs: Vec<Option<(Info, Passage)>>) -> String {
    passage_pairs
        .into_iter()
        .map(|pair| {
            // TODO: Use a match
            if pair.is_some() {
                let unwrapped = pair.unwrap();
                return build_reply(&unwrapped.0, &unwrapped.1);
            }

            // TODO: Consider using an error type?
            // TODO: Then you can send an error message back to Reddit for errors of that type rather than having it appear as a success value
            String::from("Could not find requested passage\n\n")
        })
        .collect::<Vec<String>>()
        .join("\n---\n")
}
