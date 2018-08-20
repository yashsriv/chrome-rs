use actix_web::client::{ ClientRequest, ClientRequestBuilder, ClientResponse };
use futures::future::{ self, Future };
use serde_json::{self, Value};
use serde_urlencoded;
use url::Url;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use cli::Config;
use errors::ChromeError;
use request_item::RequestItemType::*;
use output::*;

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub enum BodyType {
    JSON,
    Form,
    Multipart,
    Undecided,
}

pub fn make_request(config: &Config) -> Box<Future<Item = ClientResponse, Error = ChromeError>> {
    let mut req_builder = ClientRequest::build();
    req_builder
        .header("User-Agent", format!("{}/{}", crate_name!(), crate_version!()))
        .method(config.method.clone());

    match parse_request_items(config, req_builder) {
        Err(e) => Box::new(future::err(e)),
        Ok((body, request)) => {
            if config.verbose {
                process_request(config, &request, body);
            }
            Box::new(request.send().map_err(ChromeError::from))
        }
    }
}

fn process_request(config: &Config, request: &ClientRequest, body: Body) {
    let mut request_str = String::new();
    let first_line = format!("{} {}{}{} {:?}\n", request.method().as_ref(), request.uri().path(),
                             request.uri().query().map(|_| "?").unwrap_or(""), request.uri().query().unwrap_or(""),
                             request.version());
    request_str.push_str(&first_line);
    for (key, value) in request.headers().iter() {
        let headerval_pair = format!("{}: {}\n", key.as_str(), value.to_str().expect(""));
        request_str.push_str(&headerval_pair);
    }
    print_http(request_str, body, config.colored_output, config.true_color, false);
    println!("");
}

fn parse_request_items(config: &Config, mut req: ClientRequestBuilder) -> Result<(Body, ClientRequest), ChromeError> {

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

    // TODO: Allow overriding default in config
    let mut body_type = config.body_type;
    let mut json_map: HashMap<String, Value> = HashMap::new();
    let mut data_map: HashMap<String, String> = HashMap::new();
    for item in body_items {
        match item.variant {
            DataField => {
                data_map.insert(item.key.clone(), item.value.clone());
            },
            JsonData => {
                if body_type != BodyType::Undecided && body_type != BodyType::JSON {
                    return Err(ChromeError::UnexpectedError);
                }
                body_type = BodyType::JSON;
                json_map.insert(item.key.clone(), serde_json::from_str(item.value.as_str())?);
            },
            FileDataField => {
                let mut file = File::open(item.value.as_str())?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                data_map.insert(item.key.clone(), contents);
            },
            FileJsonData => {
                if body_type != BodyType::Undecided && body_type != BodyType::JSON {
                    return Err(ChromeError::UnexpectedError);
                }
                body_type = BodyType::JSON;
                let mut file = File::open(item.value.as_str())?;
                json_map.insert(item.key.clone(), serde_json::from_reader(file)?);
            }
            FormFile => {
                if body_type == BodyType::JSON {
                    return Err(ChromeError::UnexpectedError);
                }
                // body_type = BodyType::Multipart;
                unimplemented!()
            },
            _ => return Err(ChromeError::UnexpectedError),
        };
    }
    match body_type {
        BodyType::Undecided => {
            if data_map.is_empty() {
                Ok((Body::Empty, req.finish()?))
            } else {
                Ok((Body::Json(serde_json::to_string_pretty(&data_map)?), req.json(data_map)?))
            }
        },
        BodyType::JSON => {
            json_map.extend(data_map.into_iter().map(|(k, v)| (k, Value::String(v))));
            Ok((Body::Json(serde_json::to_string_pretty(&json_map)?), req.json(json_map)?))
        },
        BodyType::Form => {
            Ok((Body::Form(serde_urlencoded::to_string(&data_map)?), req.form(data_map)?))
        },
        BodyType::Multipart => unimplemented!(),
    }
}
