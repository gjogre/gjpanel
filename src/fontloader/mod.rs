use figlet_rs::{FIGfont, FIGure};
use ratatui::{
    style::Style,
    text::{Line, Text},
};
use std::fs;

pub fn load_font_from_file_or_err(path: &str) -> FIGfont {
    match fs::read_to_string(path) {
        Ok(content) => match FIGfont::from_content(&content) {
            Ok(font) => font,
            Err(e) => {
                eprintln!("Invalid font format in {}: {}", path, e);
                make_error_font()
            }
        },
        Err(e) => {
            eprintln!("Failed to read font file {}: {}", path, e);
            make_error_font()
        }
    }
}

pub fn load_font_by_name_or_err(font_name: &str) -> Option<FIGfont> {
    if font_name.is_empty() {
        return None;
    }
    let default_path = format!("./fonts/{}.flf", font_name);
    Some(load_font_from_file_or_err(&default_path))
}

pub fn render_figlet_text<'a>(font: &'a FIGfont, text: &'a str) -> FIGure<'a> {
    font.convert(text).unwrap_or_else(|| {
        font.convert("ERR")
            .unwrap_or_else(|| font.convert("!").unwrap())
    })
}

pub fn to_styled_text<'a>(font: &'a Option<FIGfont>, text: &'a str, style: Style) -> Text<'a> {
    let mut result_text = Text::default();

    match font {
        Some(font) => {
            let fig_text = render_figlet_text(font, text);
            for line in fig_text.to_string().lines() {
                result_text
                    .lines
                    .push(Line::styled(line.to_string(), style));
            }
        }
        None => {
            result_text.lines.push(Line::styled(text, style));
        }
    }

    result_text
}

fn make_error_font() -> FIGfont {
    FIGfont::standard().unwrap()
}
