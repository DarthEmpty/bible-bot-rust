mod constants;
pub mod err;
mod passage;
mod tests;

use err::{BibleLookupError, BibleLookupResult};
use log::error;
use passage::*;
use regex::Regex;
use reqwest;
use serde_json;
use std::collections::HashMap;

pub fn extract_refs(text: &str) -> BibleLookupResult<Vec<String>> {
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
        Err(BibleLookupError::NoRefs)
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

fn to_passage_pair(reference: &str) -> BibleLookupResult<(Info, Passage)> {
    let text = fetch_ref(&reference)?;
    let json = serde_json::from_str(&text)?;

    Ok((Info::new(&json)?, Passage::new(&json)?))
}

pub fn lookup_refs(refs: Vec<String>) -> HashMap<String, BibleLookupResult<(Info, Passage)>> {
    refs.into_iter()
        .map(|reference| (reference.clone(), to_passage_pair(&reference)))
        .collect()
}

pub fn build_replies(passage_map: &HashMap<String, BibleLookupResult<(Info, Passage)>>) -> String {
    passage_map
        .keys()
        .map(|key| match &passage_map[key] {
            Ok((info, passage)) => format!("{}\n\n{}", info.to_string(), passage.to_string()),
            Err(e) => {
                error!("[{}]: {}", key, e);
                format!("Error finding {}", key)
            }
        })
        .collect::<Vec<String>>()
        .join("\n\n___\n\n")
}
