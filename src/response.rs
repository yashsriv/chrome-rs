use actix_web::HttpMessage;
use actix_web::client::ClientResponse;
use actix_web::error::ContentTypeError;
use bytes::Bytes;
use futures::Future;
use serde_json::{to_string_pretty, from_slice, Value};

use cli::Config;
use errors::ChromeError;
use output::*;

pub fn process_response(config: &Config, res: ClientResponse) -> impl Future<Item = bool, Error = ChromeError> {
    let mut response_str = String::new();
    if config.interactive_output || config.verbose {
        let first_line = format!("{:?} {} {}\n", res.version(), res.status().as_u16(),
                                  res.status().canonical_reason().unwrap_or(""));
        response_str.push_str(&first_line);
        for (key, value) in res.headers().iter() {
            let headerval_pair = format!("{}: {}\n", key.as_str(), value.to_str().expect(""));
            response_str.push_str(&headerval_pair);
        }
    }

    let mime_type = res.mime_type();
    let success = res.status().is_success();

    let colored = config.colored_output;
    let true_color = config.true_color;
    let only_body = !(config.interactive_output || config.verbose);

    res.body()
        .from_err()
        .and_then(move |bytes: Bytes| {  // <- complete body

            mime_type
                .and_then(|option_mime| option_mime.ok_or(ContentTypeError::UnknownEncoding))
                .map(|m| {
                    let cur_mime = format!("{}/{}", m.type_(), m.subtype());
                    let expected_mime = String::from("application/json");
                    cur_mime == expected_mime
                })
                .and_then(|is_json| {
                    if is_json {
                        from_slice::<Value>(&bytes)
                            .and_then(|x| to_string_pretty(&x))
                            .map(|s| Body::Json(s))
                            .map_err(|_| ContentTypeError::ParseError)
                    } else {
                        // unwrap_or will automatically handle this case
                        // so this isn't exactly a parse error
                        Err(ContentTypeError::ParseError)
                    }
                })
                .or(
                    String::from_utf8(bytes.to_vec())
                        .map(|s| Body::Normal(s))
                        .map_err(|_| ChromeError::UnexpectedError)
                )
                .map(|output| print_http(response_str, output, colored, true_color, only_body))
        })
        .map(move |_| success)

}
