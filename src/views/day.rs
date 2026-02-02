use chrono::{Datelike, NaiveDate, Timelike};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::calendar::CalendarState;
use crate::ui::category_colors;

pub struct DayView {
    pub selected_date: NaiveDate,
    pub selected_event_index: Option<usize>,
}

impl DayView {
    pub fn new() -> Self {
        Self {
            selected_date: chrono::Local::now().date_naive(),
            selected_event_index: None,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, state: &CalendarState) {
        // Split into day view (left) and side panel (right)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        self.render_day(frame, chunks[0], state);
        self.render_side_panel(frame, chunks[1], state);
    }

    fn render_day(&self, frame: &mut Frame, area: Rect, state: &CalendarState) {
        let block = Block::default()
            .title(format!(
                " {} ",
                self.selected_date.format("%A, %B %d, %Y")
            ))
            .borders(Borders::ALL);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // Split into time slots (hourly from 6am to 10pm)
        let slots: Vec<Constraint> = (6..22)
            .map(|_| Constraint::Length(2)) // 2 lines per hour
            .collect();

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(slots.as_slice())
            .split(inner_area);

        let now = chrono::Local::now();
        let current_hour = now.hour() as i32;
        let current_date = now.date_naive();

        for (idx, hour) in (6..22).enumerate() {
            self.render_hour_slot(frame, vertical_chunks[idx], hour, state, hour == current_hour && self.selected_date == current_date);
        }
    }

    fn render_hour_slot(&self, frame: &mut Frame, area: Rect, hour: i32, state: &CalendarState, is_current_time: bool) {
        // Split into time label and event area
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(8),  // Time label
                Constraint::Min(0),      // Event area
            ])
            .split(area);

        // Render time label
        let time_str = format!("{:02}:00", hour);
        let time_style = if is_current_time {
            Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let time_paragraph = Paragraph::new(time_str).style(time_style);
        frame.render_widget(time_paragraph, chunks[0]);

        // Get events for this hour
        let events: Vec<_> = state.events_on_date(self.selected_date)
            .into_iter()
            .filter(|e| e.start_time.hour() as i32 == hour)
            .collect();

        if !events.is_empty() {
            // Render events
            let mut lines = vec![];

            for event in events.iter().take(2) { // Max 2 events per slot
                let cat_color = event.category
                    .as_ref()
                    .map(|c| category_colors::category_color(&c.name))
                    .unwrap_or(Color::Gray);

                let time = event.start_time.format("%H:%M").to_string();
                let truncated_title = if event.title.len() > 30 {
                    format!("{}...", &event.title[..27])
                } else {
                    event.title.clone()
                };

                lines.push(Line::from(vec![
                    Span::styled("● ", Style::default().fg(cat_color)),
                    Span::styled(format!("{} ", time), Style::default().fg(Color::DarkGray)),
                    Span::styled(truncated_title, Style::default().fg(Color::White)),
                ]));
            }

            if events.len() > 2 {
                lines.push(Line::from(Span::styled(
                    format!("  +{} more", events.len() - 2),
                    Style::default().fg(Color::DarkGray),
                )));
            }

            let events_paragraph = Paragraph::new(lines);
            frame.render_widget(events_paragraph, chunks[1]);
        } else if is_current_time {
            // Show current time indicator line
            let line = Paragraph::new("──────────")
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(line, chunks[1]);
        }
    }

    fn render_side_panel(&self, frame: &mut Frame, area: Rect, state: &CalendarState) {
        let events = state.events_on_date(self.selected_date);

        let title = format!(
            " {} Events ",
            events.len()
        );

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        if events.is_empty() {
            let paragraph = Paragraph::new("No events scheduled");
            frame.render_widget(paragraph, inner_area);
        } else {
            let mut lines = vec![];

            for (event_idx, event) in events.iter().enumerate() {
                let is_selected = Some(event_idx) == self.selected_event_index;
                let cat_color = event.category
                    .as_ref()
                    .map(|c| category_colors::category_color(&c.name))
                    .unwrap_or(Color::Gray);

                let time_range = format!(
                    "{} - {}",
                    event.start_time.format("%H:%M"),
                    event.end_time.format("%H:%M")
                );

                lines.push(Line::from(vec![
                    Span::styled("● ", Style::default().fg(cat_color)),
                    Span::styled(time_range, Style::default().fg(Color::DarkGray)),
                ]));

                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(&event.title, if is_selected {
                        Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    }),
                ]));

                // Add description if available
                if let Some(desc) = &event.description {
                    if !desc.is_empty() {
                        let truncated = if desc.len() > 40 {
                            format!("  {}...", &desc[..37])
                        } else {
                            format!("  {}", desc)
                        };
                        lines.push(Line::from(Span::styled(
                            truncated,
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                }

                lines.push(Line::from(Span::raw("")));
            }

            let paragraph = Paragraph::new(lines);
            frame.render_widget(paragraph, inner_area);
        }
    }

    pub fn select_date(&mut self, date: NaiveDate) {
        self.selected_date = date;
    }

    pub fn move_selection(&mut self, days: i64) {
        self.selected_date = self.selected_date
            .checked_add_signed(chrono::Duration::days(days))
            .unwrap_or(self.selected_date);
    }

    pub fn move_to_next_day(&mut self) {
        self.selected_date = self.selected_date
            .checked_add_signed(chrono::Duration::days(1))
            .unwrap_or(self.selected_date);
    }

    pub fn move_to_prev_day(&mut self) {
        self.selected_date = self.selected_date
            .checked_sub_signed(chrono::Duration::days(1))
            .unwrap_or(self.selected_date);
    }

    pub fn jump_to_today(&mut self) {
        self.selected_date = chrono::Local::now().date_naive();
    }

    pub fn select_next_event(&mut self, event_count: usize) {
        if event_count == 0 {
            self.selected_event_index = None;
            return;
        }

        self.selected_event_index = Some(match self.selected_event_index {
            Some(idx) if idx + 1 < event_count => idx + 1,
            Some(_) => 0,
            None => 0,
        });
    }

    pub fn select_prev_event(&mut self, event_count: usize) {
        if event_count == 0 {
            self.selected_event_index = None;
            return;
        }

        self.selected_event_index = Some(match self.selected_event_index {
            Some(0) => event_count - 1,
            Some(idx) => idx - 1,
            None => event_count - 1,
        });
    }
}

impl Default for DayView {
    fn default() -> Self {
        Self::new()
    }
}
