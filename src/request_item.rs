use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Mutex;

#[derive(Clone, Copy)]
pub enum RequestItemType {
    HTTPHeader,
    URLParameter,
    DataField,
    JsonData,
    FormFile,
    FileDataField,
    FileJsonData,
}

#[derive(Clone)]
pub struct RequestItem {
    pub key: String,
    pub value: String,
    pub variant: RequestItemType,
}

enum TokenisedItem {
    Normal (String),
    Escaped (String),
}

lazy_static! {
    // NOTE: This should be descending order of len of separators
    // TODO: Use type of SEPARATORS as BTreeSet once rfind is no longer in nightly to avoid this
    static ref SEPARATORS: Vec<&'static str> = vec![":=@", ":=", "=@", "==", "@", "=", ":"];

    static ref SEPARATOR_MAP: HashMap<&'static str, RequestItemType> = SEPARATORS.iter().cloned()
        .zip(
            [ RequestItemType::FileJsonData, RequestItemType::JsonData, RequestItemType::FileDataField,
              RequestItemType::URLParameter, RequestItemType::FormFile, RequestItemType::DataField,
              RequestItemType::HTTPHeader
            ].iter().cloned()
        )
        .collect();

    static ref SPECIAL_CHARS_SET: HashSet<char> = {
        let mut s: HashSet<char> = SEPARATORS.iter()
            .map(|i| i.chars().next().unwrap())
            .collect();
        s.insert('\\');
        s
    };

    // Store result in a hashmap using lazy_static
    static ref CACHE: Mutex<HashMap<String, RequestItem>> = Mutex::new(HashMap::new());
}

pub fn is_request_item(v: String) -> Result<(), String> {
    get_request_item(v).map(|_| ())
        .ok_or(String::from("The value does not match the format of a RequestItem"))
}

pub fn get_request_item(v: String) -> Option<RequestItem> {
    if let Some(item) = get_from_cache(&v) {
        return Some(item);
    };

    // Better tokenization needed. Maybe use [1] once its
    // mature enough.
    // [1] - https://github.com/Jeffail/tokesies
    let tokens = tokenize(&v);

    let mut separator = None;
    let mut key = String::new();
    let mut value = String::new();
    for token in tokens {
        if let TokenisedItem::Normal(tok) = token {

            // If separator is None, search for it in this token
            if let None = separator {
                separator = SEPARATORS.iter().find(|&sep| tok.contains(sep));

                // If found, append first half to key and others to value as is
                if let Some(sep) = separator {
                    let mut splitted = tok.split(sep);
                    if let Some(x) = splitted.next() {
                        key.push_str(x);
                    } else {
                        return None;
                    }
                    let remaining: Vec<&str> = splitted.collect();
                    value.push_str(remaining.join(sep).as_str());
                    continue
                }
            }

            // If separator not found yet, this is part of key
            if let None = separator {
                key.push_str(tok.as_str());
            } else {
                value.push_str(tok.as_str());
            }
        } else if let TokenisedItem::Escaped(tok) = token {
            // If separator not found yet, this is part of key
            if let None = separator {
                key.push_str(tok.as_str());
            } else {
                value.push_str(tok.as_str());
            }
        }
    }

    separator.map(move |sep| {
        let variant = SEPARATOR_MAP.get(sep).unwrap().clone();
        let request_item = RequestItem {
            variant: variant,
            key: key,
            value: value,
        };
        insert_to_cache(v, request_item.clone());
        request_item
    })
}

fn tokenize(string: &str) -> Vec<TokenisedItem> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    for chr in string.chars() {
        if chr == '\\' {
            escaped = true;
            continue;
        }
        if escaped {
            if SPECIAL_CHARS_SET.contains(&chr) {
                tokens.push(TokenisedItem::Normal(current));
                current = String::new();
                current.push(chr);
                tokens.push(TokenisedItem::Escaped(current));
                current = String::new();
            } else {
                current.push('\\');
                current.push(chr);
            }
        } else {
            current.push(chr);
        }
        escaped = false
    }
    tokens.push(TokenisedItem::Normal(current));
    tokens
}

fn get_from_cache(v: &String) -> Option<RequestItem> {
    CACHE.lock().unwrap().get(v).map(Clone::clone)
}

fn insert_to_cache(v: String, request_item: RequestItem) {
    CACHE.lock().unwrap().insert(v, request_item);
}
