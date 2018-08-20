use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::parsing::syntax_definition::SyntaxDefinition;
use syntect::util::as_24_bit_terminal_escaped;

pub enum Body {
    Empty,
    Normal(String),
    Json(String),
}

static NEW_LINES: bool = false;

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

pub fn print_http(header_part: String, body: Body, colored_output: bool, true_color: bool, only_body: bool) {
    if !colored_output {
        if !only_body {
            println!("{}", header_part);
        }
        match body {
            Body::Empty => (),
            Body::Normal(s) => println!("{}", s),
            Body::Json(s) => println!("{}", s),
        }
        return
    }
    let ts = get_theme_set();
    let ss = get_syntax_set();

    let theme = &ts.themes["Solarized (dark)"];

    if !only_body {
        let syntax_http = ss.find_syntax_by_name("HTTP").unwrap();
        let mut h = HighlightLines::new(syntax_http, theme);
        for line in header_part.lines() {
            let ranges: Vec<(Style, &str)> = h.highlight(line);
            let escaped = as_24_bit_terminal_escaped(&ranges[..], true_color);
            println!("{}", escaped);
        }
        println!("\x1b[0m");
    }
    match body {
        Body::Empty => (),
        Body::Normal(s) => println!("{}", s),
        Body::Json(s) => {
            let syntax_json = ss.find_syntax_by_extension("json").unwrap();
            let mut h = HighlightLines::new(syntax_json, theme);
            for line in s.lines() {
                let ranges: Vec<(Style, &str)> = h.highlight(line);
                let escaped = as_24_bit_terminal_escaped(&ranges[..], true_color);
                println!("{}", escaped);
            }
            println!("\x1b[0m");
        },
    }
}
