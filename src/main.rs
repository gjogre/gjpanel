use app::App;
use logger::Logger;

mod app;
mod config;
mod fontloader;
mod logger;
mod widgets;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Box::new(Logger::new("rust-panel.log"));
    let logger_ref: &'static Logger = Box::leak(logger);

    let mut terminal = ratatui::init();
    let res = App::default().run_app(&mut terminal, logger_ref);
    ratatui::restore();
    res
}
