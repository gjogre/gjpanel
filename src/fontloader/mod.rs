use figlet_rs::{FIGfont, FIGure};
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

pub fn load_font_by_name_or_err(font_name: &str) -> FIGfont {
    let default_path = format!("./fonts/{}.flf", font_name);
    load_font_from_file_or_err(&default_path)
}

pub fn render_figlet_text<'a>(font: &'a FIGfont, text: &'a str) -> FIGure<'a> {
    font.convert(text).unwrap_or_else(|| {
        font.convert("ERR")
            .unwrap_or_else(|| font.convert("!").unwrap())
    })
}

fn make_error_font() -> FIGfont {
    FIGfont::standard().unwrap()
}
