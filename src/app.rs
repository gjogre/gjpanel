use crate::config::load_config;
use crate::widgets::{GJWidget, clock::ClockWidget, weather::WeatherWidget};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
}

impl App {
    pub fn run_app(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config = load_config("gjwidgets.toml");

        let mut widgets: Vec<(Box<dyn GJWidget>, Duration, Instant)> = vec![
            (
                Box::new(ClockWidget::new(config.clock)),
                Duration::from_secs(1),
                Instant::now(),
            ),
            (
                Box::new(WeatherWidget::new()),
                Duration::from_secs(3600),
                Instant::now(),
            ),
        ];

        let mut init = true;
        while !self.exit {
            for (widget, interval, last_polled) in widgets.iter_mut() {
                if last_polled.elapsed() >= *interval || init {
                    widget.poll();
                    *last_polled = Instant::now();
                }
            }

            terminal.draw(|f| {
                let size = f.area();
                let chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints(vec![
                        ratatui::layout::Constraint::Length(15),
                        ratatui::layout::Constraint::Length(30),
                    ])
                    .split(size);

                for ((widget, _, _), area) in widgets.iter().zip(chunks.iter()) {
                    f.render_widget(widget.render(), *area);
                }
            })?;

            self.handle_events()?;
            init = false;
            std::thread::sleep(Duration::from_millis(200));
        }

        Ok(())
    }
    fn handle_events(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }
    fn exit(&mut self) {
        self.exit = true;
    }
}
