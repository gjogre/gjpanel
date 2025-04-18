use app::App;

mod app;
mod config;
mod fonts;
mod widgets;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();
    let res = App::default().run_app(&mut terminal);
    ratatui::restore();
    res
}
