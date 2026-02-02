use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct HelpView {
    pub active: bool,
}

impl HelpView {
    pub fn new() -> Self {
        Self { active: false }
    }

    pub fn toggle(&mut self) {
        self.active = !self.active;
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Create centered modal area
        let modal_width = 70.min(area.width.saturating_sub(4));
        let modal_height = 40.min(area.height.saturating_sub(4));

        let modal_area = Rect {
            x: (area.width.saturating_sub(modal_width)) / 2,
            y: (area.height.saturating_sub(modal_height)) / 2,
            width: modal_width,
            height: modal_height,
        };

        // Clear the area behind the modal
        frame.render_widget(ratatui::widgets::Clear, modal_area);

        // Modal block
        let block = Block::default()
            .title(" Keybindings & Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        // Layout for help content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Spacer
                Constraint::Length(3),  // Navigation header
                Constraint::Length(8),  // Navigation keys
                Constraint::Length(1),  // Spacer
                Constraint::Length(3),  // Event management header
                Constraint::Length(6),  // Event keys
                Constraint::Length(1),  // Spacer
                Constraint::Length(3),  // Modal editing header
                Constraint::Length(8),  // Modal keys
                Constraint::Length(1),  // Spacer
                Constraint::Length(2),  // Footer
            ])
            .split(inner_area);

        // Headers
        let header_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        frame.render_widget(
            Paragraph::new("Navigation").style(header_style),
            chunks[1]
        );

        // Navigation keys
        let nav_lines = vec![
            Line::from(vec![
                Span::styled(" h / ← ", Style::default().fg(Color::Green)),
                Span::raw(" Move left  "),
                Span::styled(" l / → ", Style::default().fg(Color::Green)),
                Span::raw(" Move right"),
            ]),
            Line::from(vec![
                Span::styled(" j / ↓ ", Style::default().fg(Color::Green)),
                Span::raw(" Move down  "),
                Span::styled(" k / ↑ ", Style::default().fg(Color::Green)),
                Span::raw(" Move up"),
            ]),
            Line::from(vec![
                Span::styled(" w ", Style::default().fg(Color::Green)),
                Span::raw(" Week start  "),
                Span::styled(" e ", Style::default().fg(Color::Green)),
                Span::raw(" Week end"),
            ]),
            Line::from(vec![
                Span::styled(" b ", Style::default().fg(Color::Green)),
                Span::raw(" Prev month  "),
                Span::styled(" f ", Style::default().fg(Color::Green)),
                Span::raw(" Next month"),
            ]),
            Line::from(vec![
                Span::styled(" G ", Style::default().fg(Color::Green)),
                Span::raw(" Go to today  "),
                Span::styled(" g ", Style::default().fg(Color::Green)),
                Span::raw(" Jump to date"),
            ]),
            Line::from(vec![
                Span::styled(" Tab/Shift+Tab ", Style::default().fg(Color::Green)),
                Span::raw(" Navigate events"),
            ]),
            Line::from(vec![
                Span::styled(" q / Esc ", Style::default().fg(Color::Green)),
                Span::raw(" Quit"),
            ]),
        ];

        frame.render_widget(
            Paragraph::new(nav_lines).style(Style::default().fg(Color::White)),
            chunks[2]
        );

        // Event management header
        frame.render_widget(
            Paragraph::new("Event Management").style(header_style),
            chunks[4]
        );

        // Event management keys
        let event_lines = vec![
            Line::from(vec![
                Span::styled(" o ", Style::default().fg(Color::Green)),
                Span::raw(" Create new event"),
            ]),
            Line::from(vec![
                Span::styled(" E / Enter ", Style::default().fg(Color::Green)),
                Span::raw(" Edit selected event"),
            ]),
            Line::from(vec![
                Span::styled(" d ", Style::default().fg(Color::Green)),
                Span::raw(" Delete selected event"),
            ]),
            Line::from(vec![
                Span::styled(" y ", Style::default().fg(Color::Green)),
                Span::raw(" Confirm delete"),
            ]),
            Line::from(vec![
                Span::styled(" n / Esc ", Style::default().fg(Color::Green)),
                Span::raw(" Cancel operation"),
            ]),
        ];

        frame.render_widget(
            Paragraph::new(event_lines).style(Style::default().fg(Color::White)),
            chunks[5]
        );

        // Modal editing header
        frame.render_widget(
            Paragraph::new("Modal Editing (Vim-style)").style(header_style),
            chunks[7]
        );

        // Modal editing keys
        let modal_lines = vec![
            Line::from(vec![
                Span::styled(" i ", Style::default().fg(Color::Green)),
                Span::raw(" Insert mode (before cursor)  "),
                Span::styled(" a ", Style::default().fg(Color::Green)),
                Span::raw(" Append after cursor"),
            ]),
            Line::from(vec![
                Span::styled(" A ", Style::default().fg(Color::Green)),
                Span::raw(" Append at end  "),
                Span::styled(" I ", Style::default().fg(Color::Green)),
                Span::raw(" Insert at start"),
            ]),
            Line::from(vec![
                Span::styled(" Esc ", Style::default().fg(Color::Green)),
                Span::raw(" Normal mode  "),
                Span::styled(" Tab ", Style::default().fg(Color::Green)),
                Span::raw(" Next field"),
            ]),
            Line::from(vec![
                Span::styled(" :w ", Style::default().fg(Color::Green)),
                Span::raw(" Save  "),
                Span::styled(" :q ", Style::default().fg(Color::Green)),
                Span::raw(" Cancel"),
            ]),
            Line::from(vec![
                Span::styled(" Ctrl+S ", Style::default().fg(Color::Green)),
                Span::raw(" Quick save"),
            ]),
        ];

        frame.render_widget(
            Paragraph::new(modal_lines).style(Style::default().fg(Color::White)),
            chunks[8]
        );

        // Footer
        let footer = "Press ? or Esc to close this help";
        frame.render_widget(
            Paragraph::new(footer)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray)),
            chunks[10]
        );
    }
}

impl Default for HelpView {
    fn default() -> Self {
        Self::new()
    }
}
