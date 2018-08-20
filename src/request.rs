use actix_web::client::{ ClientRequest, ClientRequestBuilder, ClientResponse };
use futures::future::{ self, Future };
use serde_json::{self, Value};
use url::Url;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use cli::Config;
use errors::BrazeError;
use request_item::RequestItemType::*;
use output::*;

pub fn make_request(config: &Config) -> Box<Future<Item = ClientResponse, Error = BrazeError>> {
    let mut req_builder = ClientRequest::build();
    req_builder
        .method(config.method.clone());

    match parse_request_items(config, req_builder) {
        Err(e) => Box::new(future::err(e)),
        Ok((body_string, request)) => {
            if config.verbose {
                let mut request_str = String::new();
                let first_line = format!("{} {}{}{} {:?}\n", request.method().as_ref(), request.uri().path(),
                         request.uri().query().map(|_| "?").unwrap_or(""), request.uri().query().unwrap_or(""),
                         request.version());
                request_str.push_str(&first_line);
                for (key, value) in request.headers().iter() {
                    let headerval_pair = format!("{}: {}\n", key.as_str(), value.to_str().expect(""));
                    request_str.push_str(&headerval_pair);
                }
                let body = if body_string != String::from("") &&
                    body_string != String::from("{}") {
                        Body::Normal(format!("{}", body_string))
                    } else {
                        Body:: Empty
                    };
                print_http(request_str, body, config.colored_output, config.true_color, false);
            }
            Box::new(request.send().map_err(BrazeError::from))
        }
    }
}

pub fn parse_request_items(config: &Config, mut req: ClientRequestBuilder) -> Result<(String, ClientRequest), BrazeError> {

    // Process query params
    let query_params: Vec<(&String, &String)> = config.items.iter()
        .filter(|x| match x.variant { URLParameter => true, _ => false })
        .map(|x| (&x.key, &x.value))
        .collect();
    let url = Url::parse_with_params(&config.url, &query_params)?;

    req.uri(url.as_str());

   // Process headers
    let headers = config.items.iter()
        .filter(|x| match x.variant { HTTPHeader => true, _ => false });

    for header in headers {
        req.header(header.key.as_str(), header.value.as_str());
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
            _ => return Err(BrazeError::UnexpectedError),
        };
    }
    Ok((serde_json::to_string_pretty(&map)?, req.json(map)?))
}
