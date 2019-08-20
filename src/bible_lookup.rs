use std::collections::HashMap;
use reqwest;
use regex::Regex;
use serde::Deserialize;
use serde_json::{self, Value};

#[derive(Deserialize, Debug)]
struct Passage(HashMap<String, HashMap<String, Value>>);

impl From<Value> for Passage {
    fn from(v: Value) -> Passage {
        serde_json::from_value(v).unwrap()
    }
}

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
        .map(|cap| {
            String::from(cap.get(1).unwrap().as_str())
        })
        .collect()
}

fn lookup_ref(reference: &str) -> Result<String, reqwest::Error> {
    let url = format!("https://getbible.net/json?text={}", reference);
    let text: String = reqwest
        ::get(&url)?
        .text()?
        .replace("(", "")
        .replace(");", "");

    Ok(text)
}

fn to_passage(text: &str) -> Option<Passage> {
    let mut json: Value = serde_json::from_str(&text).unwrap_or_default();

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

// fn lookup_refs(refs: Vec<String>) -> Vec<Passage> {
//     refs
//         .into_iter()
//         .map(|reference| {
//             lookup_ref(&reference).unwrap()
//         })
//         .collect()
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_refs() {
        let refs = extract_refs(
            "I wanna look at [[John 3: 16 - 17]]
            and I wanna look at [[1Corinthians13]]",
        );

        assert_eq!(
            refs,
            vec!["John3:16-17", "1Corinthians13"]
        );
    }

    #[test]
    fn test_lookup_ref() {
        let text = lookup_ref("John3:16-17");
        assert!(text.is_ok());
    }

    #[test]
    fn test_to_passage() {
        let text = lookup_ref("John3:16-17");
        let passage = to_passage(&text.unwrap());
        assert!(passage.is_some())
    }
}
