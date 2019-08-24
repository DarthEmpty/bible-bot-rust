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

impl ToString for Passage {
    fn to_string(&self) -> String {
        let mut string = String::new();
        let mut keys: Vec<u8> = self.0
            .keys()
            .map(|k|
                k.parse::<_>().unwrap_or_default()
            )
            .collect();

        keys.sort();

        for key in keys {
            let k = key.to_string();
            let verse = self.0
                .get(&k)
                .and_then(|v|
                    v.get("verse")
                )
                .and_then(|verse_value|
                    verse_value.as_str()
                );

            let formed_verse = format!("^({}) {}", k, verse.unwrap_or_default());
            string.push_str(&formed_verse);
        }

        string
    }
}

struct PassageInfo {
    book: String,
    chapter: String,
    version: String
}

impl PassageInfo {
    fn new(book: Value, chapter: Value, version: Value) -> PassageInfo {
        PassageInfo {
            book: String::from(book.as_str().unwrap()),
            chapter: String::from(chapter.as_str().unwrap()),
            version: String::from(version.as_str().unwrap())
        }
    }
}

impl ToString for PassageInfo {
    fn to_string(&self) -> String {
        format!("{} {} ({})", self.book, self.chapter, self.version)
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
    fn test_fetch_ref() {
        let text_res = fetch_ref("John3:16-17");
        assert!(text_res.is_ok());
    }

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
            .filter(|passage|
                passage.is_none()
            )
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
}
