use reqwest::{ Response };

use serde_json::{to_vec_pretty, from_slice, Value};
use std::io::{self, Write};
use std::result::{Result as StdResult};

use cli::Config;
use errors::*;

pub fn process_response(config: &Config, mut res: Response) -> Result<bool> {

    if config.interactive_output || config.verbose {
        println!("{:?} {} {}", res.version(), res.status().as_u16(),
                 res.status().canonical_reason().unwrap_or(""));
        for (key, value) in res.headers().iter() {
            println!("{}: {}", key.as_str(), value.to_str().expect(""))
        }
        println!("");
    }

    let body = res.text().map(String::into_bytes)?;

    let json: StdResult<Value, _> = from_slice(&body);

    let pretty_json = json.and_then(|x| to_vec_pretty(&x));

    let output = pretty_json.unwrap_or(body);

    io::stdout().write_all(&output)
        .and_then(|_| io::stdout().write_all(&[10]))
        .map(|_| res.status().is_success())
        .chain_err(|| "unable to write to stdout")

}
