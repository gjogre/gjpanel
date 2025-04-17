use crate::widgets::GJWidget;
use std::env;
use std::process::Command;
pub struct WeatherWidget {
    state: String,
}
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
};

impl WeatherWidget {
    pub fn new() -> Self {
        Self {
            state: "Loading".to_string(),
        }
    }

    fn fetch_weather() -> String {
        match Command::new("sh").arg("-c").arg("./weather.sh").output() {
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
        self.state = Self::fetch_weather();
    }

    fn render(&self) -> Paragraph {
        let style = Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::ITALIC);

        let mut text = Text::default();
        for line in self.state.lines() {
            text.lines.push(Line::styled(line.to_string(), style));
        }

        Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Weather"))
    }
}
