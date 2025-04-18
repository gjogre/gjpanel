use crate::config::load_config;
use crate::widgets::{GJWidget, clock::ClockWidget, weather::WeatherWidget};
use crossterm::event::{self, KeyCode};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::Stdout;
use std::time::{Duration, Instant};

pub fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
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
    for (widget, _interval, _last_polled) in widgets.iter_mut() {
        widget.poll();
    }
    loop {
        if event::poll(Duration::from_millis(50))? {
            if let event::Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        for (widget, interval, last_polled) in widgets.iter_mut() {
            if last_polled.elapsed() >= *interval {
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

        std::thread::sleep(Duration::from_millis(200));
    }

    Ok(())
}
