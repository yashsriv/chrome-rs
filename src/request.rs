use reqwest::{Client, RequestBuilder};
use serde_json::{self, Value};

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use cli::Config;
use errors::*;
use request_item::RequestItemType::*;
use response::process_response;

pub fn make_request(config: &Config) -> Result<bool> {
    let client = Client::builder().build()?;
    let req = client.request(config.method.clone(), config.url.clone());
    let (body_string, req) = parse_request_items(config, req)?;
    let request = req.build()?;
    if config.verbose {
        println!("{} {}", request.method().as_ref(), request.url().path());
        for (key, value) in request.headers().iter() {
            println!("{}: {}", key.as_str(), value.to_str().expect(""))
        }
        println!("");
        println!("{}", body_string);
        println!("");
    }
    let response = client.execute(request)?;
    process_response(config, response)
}

pub fn parse_request_items(config: &Config, mut req: RequestBuilder) -> Result<(String, RequestBuilder)> {
    // Process query params
    let query_params: Vec<(&String, &String)> = config.items.iter()
        .filter(|x| match x.variant { URLParameter => true, _ => false })
        .map(|x| (&x.key, &x.value))
        .collect();
    req = req.query(&query_params);

    // Process headers
    let headers = config.items.iter()
        .filter(|x| match x.variant { HTTPHeader => true, _ => false });

    for header in headers {
        req = req.header(header.key.as_str(), header.value.as_str());
    }

    // Process body
    let body_items = config.items.iter()
        .filter(|x| match x.variant { HTTPHeader => false, URLParameter => false, _ => true });

    // TODO: Check if form urlencoded or json. For now, assuming json
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
    req = req.json(&map);
    Ok((serde_json::to_string_pretty(&map)?, req))
}
