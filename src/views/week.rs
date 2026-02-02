use chrono::{Datelike, NaiveDate, Timelike};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Clear},
    Frame,
};
use crate::calendar::CalendarState;
use crate::ui::category_colors;

pub struct WeekView {
    pub selected_date: NaiveDate,
    pub selected_event_index: Option<usize>,
}

impl WeekView {
    pub fn new() -> Self {
        Self {
            selected_date: chrono::Local::now().date_naive(),
            selected_event_index: None,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, state: &CalendarState) {
        // Split into week view (left) and side panel (right)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        self.render_week(frame, chunks[0], state);
        self.render_side_panel(frame, chunks[1], state);
    }

    fn render_week(&self, frame: &mut Frame, area: Rect, state: &CalendarState) {
        let block = Block::default()
            .title(format!(
                " Week of {} ",
                self.week_range_string()
            ))
            .borders(Borders::ALL);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // Get the 7 days of the selected week
        let week_days = self.get_week_days(self.selected_date);

        // Split into header (day names) and time grid
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Header
                Constraint::Min(0),      // Time grid
            ])
            .split(inner_area);

        // Render day headers
        self.render_week_header(frame, vertical_chunks[0], &week_days);

        // Render time slots (simplified - showing 4 time blocks for now)
        let time_blocks = ["Morning", "Midday", "Afternoon", "Evening"];
        let num_blocks = time_blocks.len();

        for (idx, _time_block) in time_blocks.iter().enumerate() {
            let chunk_height = vertical_chunks[1].height / num_blocks as u16;
            let time_area = Rect {
                x: vertical_chunks[1].x,
                y: vertical_chunks[1].y + (idx as u16 * chunk_height),
                width: vertical_chunks[1].width,
                height: chunk_height,
            };

            self.render_time_block(frame, time_area, &week_days, state, idx);
        }
    }

    fn render_week_header(&self, frame: &mut Frame, area: Rect, week_days: &[NaiveDate]) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(14); 7])
            .split(area);

        let today = chrono::Local::now().date_naive();

        for (i, day) in week_days.iter().enumerate() {
            let is_today = *day == today;
            let is_selected = *day == self.selected_date;

            let mut style = Style::default();
            if is_today {
                style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
            }
            if is_selected {
                style = style.bg(Color::DarkGray);
            }

            let day_name = format!("{} {}", day.weekday(), day.day());
            let paragraph = Paragraph::new(day_name)
                .alignment(ratatui::layout::Alignment::Center)
                .style(style);

            frame.render_widget(paragraph, chunks[i]);
        }
    }

    fn render_time_block(
        &self,
        frame: &mut Frame,
        area: Rect,
        week_days: &[NaiveDate],
        state: &CalendarState,
        block_idx: usize,
    ) {
        // Split into time label and day columns
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(10),  // Time label
                Constraint::Min(0),      // Day columns
            ])
            .split(area);

        let time_labels = ["06:00-12:00", "12:00-15:00", "15:00-18:00", "18:00-24:00"];
        let time_text = format!(" {}", time_labels[block_idx]);

        let time_paragraph = Paragraph::new(time_text)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(time_paragraph, chunks[0]);

        // Split days into columns
        let day_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(14); 7])
            .split(chunks[1]);

        for (day_idx, day) in week_days.iter().enumerate() {
            let is_selected = *day == self.selected_date;

            // Get events for this day and time block
            let events = self.events_in_time_block(state, *day, block_idx);

            let mut lines = vec![];

            // Add indicator if this is the selected day
            if is_selected {
                frame.render_widget(Clear, day_chunks[day_idx]);
            }

            // Show events
            for event in events.iter().take(3) { // Max 3 events per block
                let cat_color = event.category
                    .as_ref()
                    .map(|c| category_colors::category_color(&c.name))
                    .unwrap_or(Color::Gray);

                let time = event.start_time.format("%H:%M").to_string();
                let truncated_title = if event.title.len() > 15 {
                    format!("{}...", &event.title[..12])
                } else {
                    event.title.clone()
                };

                lines.push(Line::from(vec![
                    Span::styled(format!("{} ", time), Style::default().fg(Color::DarkGray)),
                    Span::styled("● ", Style::default().fg(cat_color)),
                    Span::styled(truncated_title, Style::default().fg(Color::White)),
                ]));
            }

            if !lines.is_empty() || is_selected {
                let paragraph = Paragraph::new(lines);
                frame.render_widget(paragraph, day_chunks[day_idx]);
            }
        }
    }

    fn render_side_panel(&self, frame: &mut Frame, area: Rect, state: &CalendarState) {
        let events = state.events_on_date(self.selected_date);

        let title = format!(
            " {} ",
            self.selected_date.format("%a, %b %d, %Y")
        );

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        if events.is_empty() {
            let paragraph = Paragraph::new("No events");
            frame.render_widget(paragraph, inner_area);
        } else {
            let mut lines = vec![];

            for event in events.iter() {
                let time = event.start_time.format("%H:%M").to_string();
                let cat_color = event.category
                    .as_ref()
                    .map(|c| category_colors::category_color(&c.name))
                    .unwrap_or(Color::Gray);

                lines.push(Line::from(vec![
                    Span::styled(format!("{} ", time), Style::default().fg(Color::DarkGray)),
                    Span::styled("● ", Style::default().fg(cat_color)),
                    Span::styled(&event.title, Style::default().fg(Color::White)),
                ]));

                // Add description if available
                if let Some(desc) = &event.description {
                    if !desc.is_empty() {
                        let truncated = if desc.len() > 35 {
                            format!("{}...", &desc[..32])
                        } else {
                            desc.clone()
                        };
                        lines.push(Line::from(Span::styled(
                            format!("  {}", truncated),
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                }
            }

            let paragraph = Paragraph::new(lines);
            frame.render_widget(paragraph, inner_area);
        }
    }

    fn get_week_days(&self, date: NaiveDate) -> Vec<NaiveDate> {
        // Find Sunday of this week
        let weekday = date.weekday();
        let days_from_sunday = weekday.num_days_from_sunday() as i64;

        let sunday = date - chrono::Duration::days(days_from_sunday);

        // Generate 7 days
        (0..7)
            .map(|i| sunday + chrono::Duration::days(i))
            .collect()
    }

    fn week_range_string(&self) -> String {
        let week_days = self.get_week_days(self.selected_date);
        let start = week_days.first().unwrap();
        let end = week_days.last().unwrap();
        format!(
            "{} - {}",
            start.format("%b %d"),
            end.format("%b %d, %Y")
        )
    }

    fn events_in_time_block<'a>(&self, state: &'a CalendarState, date: NaiveDate, block_idx: usize) -> Vec<&'a crate::events::Event> {
        let events = state.events_on_date(date);

        let time_ranges = [
            (6, 12),  // Morning
            (12, 15), // Midday
            (15, 18), // Afternoon
            (18, 24), // Evening
        ];

        let (start_hour, end_hour) = time_ranges[block_idx];

        events
            .iter()
            .filter(|e| {
                let hour = e.start_time.hour();
                hour >= start_hour && hour < end_hour
            })
            .copied()
            .collect()
    }

    pub fn select_date(&mut self, date: NaiveDate) {
        self.selected_date = date;
    }

    pub fn move_selection(&mut self, days: i64) {
        self.selected_date = self.selected_date
            .checked_add_signed(chrono::Duration::days(days))
            .unwrap_or(self.selected_date);
    }

    pub fn move_to_next_week(&mut self) {
        self.selected_date = self.selected_date
            .checked_add_signed(chrono::Duration::weeks(1))
            .unwrap_or(self.selected_date);
    }

    pub fn move_to_prev_week(&mut self) {
        self.selected_date = self.selected_date
            .checked_sub_signed(chrono::Duration::weeks(1))
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

impl Default for WeekView {
    fn default() -> Self {
        Self::new()
    }
}
