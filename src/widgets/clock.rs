use crate::fontloader;
use crate::widgets::GJWidget;
use crate::{config::ClockConfig, fontloader::load_font_by_name_or_err};
use chrono::Local;
use figlet_rs::FIGfont;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

pub struct ClockWidget {
    pub config: ClockConfig,
    font_time: Option<FIGfont>,
    font_date: Option<FIGfont>,
}

impl ClockWidget {
    pub fn new(config: ClockConfig) -> Self {
        let font_time = load_font_by_name_or_err(&config.time_font);
        let font_date = load_font_by_name_or_err(&config.date_font);
        Self {
            font_time,
            font_date,
            config,
        }
    }
}

impl GJWidget for ClockWidget {
    fn render(&self, f: &mut Frame, area: Rect) {
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

        let text = fontloader::to_styled_text(&self.font_time, &time_str, time_style)
            + fontloader::to_styled_text(&self.font_date, &date_str, date_style);

        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(paragraph, area);
    }
}
