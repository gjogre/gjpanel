use crate::widgets::{GJWidget, clock::ClockWidget, weather::WeatherWidget};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::stdout;
use std::time::{Duration, Instant};

pub fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut widgets: Vec<(Box<dyn GJWidget>, Duration, Instant)> = vec![
        (
            Box::new(ClockWidget::new()),
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
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
