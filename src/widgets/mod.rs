use ratatui::{Frame, backend::CrosstermBackend};

pub trait GJWidget {
    fn update(&mut self);
    fn render(&self, f: &mut Frame<CrosstermBackend<std::io::Stdout>>, area: ratatui::layout::Rect);
}

pub mod clock;
pub mod suspend;
pub mod sysinfo;
pub mod weather;
pub mod workspace;
