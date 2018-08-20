use actix_web::http::Method;
use atty::{self, Stream};
use clap::{App as ClapApp, AppSettings, Arg, ArgMatches};
use console::Term;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::parsing::syntax_definition::SyntaxDefinition;

#[cfg(windows)]
use ansi_term;

use std::env;
use std::str::FromStr;

use errors::ChromeError;
use request::BodyType;
use request_item::{RequestItem, is_request_item, get_request_item};

static NEW_LINES: bool = false;

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
    pub body_type: BodyType,
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
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
            .arg(Arg::with_name("json")
                 .short("j")
                 .long("json")
                 .help("Force json for request arguments")
            )
            .arg(Arg::with_name("form")
                 .short("f")
                 .long("form")
                 .help("Force sending as form for request arguments")
                 .conflicts_with("json")
            )
            .get_matches()
    }

    pub fn config(&self) -> Result<Config, ChromeError> {
        let url = self.matches.value_of("URL").unwrap();
        let request_items = self.request_items();
        let body_type = if self.matches.is_present("json") {
            BodyType::JSON
        } else if self.matches.is_present("form") {
            BodyType::Form
        } else {
            BodyType::Undecided
        };

        Ok(Config {
            method: self.method()?,
            url: String::from(url),
            items: request_items,
            body_type: body_type,
            colored_output: self.interactive_output,
            interactive_output: self.interactive_output,
            term_width: Term::stdout().size().1 as usize,
            true_color: is_truecolor_terminal(),
            verbose: self.matches.is_present("verbose"),
            syntax_set: get_syntax_set(),
            theme_set: get_theme_set(),
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

pub fn get_syntax_set() -> SyntaxSet {
    let http_def: SyntaxDefinition = SyntaxDefinition::load_from_str(
        include_str!("./http.sublime-syntax"),
        NEW_LINES,
        Some("HTTP")
    ).expect("Unable to parse http sublime syntax");

    let mut ss = if NEW_LINES {
        SyntaxSet::load_defaults_newlines()
    } else {
        SyntaxSet::load_defaults_nonewlines()
    };
    ss.add_syntax(http_def);
    ss.link_syntaxes();
    ss
}

pub fn get_theme_set() -> ThemeSet {
    ThemeSet::load_defaults()
}
