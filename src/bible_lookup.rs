use std::collections::HashMap;
use reqwest;
use regex::Regex;
use serde::Deserialize;
use serde_json::{self, Value};

// Hashmap structure:
// Passage({
//     <verse-no> : { 
//         "verse_nr" : Number | String,
//         "verse" : String
// })
#[derive(Default, Deserialize, Debug)]
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
        .map(|cap| 
            String::from(cap.get(1).unwrap().as_str())
        )
        .collect()
}

fn get_ref(reference: &str) -> Result<String, reqwest::Error> {
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

fn refs_to_passages(refs: Vec<&str>) -> Vec<Option<Passage>> {
    refs
        .into_iter()
        .map(|reference| {
            let text = get_ref(&reference).unwrap_or_default();
            
            to_passage(&text)
        })
        .collect()
}

fn build_reply(book: &str, chapter: &str, passage: Passage, version: &str) -> String {
    let mut reply = format!("{} {} ({})\n\n", book, chapter, version);
    let mut keys: Vec<u8> = passage.0
        .keys()
        .map(|k|
            k.parse::<_>().unwrap_or_default()
        )
        .collect();

    keys.sort();
    
    for key in keys {
        let k = key.to_string();
        let verse = passage.0
            .get(&k)
            .and_then(|v|
                v.get("verse")
            )
            .and_then(|verse_value|
                verse_value.as_str()
            );

        let formed_verse = format!("^({}) {}", k, verse.unwrap_or_default());
        reply.push_str(&formed_verse);
    }

    reply
}

#[cfg(test)]
mod tests {
    // TODO: Improve these to be more thorough
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
        let text = get_ref("John3:16-17");
        assert!(text.is_ok());
    }

    #[test]
    fn test_to_passage() {
        let text = get_ref("John3:16-17");
        let passage = to_passage(&text.unwrap());
        assert!(passage.is_some());
    }

    #[test]
    fn test_lookup_refs() {
        let passages = refs_to_passages(vec!["John3:16-17", "1Corinthians13"]);
        let res: Vec<Option<Passage>> = passages.into_iter().filter(|passage| passage.is_none()).collect();
        assert!(res.is_empty());
    }

    #[test]
    fn test_build_reply() {
        let text = get_ref("John3:16-17");
        let passage = to_passage(&text.unwrap());
        let reply = build_reply("John", "3", passage.unwrap(), "kjv");
        assert!(!reply.is_empty());
        println!("{}", reply);
    }
}
