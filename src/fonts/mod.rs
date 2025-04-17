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

pub fn load_embedded_font_or_err(name: &str) -> FIGfont {
    let content = match name {
        "font_time" => include_str!("./large.flf"),
        "font_date" => include_str!("./small.flf"),
        other => {
            eprintln!("Unknown embedded font name: {}", other);
            return make_error_font();
        }
    };

    FIGfont::from_content(content).unwrap_or_else(|e| {
        eprintln!("Failed to load embedded font '{}': {}", name, e);
        make_error_font()
    })
}

pub fn load_fonts_with_fallback(
    font_time_path: Option<String>,
    font_date_path: Option<String>,
) -> Result<(FIGfont, FIGfont), String> {
    let font_time = match font_time_path {
        Some(path) => load_font_from_file_or_err(&path),
        None => load_embedded_font_or_err("font_time"),
    };

    let font_date = match font_date_path {
        Some(path) => load_font_from_file_or_err(&path),
        None => load_embedded_font_or_err("font_date"),
    };

    Ok((font_time, font_date))
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
