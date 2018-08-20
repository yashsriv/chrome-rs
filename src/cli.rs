use atty::{self, Stream};
use clap::{App as ClapApp, AppSettings, Arg, ArgMatches};
use console::Term;
use actix_web::http::Method;

#[cfg(windows)]
use ansi_term;

use std::env;
use std::str::FromStr;

use errors::ChromeError;
use request_item::{RequestItem, is_request_item, get_request_item};

pub struct App {
    pub matches: ArgMatches<'static>,
    interactive_output: bool,
}

pub struct Config {
    // Main stuff relating to the request to be made
    pub items: Vec<RequestItem>,
    pub method: Method,
    pub url: String,
    // Formatting options, etc.
    pub colored_output: bool,
    pub interactive_output: bool,
    pub term_width: usize,
    pub true_color: bool,
    pub verbose: bool,
    // pub output_wrap
    // pub paging_mode
}

impl App {
    pub fn new() -> Self {
        let interactive_output = atty::is(Stream::Stdout);

        #[cfg(windows)]
        let interactive_output = interactive_output && ansi_term::enable_ansi_support().is_ok();

        App {
            matches: Self::matches(interactive_output),
            interactive_output,
        }
    }

    fn matches(interactive_output: bool) -> ArgMatches<'static> {
        let clap_color_setting = if interactive_output {
            AppSettings::ColoredHelp
        } else {
            AppSettings::ColorNever
        };

        ClapApp::new(crate_name!())
            .version(crate_version!())
            .author(crate_authors!())
            .about("A command-line curl replacement with a better UX")
            .max_term_width(90)
            .global_setting(clap_color_setting)
            .arg(Arg::with_name("METHOD")
                 .help("The HTTP method to be used for the request (GET, POST, PUT, DELETE, ...).")
                 .long_help(include_str!("./help/method.help.txt"))
                 .required(true)
                 .index(1)
            )
            .arg(Arg::with_name("URL")
                 .help("The URL for the request.")
                 .long_help(include_str!("./help/url.help.txt"))
                 .required(true)
                 .index(2)
            )
            .arg(Arg::with_name("REQUEST_ITEM")
                 .help("A part of the request to be sent")
                 .long_help(include_str!("./help/request_item.help.txt"))
                 .multiple(true)
                 .validator(is_request_item)
                 .index(3)
            )
            .arg(Arg::with_name("verbose")
                 .short("v")
                 .long("verbose")
                 .help("Verbose output")
                 .long_help(include_str!("./help/verbose.help.txt"))
            )
            .get_matches()
    }

    pub fn config(&self) -> Result<Config, ChromeError> {
        let url = self.matches.value_of("URL").unwrap();
        let request_items = self.request_items();

        Ok(Config {
            method: self.method()?,
            url: String::from(url),
            items: request_items,
            colored_output: self.interactive_output,
            interactive_output: self.interactive_output,
            term_width: Term::stdout().size().1 as usize,
            true_color: is_truecolor_terminal(),
            verbose: self.matches.is_present("verbose"),
        })
    }

    fn request_items(&self) -> Vec<RequestItem> {
        self.matches
            .values_of("REQUEST_ITEM")
            .map(|values| {
                values
                    .map(String::from)
                    .map(get_request_item)
                    .map(Option::unwrap)  // We can unwrap safely here because due to validation, we know this can be parsed
                    .collect()
            })
            .unwrap_or_else(|| vec![])
    }

    fn method(&self) -> Result<Method, ChromeError> {
        let method = self.matches.value_of("METHOD").unwrap().to_uppercase();
        Method::from_str(method.as_str()).map_err(ChromeError::from)
    }
}

fn is_truecolor_terminal() -> bool {
    env::var("COLORTERM")
        .map(|colorterm| colorterm == "truecolor" || colorterm == "24bit")
        .unwrap_or(false)
}
