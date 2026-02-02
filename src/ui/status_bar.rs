use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::calendar::ViewMode;
use crate::ui::InputMode;
use chrono::Local;

pub struct StatusBar {
    pub height: u16,
}

impl StatusBar {
    pub fn new() -> Self {
        Self { height: 2 }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, input_mode: InputMode, view_mode: ViewMode, selected_date: chrono::NaiveDate, event_count: usize) {
        // Create status bar area at bottom of screen
        let status_area = Rect {
            x: 0,
            y: area.height.saturating_sub(self.height),
            width: area.width,
            height: self.height,
        };

        // Split status bar into two lines
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(status_area);

        // Top line: view mode, date, event count
        self.render_top_line(frame, chunks[0], view_mode, selected_date, event_count);

        // Bottom line: input mode, time, hints
        self.render_bottom_line(frame, chunks[1], input_mode);
    }

    fn render_top_line(&self, frame: &mut Frame, area: Rect, view_mode: ViewMode, selected_date: chrono::NaiveDate, event_count: usize) {
        let view_name = match view_mode {
            ViewMode::Day => "Day",
            ViewMode::Week => "Week",
            ViewMode::Month => "Month",
            ViewMode::Year => "Year",
        };

        let date_str = selected_date.format("%a, %b %d, %Y").to_string();

        let line = Line::from(vec![
            Span::styled(
                format!(" {} ", view_name),
                Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("{}", date_str),
                Style::default().fg(Color::White),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("{} event{}", event_count, if event_count == 1 { "" } else { "s" }),
                Style::default().fg(Color::Yellow),
            ),
        ]);

        let paragraph = Paragraph::new(line)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));

        frame.render_widget(paragraph, area);
    }

    fn render_bottom_line(&self, frame: &mut Frame, area: Rect, input_mode: InputMode) {
        let mode_text = match input_mode {
            InputMode::Normal => "NORMAL",
            InputMode::DatePrompt => "GOTO",
            InputMode::EventModal => "MODAL",
            InputMode::DeleteConfirmation => "CONFIRM",
        };

        let (mode_color, hints) = match input_mode {
            InputMode::Normal => (
                Color::Green,
                "? Help | o New | ? for help",
            ),
            InputMode::DatePrompt => (
                Color::Yellow,
                "Enter: go | Esc: cancel",
            ),
            InputMode::EventModal => (
                Color::Cyan,
                "i/a: insert | Esc: normal | :w save | :q cancel",
            ),
            InputMode::DeleteConfirmation => (
                Color::Red,
                "y: confirm | n/Esc: cancel",
            ),
        };

        let now = Local::now().format("%H:%M:%S").to_string();

        let line = Line::from(vec![
            Span::styled(
                format!(" {} ", mode_text),
                Style::default().fg(mode_color).add_modifier(ratatui::style::Modifier::BOLD),
            ),
            Span::styled(
                format!(" {} ", hints),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw(" "),
            Span::styled(
                format!(" {} ", now),
                Style::default().fg(Color::DarkGray),
            ),
        ]);

        let paragraph = Paragraph::new(line);

        frame.render_widget(paragraph, area);
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}
