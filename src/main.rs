#[macro_use] extern crate clap;
#[macro_use] extern crate failure_derive;
#[macro_use] extern crate lazy_static;

extern crate actix_web;
extern crate ansi_term;
extern crate atty;
extern crate bytes;
extern crate console;
extern crate failure;
extern crate futures;
extern crate http;
extern crate serde_json;
extern crate url;

mod cli;
mod errors;
mod request;
mod request_item;
mod response;

use actix_web::actix;
use futures::future::Future;

use std::process;

use cli::App;
use errors::*;
use request::*;
use response::*;

/// Returns `Err(..)` upon fatal errors. Otherwise, returns `Some(true)` on full success and
/// `Some(false)` if any intermediate errors occurred (were printed).
fn main() {
    let app = App::new();
    actix::run(move || {
        let config = app.config().unwrap();
        make_request(&config)
            .and_then(move |response| {                     // <- server http response
                process_response(&config, response)
            })
            .map(|v| {
                actix::System::current().stop();
                if v {
                    process::exit(0);
                }
                process::exit(1);
            })
            .map_err(|e| {
                handle_error(e);
                actix::System::current().stop();
                process::exit(1);
            })
    })
}
