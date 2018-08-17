use actix_web::HttpMessage;
use actix_web::client::ClientResponse;
use bytes::Bytes;
use futures::Future;
// use serde_json::{to_vec_pretty, from_slice, Value};
use std::io::{self, Write};

use cli::Config;
use errors::BrazeError;

pub fn process_response(config: &Config, res: ClientResponse) -> impl Future<Item = bool, Error = BrazeError> {

    if config.interactive_output || config.verbose {
        println!("{:?} {} {}", res.version(), res.status().as_u16(),
                 res.status().canonical_reason().unwrap_or(""));
        for (key, value) in res.headers().iter() {
            println!("{}: {}", key.as_str(), value.to_str().expect(""))
        }
        println!("");
    }

    res.body()
        .map_err(BrazeError::from)
        .and_then(move |bytes: Bytes| {  // <- complete body
            io::stdout().write_all(&bytes)
                .and_then(|_| io::stdout().write_all(&[10]))
                .map(|_| res.status().is_success())
                .map_err(BrazeError::from)
        })
//     let body = res.text().map(String::into_bytes)?;

//     let json: StdResult<Value, _> = from_slice(&body);

//     let pretty_json = json.and_then(|x| to_vec_pretty(&x));

//     let output = pretty_json.unwrap_or(body);

//     io::stdout().write_all(&output)
//         .and_then(|_| io::stdout().write_all(&[10]))
//         .map(|_| res.status().is_success())
//         .chain_err(|| "unable to write to stdout")

}
