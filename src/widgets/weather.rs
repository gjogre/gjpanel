use crate::fontloader;
use crate::widgets::GJWidget;
use crate::{config::WeatherConfig, fontloader::load_font_by_name_or_err};
use std::env;
use std::process::Command;

use figlet_rs::FIGfont;
use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
};

pub struct WeatherWidget {
    config: WeatherConfig,
    font: FIGfont,
    state: String,
}

impl WeatherWidget {
    pub fn new(config: WeatherConfig) -> Self {
        let font = load_font_by_name_or_err(&config.font);
        Self {
            state: "Loading".to_string(),
            config,
            font,
        }
    }

    fn fetch_weather(location: String) -> String {
        match Command::new("sh")
            .arg("-c")
            .arg(format!("./sh/weather.sh {}", location))
            .output()
        {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            }
            Ok(output) => {
                format!(
                    "⚠️ Script error: {}",
                    String::from_utf8_lossy(&output.stderr)
                        + "script path: "
                        + env::current_dir().unwrap().to_string_lossy(),
                )
            }
            Err(err) => format!("⚠️ Exec failed: {err}"),
        }
    }
}

impl GJWidget for WeatherWidget {
    fn poll(&mut self) {
        self.state = Self::fetch_weather(self.config.location.clone());
    }

    fn render(&self) -> Paragraph {
        let style = Style::default().fg(Color::Blue);
        //.add_modifier(Modifier::ITALIC);

        let mut text = Text::default();
        let state_fig = fontloader::render_figlet_text(&self.font, &self.state);
        for line in state_fig.to_string().lines() {
            text.lines.push(Line::styled(line.to_string(), style));
        }

        Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Weather"))
    }
}
