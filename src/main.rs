mod app;
mod fonts;
mod widgets;

use app::run_app;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_app()
}
