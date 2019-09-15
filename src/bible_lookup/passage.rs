use super::err::{BibleLookupError, BibleLookupResult};
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

impl Passage {
    pub fn new(v: &Value) -> BibleLookupResult<Self> {
        let contents = match v["type"].as_str().unwrap_or_default() {
            "chapter" => Ok(v["chapter"].clone()),
            "verse" => Ok(v["book"][0]["chapter"].clone()),
            _ => Err(BibleLookupError::BadPassageType),
        }?;

        let res = serde_json::from_value(contents)?;

        Ok(res)
    }
}

impl ToString for Passage {
    fn to_string(&self) -> String {
        let mut keys: Vec<usize> = self
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
                    .and_then(Value::as_str);
                format!("^({}) {}", k, verse.unwrap_or_default())
            })
            .collect::<String>()
    }
}

pub struct Info {
    book: String,
    chapter: String,
    version: String,
}

impl Info {
    pub fn new(v: &Value) -> BibleLookupResult<Self> {
        let (book, chapter, version) = match v["type"].as_str().unwrap_or_default() {
            "chapter" => Ok((
                v["book"].clone(),
                v["chapter"].clone(),
                v["version"].clone(),
            )),
            "verse" => Ok((
                v["book"][0]["book_name"].clone(),
                v["book"][0]["chapter_nr"].clone(),
                v["version"].clone(),
            )),
            _ => Err(BibleLookupError::BadPassageType),
        }?;

        Ok(Self {
            book: serde_json::from_value(book)?,
            chapter: serde_json::from_value(chapter)?,
            version: serde_json::from_value(version)?,
        })
    }
}

impl ToString for Info {
    fn to_string(&self) -> String {
        format!("{} {} ({})", self.book, self.chapter, self.version)
    }
}
