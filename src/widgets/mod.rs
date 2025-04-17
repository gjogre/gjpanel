pub mod clock;
pub mod weather;

pub trait GJWidget {
    fn poll(&mut self) {}
    fn render(&self) -> ratatui::widgets::Paragraph;
}
