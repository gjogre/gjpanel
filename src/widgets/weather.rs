use crate::fontloader;
use crate::logger::Logger;
use crate::widgets::GJWidget;
use crate::{config::WeatherConfig, fontloader::load_font_by_name_or_err};
use std::env;
use std::process::Command;

use figlet_rs::FIGfont;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

pub struct WeatherWidget {
    config: WeatherConfig,
    font: Option<FIGfont>,
    state: String,
    logger: &'static Logger,
}

impl WeatherWidget {
    pub fn new(config: WeatherConfig, logger: &'static Logger) -> Self {
        let font = load_font_by_name_or_err(&config.font);
        Self {
            state: "Loading".to_string(),
            config,
            font,
            logger,
        }
    }

    fn fetch_weather(location: String, logger: &'static Logger) -> String {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "curl -s wttr.in/{}?format=%c+%t+%w+%m+%p",
                location
            ))
            .output()
            .expect("Failed to run weather command");

        match output {
            output if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            }
            output => {
                let error_message = format!(
                    "Weather script error: {}\nscript path: {}",
                    String::from_utf8_lossy(&output.stderr),
                    env::current_dir().unwrap().to_string_lossy(),
                );
                logger.error(&error_message);
                error_message
            }
        }
    }
}
impl GJWidget for WeatherWidget {
    fn poll(&mut self) {
        self.state = Self::fetch_weather(self.config.location.clone(), self.logger);
    }

    fn render(&self, f: &mut Frame, area: Rect) {
        let style = Style::default().fg(Color::DarkGray);
        //.add_modifier(Modifier::ITALIC);

        let text = fontloader::to_styled_text(&self.font, &self.state, style);

        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::NONE));

        f.render_widget(paragraph, area);
    }
}
