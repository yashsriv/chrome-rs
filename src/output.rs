use syntect::easy::HighlightLines;
use syntect::highlighting::Style;
use syntect::util::as_24_bit_terminal_escaped;

use cli::{get_syntax_set, get_theme_set};

pub enum Body {
    Empty,
    Form(String),
    Json(String),
}

pub fn print_http(header_part: String, body: Body, colored_output: bool, true_color: bool, only_body: bool) {
    if !colored_output {
        if !only_body {
            println!("{}", header_part);
        }
        match body {
            Body::Empty => (),
            Body::Form(s) => println!("{}", s),
            Body::Json(s) => println!("{}", s),
        }
        return
    }

    // TODO: Find some way to get this statically
    // Help Needed!!
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
        Body::Form(s) => println!("{}", s),
        Body::Json(s) => {
            let syntax_json = ss.find_syntax_by_extension("json").unwrap();
            let mut h = HighlightLines::new(syntax_json, theme);
            for line in s.lines() {
                let ranges: Vec<(Style, &str)> = h.highlight(line);
                let escaped = as_24_bit_terminal_escaped(&ranges[..], true_color);
                println!("{}", escaped);
            }
            print!("\x1b[0m");
        },
    }
}
