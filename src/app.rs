use crate::config::load_config;
use crate::logger::Logger;
use crate::widgets::sysinfo::SysInfoWidget;
use crate::widgets::{
    GJWidget, clock::ClockWidget, weather::WeatherWidget, workspaces::WorkspacesWidget,
};
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
        logger: &'static Logger,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config = load_config("gjwidgets.toml");

        let mut widgets: Vec<(Box<dyn GJWidget>, Duration, Instant)> = vec![
            (
                Box::new(ClockWidget::new(config.clock)),
                Duration::from_secs(1),
                Instant::now(),
            ),
            (
                Box::new(WeatherWidget::new(config.weather, logger)),
                Duration::from_secs(3600),
                Instant::now(),
            ),
            (
                Box::new(WorkspacesWidget::new(config.workspaces, logger)),
                Duration::from_millis(100),
                Instant::now(),
            ),
            (
                Box::new(SysInfoWidget::new(logger)),
                Duration::from_secs(2),
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
                        ratatui::layout::Constraint::Min(13),
                        ratatui::layout::Constraint::Max(3),
                        ratatui::layout::Constraint::Min(19),
                        ratatui::layout::Constraint::Min(20),
                    ])
                    .split(size);

                for ((widget, _, _), area) in widgets.iter().zip(chunks.iter()) {
                    widget.render(f, *area);
                }
            })?;

            self.handle_events(logger)?;
            init = false;
            std::thread::sleep(Duration::from_millis(20)); // Slightly faster sleep
        }

        Ok(())
    }

    fn handle_events(&mut self, logger: &Logger) -> Result<(), Box<dyn std::error::Error>> {
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event, logger)
                }
                _ => {}
            };
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, logger: &Logger) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(logger),
            _ => {}
        }
    }

    fn exit(&mut self, logger: &Logger) {
        logger.info("Exiting application");
        self.exit = true;
    }
}
