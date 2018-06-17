use reqwest::{ Response };

use std::io::{self, Write};

use cli::Config;
use errors::*;

pub fn process_response(_config: &Config, mut res: Response) -> Result<bool> {
    println!("Response: {}", res.status());
    println!("Headers: {:#?}", res.headers());

    let body = res.text().map(String::into_bytes)?;
    io::stdout().write_all(&body)
        .map(|_| res.status().is_success())
        .chain_err(|| "unable to write to stdout")
}
