//! Converts ANSI to HTML
//!
//! A lot of assumptions are made about the input, so there's plenty of possible panics and
//! potentially bad minification if you open it up to any kind of input

use std::{
    io::{self, prelude::*},
    mem,
};

use console::AnsiCodeIterator;

#[derive(Clone, Debug)]
enum HtmlTag {
    BoldStart,
    BoldEnd,
    SpanStart(Color),
    SpanEnd,
    Text(String),
}

#[derive(Clone, Copy, Debug)]
enum Ansi {
    Reset,
    Bold,
    Color(Color),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
}

impl Ansi {
    fn new(s: &str) -> Self {
        let value = s
            .strip_prefix("\u{1b}[")
            .unwrap()
            .strip_suffix('m')
            .unwrap();

        match value {
            "0" => Self::Reset,
            "1" => Self::Bold,
            "31" | "38;5;9" => Self::Color(Color::Red),
            "32" => Self::Color(Color::Green),
            "33" => Self::Color(Color::Yellow),
            "34" | "38;5;12" => Self::Color(Color::Blue),
            "35" => Self::Color(Color::Magenta),
            "36" => Self::Color(Color::Cyan),
            _ => panic!("Unexpected val: {value:?}"),
        }
    }
}

fn ansi_to_html(s: &str) -> Vec<HtmlTag> {
    let mut tags = Vec::new();
    let mut is_bold = false;
    let mut current_color = None;
    let mut pending_bold_reset = false;
    let mut pending_color_reset = false;
    for (snippet, is_ansi) in AnsiCodeIterator::new(s) {
        if is_ansi {
            match Ansi::new(snippet) {
                Ansi::Reset => {
                    pending_bold_reset = is_bold;
                    pending_color_reset = current_color.is_some();
                }
                Ansi::Bold => {
                    pending_bold_reset = false;
                    if !is_bold {
                        is_bold = true;
                        tags.push(HtmlTag::BoldStart);
                    }
                }
                Ansi::Color(color) => {
                    if current_color == Some(color) {
                        pending_color_reset = false;
                    } else {
                        if mem::take(&mut pending_color_reset) {
                            tags.push(HtmlTag::SpanEnd);
                        }
                        current_color = Some(color);
                        tags.push(HtmlTag::SpanStart(color))
                    }
                }
            }
        } else {
            let contains_non_space = snippet.bytes().any(|b| b != b' ');
            if contains_non_space {
                if mem::take(&mut pending_color_reset) {
                    current_color = None;
                    tags.push(HtmlTag::SpanEnd);
                }
                if mem::take(&mut pending_bold_reset) {
                    is_bold = false;
                    tags.push(HtmlTag::BoldEnd)
                }
            }

            tags.push(HtmlTag::Text(snippet.to_owned()));
        }
    }

    if pending_color_reset {
        tags.push(HtmlTag::SpanEnd);
    }
    if pending_bold_reset {
        tags.push(HtmlTag::BoldEnd);
    }

    tags
}

fn main() {
    let stdin = io::stdin();

    println!("<pre>");
    for line in stdin.lock().lines() {
        let mut output = String::new();
        let tags = ansi_to_html(&line.unwrap());
        for tag in tags {
            match tag {
                HtmlTag::BoldStart => output.push_str("<b>"),
                HtmlTag::BoldEnd => output.push_str("</b>"),
                HtmlTag::SpanStart(color) => {
                    let color = match color {
                        Color::Red => "term-red",
                        Color::Green => "term-green",
                        Color::Yellow => "term-yellow",
                        Color::Blue => "term-blue",
                        Color::Magenta => "term-magenta",
                        Color::Cyan => "term-cyan",
                    };

                    output.push_str(&format!("<span class=\"{color}\">"));
                }
                HtmlTag::SpanEnd => output.push_str("</span>"),
                HtmlTag::Text(text) => output.push_str(&text),
            }
        }

        println!("{output}");
    }
    println!("</pre>");
}
