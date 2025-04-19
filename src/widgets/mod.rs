pub mod clock;
pub mod weather;
pub mod workspaces;
pub trait GJWidget {
    fn poll(&mut self) {}
    fn render(&self) -> ratatui::widgets::Paragraph;
}
