mod constants;
mod passage;
mod tests;

use crate::err::{BibleBotError, BibleBotResult};
use passage::*;
use regex::Regex;
use reqwest;
use serde_json;

pub fn extract_refs(text: &str) -> BibleBotResult<Vec<String>> {
    // Matches with:
    // <book><chapter> (book may have digit as prefix)
    // <book><chapter>:<verse>
    // <book><chapter>:<verse>-<verse>
    let pattern = Regex::new(constants::REFERENCE_PATTERN).unwrap();
    let string = text.replace(" ", "").replace("\\", "");

    let res: Vec<String> = pattern
        .captures_iter(&string)
        .filter_map(|cap| cap.get(1).and_then(|m| Some(String::from(m.as_str()))))
        .collect();

    if res.is_empty() {
        Err(BibleBotError::NoRefs)
    } else {
        Ok(res)
    }
}

fn fetch_ref(reference: &str) -> Result<String, reqwest::Error> {
    let url = format!("{}{}", constants::BIBLE_API_URL, reference);
    let text: String = reqwest::get(&url)?
        .text()?
        .replace("(", "")
        .replace(");", "");

    Ok(text)
}

pub fn refs_to_passage_pairs(refs: Vec<String>) -> Vec<BibleBotResult<(Info, Passage)>> {
    refs.into_iter()
        .map(|reference| {
            let text = fetch_ref(&reference)?;
            let json = serde_json::from_str(&text)?;

            Ok((Info::new(&json)?, Passage::new(&json)?))
        })
        .collect()
}

fn build_reply(info: &Info, passage: &Passage) -> String {
    format!("{}\n\n{}", info.to_string(), passage.to_string())
}

pub fn build_replies(passage_pairs: Vec<BibleBotResult<(Info, Passage)>>) -> String {
    passage_pairs
        .into_iter()
        .map(|pair| match pair {
            Ok(v) => build_reply(&v.0, &v.1),
            Err(e) => format!("{}", e),
        })
        .collect::<Vec<String>>()
        .join("\n---\n")
}
