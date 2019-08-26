use serde::Deserialize;
use serde_json::{self, Value};
use std::collections::HashMap;

// Hashmap structure:
// Passage({
//     <verse-no> : {
//         "verse_nr" : Number | String,
//         "verse" : String
// })
#[derive(Default, Deserialize, Debug)]
pub struct Passage(HashMap<String, HashMap<String, Value>>);

impl From<Value> for Passage {
    fn from(v: Value) -> Passage {
        serde_json::from_value(v).unwrap_or_default()
    }
}

impl ToString for Passage {
    fn to_string(&self) -> String {
        let mut keys: Vec<u8> = self
            .0
            .keys()
            .map(|k| k.parse::<_>().unwrap_or_default())
            .collect();

        keys.sort();

        keys.into_iter()
            .map(|key| {
                let k = key.to_string();
                let verse = self
                    .0
                    .get(&k)
                    .and_then(|v| v.get("verse"))
                    .and_then(|verse_value| verse_value.as_str());
                format!("^({}) {}", k, verse.unwrap_or_default())
            })
            .collect::<String>()
    }
}

pub struct PassageInfo {
    book: String,
    chapter: String,
    version: String,
}

impl PassageInfo {
    pub fn new(book: Value, chapter: Value, version: Value) -> PassageInfo {
        PassageInfo {
            book: String::from(book.as_str().unwrap_or_default()),
            chapter: String::from(chapter.as_str().unwrap_or_default()),
            version: String::from(version.as_str().unwrap_or_default()),
        }
    }
}

impl ToString for PassageInfo {
    fn to_string(&self) -> String {
        format!("{} {} ({})", self.book, self.chapter, self.version)
    }
}
