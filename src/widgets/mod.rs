use ratatui::{Frame, layout::Rect};

pub mod clock;
pub mod sysinfo;
pub mod weather;
pub mod workspaces;
pub trait GJWidget {
    fn poll(&mut self) {}
    fn render(&self, frame: &mut Frame, area: Rect);
}
