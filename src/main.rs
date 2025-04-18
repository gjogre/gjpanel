mod app;
mod config;
mod fonts;
mod widgets;

use app::run_app;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();
    let res = run_app(&mut terminal);
    ratatui::restore();
    res
}
