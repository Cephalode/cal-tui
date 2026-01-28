/// Views module for different UI views
use ratatui::{
    layout::Rect,
    Frame,
};

pub trait View {
    fn render(&mut self, frame: &mut Frame, area: Rect);
}
