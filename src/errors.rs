use actix_web;
use actix_web::client;
use actix_web::error;
use http::method;
use serde_json;
use url;

use std::io;

#[derive(Debug, Fail)]
pub enum BrazeError {
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

impl From<actix_web::Error> for BrazeError {
    fn from(error: actix_web::Error) -> BrazeError {
        BrazeError::ActixWebError { error: error }
    }
}

impl From<client::SendRequestError> for BrazeError {
    fn from(error: client::SendRequestError) -> BrazeError {
        BrazeError::SendRequestFailed { error: error }
    }
}

impl From<io::Error> for BrazeError {
    fn from(error: io::Error) -> BrazeError {
        BrazeError::IoError { error: error }
    }
}

impl From<error::PayloadError> for BrazeError {
    fn from(error: error::PayloadError) -> BrazeError {
        BrazeError::BodyParseError { error: error }
    }
}

impl From<method::InvalidMethod> for BrazeError {
    fn from(error: method::InvalidMethod) -> BrazeError {
        BrazeError::MethodParseError { error: error }
    }
}

impl From<url::ParseError> for BrazeError {
    fn from(error: url::ParseError) -> BrazeError {
        BrazeError::UrlParseError { error: error }
    }
}

impl From<serde_json::Error> for BrazeError {
    fn from(_error: serde_json::Error) -> BrazeError {
        BrazeError::UnexpectedError
    }
}

pub fn handle_error(error: BrazeError) -> BrazeError {
    match error {
        _ => {
            use ansi_term::Colour::Red;
            eprintln!("{}: {}", Red.paint("[braze error]"), error);
            error
        }
    }
}
