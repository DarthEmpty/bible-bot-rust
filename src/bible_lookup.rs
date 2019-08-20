use reqwest;
use regex::Regex;
use serde_json::{self, Value};

struct Chapter;

struct Verse;

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

fn lookup_ref(reference: &str) -> Option<Value> {
    let url = format!("https://getbible.net/json?text={}", reference);
    let text: String = reqwest::get(&url).unwrap()
        .text().unwrap()
        .replace("(", "")
        .replace(");", "");

    let json: Value = serde_json::from_str(&text).unwrap_or_default();

    match json["type"].as_str().unwrap() {
        "verse" | "chapter" => Some(json),
        _ => None
    }
}

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
        let passage = lookup_ref("John3:16-17");
        assert_ne!(passage, None);
    }
}
