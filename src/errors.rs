error_chain! {
    foreign_links {
        Clap(::clap::Error);
        Http(::http::Error);
        Hyper(::hyper::Error);
        Reqwest(::reqwest::Error);
        ReqwestUrl(::reqwest::UrlError);
        Std(::std::io::Error);
        Serde(::serde_json::Error);
    }
}

pub fn handle_error(error: &Error) {
    match error {
        _ => {
            use ansi_term::Colour::Red;
            eprintln!("{}: {}", Red.paint("[braze error]"), error);
        }
    };
}
