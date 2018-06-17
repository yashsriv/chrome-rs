// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate lazy_static;

extern crate ansi_term;
extern crate atty;
extern crate console;
extern crate hyper;
extern crate reqwest;
extern crate serde_json;

mod cli;
mod errors;
mod request;
mod request_item;
mod response;

use std::process;

use cli::App;
use errors::*;
use request::*;

/// Returns `Err(..)` upon fatal errors. Otherwise, returns `Some(true)` on full success and
/// `Some(false)` if any intermediate errors occurred (were printed).
fn run() -> Result<bool> {
    let app = App::new();
    let config = app.config()?;
    make_request(&config)
}

fn main() {
    let result = run();
    match result {
        Err(error) => {
            handle_error(&error);
            process::exit(1);
        }
        Ok(false) => {
            process::exit(1);
        }
        Ok(true) => {
            process::exit(0);
        }
    }
}
