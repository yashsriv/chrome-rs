use reqwest::{Client, RequestBuilder};
use serde_json::{self, Value};

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use cli::Config;
use errors::*;
use request_item::RequestItem;
use request_item::RequestItemType::*;
use response::process_response;

pub fn make_request(config: &Config) -> Result<bool> {
    let client = Client::builder().build()?;
    let mut req = client.request(config.method.clone(), config.url.clone());
    parse_request_items(config, &mut req)?;
    let response = req.send()?;
    process_response(config, response)
}

pub fn parse_request_items(config: &Config, req: &mut RequestBuilder) -> Result<()> {
    // Process query params
    let query_params: Vec<(&String, &String)> = config.items.iter()
        .filter(|x| match x.variant { URLParameter => true, _ => false })
        .map(|x| (&x.key, &x.value))
        .collect();
    req.query(&query_params);

    // TODO: Process headers. Way easier with hyper 0.12 but reqwest hasn't shifted to that (yet).
    let _headers: Vec<&RequestItem> = config.items.iter()
        .filter(|x| match x.variant { HTTPHeader => true, _ => false })
        .collect();

    // Process body
    let body_items = config.items.iter()
        .filter(|x| match x.variant { HTTPHeader => false, URLParameter => false, _ => true });

    // TODO: Check if form or json. For now, assuming json
    let mut map = HashMap::new();
    for item in body_items {
        match item.variant {
            DataField => {
                map.insert(item.key.clone(), Value::String(item.value.clone()));
            },
            JsonData => {
                map.insert(item.key.clone(), serde_json::from_str(item.value.as_str())?);
            },
            FileDataField => {
                let mut file = File::open(item.value.as_str())?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                map.insert(item.key.clone(), Value::String(contents));
            },
            FileJsonData => {
                let mut file = File::open(item.value.as_str())?;
                map.insert(item.key.clone(), serde_json::from_reader(file)?);
            }
            FormFile => unimplemented!(),
            _ => bail!("impossible scenarios")
        };
    }
    req.json(&map);
    Ok(())
}
