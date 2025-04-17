use crate::{fonts, widgets::GJWidget};
use chrono::Local;
use figlet_rs::FIGfont;
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
};

pub struct ClockWidget {
    font_time: FIGfont,
    font_date: FIGfont,
}

impl ClockWidget {
    pub fn new() -> Self {
        let (font_time, font_date) =
            fonts::load_fonts_with_fallback(None, None).expect("Failed to load figlet fonts");
        Self {
            font_time,
            font_date,
        }
    }
}

impl GJWidget for ClockWidget {
    fn render(&self) -> Paragraph {
        let time_style = Style {
            fg: Some(Color::Blue),
            bg: Some(Color::default()),
            underline_color: Some(Color::default()),
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        };

        let date_style = Style {
            fg: Some(Color::Blue),
            bg: Some(Color::default()),
            underline_color: Some(Color::default()),
            add_modifier: Modifier::DIM,
            sub_modifier: Modifier::empty(),
        };
        let now = Local::now();
        let time_str = now.format("%H:%M").to_string();
        let date_str = now.format("%d.%m.%Y").to_string();

        let time_fig = fonts::render_figlet_text(&self.font_time, &time_str);
        let date_fig = fonts::render_figlet_text(&self.font_date, &date_str);

        let mut text = Text::default();

        for line in time_fig.to_string().lines() {
            text.lines.push(Line::styled(line.to_string(), time_style));
        }

        for line in date_fig.to_string().lines() {
            text.lines.push(Line::styled(line.to_string(), date_style));
        }

        Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL))
    }
}
