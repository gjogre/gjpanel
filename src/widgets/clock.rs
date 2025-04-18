use crate::fontloader;
use crate::widgets::GJWidget;
use crate::{config::ClockConfig, fontloader::load_font_by_name_or_err};
use chrono::Local;
use figlet_rs::FIGfont;
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
};

pub struct ClockWidget {
    pub config: ClockConfig,
    font_time: FIGfont,
    font_date: FIGfont,
}

impl ClockWidget {
    pub fn new(config: ClockConfig) -> Self {
        let font_time = load_font_by_name_or_err("large");
        let font_date = load_font_by_name_or_err("small");
        Self {
            font_time,
            font_date,
            config,
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
            add_modifier: Modifier::DIM | Modifier::BOLD,
            sub_modifier: Modifier::empty(),
        };
        let now = Local::now();
        let time_str = now.format(&self.config.time_format).to_string();
        let date_str = now.format(&self.config.date_format).to_string();

        let time_fig = fontloader::render_figlet_text(&self.font_time, &time_str);
        let date_fig = fontloader::render_figlet_text(&self.font_date, &date_str);

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
