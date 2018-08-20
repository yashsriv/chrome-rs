use actix_web;
use actix_web::client;
use actix_web::error;
use http::method;
use serde_json;
use url;

use std::io;

#[derive(Debug, Fail)]
pub enum ChromeError {
    #[fail(display = "ActixWebError: {}", error)]
    ActixWebError { error: actix_web::Error },
    #[fail(display = "Payload error: {}", error)]
    BodyParseError { error: error::PayloadError },
    #[fail(display = "IO error: {}", error)]
    IoError { error: io::Error },
    #[fail(display = "Method Parse error: {}", error)]
    MethodParseError { error: method::InvalidMethod },
    #[fail(display = "SendRequestFailed: {}", error)]
    SendRequestFailed { error: client::SendRequestError },
    #[fail(display = "Uri Parse error: {}", error)]
    UrlParseError { error: url::ParseError },
    #[fail(display = "An unexpected error has occurred.")]
    UnexpectedError,
}

impl From<actix_web::Error> for ChromeError {
    fn from(error: actix_web::Error) -> ChromeError {
        ChromeError::ActixWebError { error: error }
    }
}

impl From<client::SendRequestError> for ChromeError {
    fn from(error: client::SendRequestError) -> ChromeError {
        ChromeError::SendRequestFailed { error: error }
    }
}

impl From<io::Error> for ChromeError {
    fn from(error: io::Error) -> ChromeError {
        ChromeError::IoError { error: error }
    }
}

impl From<error::PayloadError> for ChromeError {
    fn from(error: error::PayloadError) -> ChromeError {
        ChromeError::BodyParseError { error: error }
    }
}

impl From<method::InvalidMethod> for ChromeError {
    fn from(error: method::InvalidMethod) -> ChromeError {
        ChromeError::MethodParseError { error: error }
    }
}

impl From<url::ParseError> for ChromeError {
    fn from(error: url::ParseError) -> ChromeError {
        ChromeError::UrlParseError { error: error }
    }
}

impl From<serde_json::Error> for ChromeError {
    fn from(_error: serde_json::Error) -> ChromeError {
        ChromeError::UnexpectedError
    }
}

pub fn handle_error(error: ChromeError) -> ChromeError {
    match error {
        _ => {
            use ansi_term::Colour::Red;
            eprintln!("{}: {}", Red.paint("[chrome error]"), error);
            error
        }
    }
}
