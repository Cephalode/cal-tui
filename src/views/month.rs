use chrono::{Datelike, NaiveDate};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::calendar::CalendarState;
use crate::ui::category_colors;

pub struct MonthView {
    pub selected_date: NaiveDate,
    pub selected_event_index: Option<usize>,
}

impl MonthView {
    pub fn new() -> Self {
        Self {
            selected_date: chrono::Local::now().date_naive(),
            selected_event_index: None,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, state: &CalendarState) {
        // Split the screen into calendar (left) and side panel (right)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        self.render_calendar(frame, chunks[0], state);
        self.render_side_panel(frame, chunks[1], state);
    }

    fn render_calendar(&self, frame: &mut Frame, area: Rect, state: &CalendarState) {
        let block = Block::default()
            .title(format!(
                " {} {} ",
                self.month_name(state.current_date.month()),
                state.current_date.year()
            ))
            .borders(Borders::ALL);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // Calculate calendar grid
        let days = self.get_month_days(state.current_date);

        // Split into header row and 6 week rows
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Header
                Constraint::Length(3),  // Week 1
                Constraint::Length(3),  // Week 2
                Constraint::Length(3),  // Week 3
                Constraint::Length(3),  // Week 4
                Constraint::Length(3),  // Week 5
                Constraint::Length(3),  // Week 6
            ])
            .split(inner_area);

        // Render header (day names)
        self.render_week_header(frame, vertical_chunks[0]);

        // Render weeks
        for (week_idx, week_chunk) in vertical_chunks[1..].iter().enumerate() {
            let week_start = week_idx * 7;
            let week_end = (week_start + 7).min(days.len());
            if week_start < days.len() {
                let week_days = &days[week_start..week_end];
                self.render_week(frame, *week_chunk, week_days, state);
            }
        }
    }

    fn render_week_header(&self, frame: &mut Frame, area: Rect) {
        let day_names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(14); 7])
            .split(area);

        for (i, name) in day_names.iter().enumerate() {
            let paragraph = Paragraph::new(*name)
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
            frame.render_widget(paragraph, chunks[i]);
        }
    }

    fn render_week(&self, frame: &mut Frame, area: Rect, days: &[Option<NaiveDate>], state: &CalendarState) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(14); 7])
            .split(area);

        let today = chrono::Local::now().date_naive();

        for (i, day_option) in days.iter().enumerate() {
            if let Some(day) = day_option {
                let is_today = *day == today;
                let is_selected = *day == self.selected_date;

                let events = state.events_on_date(*day);
                let has_events = !events.is_empty();

                let mut style = Style::default();
                if is_today {
                    style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
                }
                if is_selected {
                    style = style.bg(Color::DarkGray);
                }

                let day_str = format!("{:2}", day.day());

                // Get indicator color from first event's category
                let indicator_color = if has_events {
                    events[0].category
                        .as_ref()
                        .map(|c| category_colors::category_color(&c.name))
                        .unwrap_or(Color::Green)
                } else {
                    Color::Green
                };

                let indicator = if has_events { "*" } else { " " };

                let lines = vec![
                    Line::from(Span::styled(day_str, style)),
                    Line::from(Span::styled(indicator, Style::default().fg(indicator_color))),
                ];

                let paragraph = Paragraph::new(lines);
                frame.render_widget(paragraph, chunks[i]);
            }
        }
    }

    fn render_side_panel(&self, frame: &mut Frame, area: Rect, state: &CalendarState) {
        let events = state.events_on_date(self.selected_date);

        let title = format!(
            " {} ",
            self.selected_date.format("%b %d, %Y")
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
            let items: Vec<ListItem> = events
                .iter()
                .enumerate()
                .map(|(idx, event)| {
                    let time = event.start_time.format("%H:%M").to_string();

                    // Get category color
                    let cat_color = event.category
                        .as_ref()
                        .map(|c| category_colors::category_color(&c.name))
                        .unwrap_or(Color::Gray);

                    // Create styled content with category color
                    let content = Line::from(vec![
                        Span::styled(
                            format!("{} ", time),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled("● ", Style::default().fg(cat_color)),
                        Span::styled(
                            &event.title,
                            Style::default().fg(Color::White),
                        ),
                    ]);

                    let mut item = ListItem::new(content);

                    // Highlight selected event
                    if Some(idx) == self.selected_event_index {
                        item = item.style(Style::default().bg(Color::DarkGray));
                    }
                    item
                })
                .collect();

            let list = List::new(items);
            frame.render_widget(list, inner_area);
        }
    }

    fn get_month_days(&self, date: NaiveDate) -> Vec<Option<NaiveDate>> {
        let year = date.year();
        let month = date.month();

        // First day of the month
        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

        // Find the weekday of the first day (0 = Sunday, 6 = Saturday)
        let first_weekday = first_day.weekday().num_days_from_sunday() as usize;

        // Days in month
        let days_in_month = self.days_in_month(year, month);

        // Create 42-day grid (6 weeks * 7 days)
        let mut days = vec![None; 42];

        // Fill in the days
        for day in 1..=days_in_month {
            let index = first_weekday + day - 1;
            days[index] = NaiveDate::from_ymd_opt(year, month, day as u32);
        }

        days
    }

    fn days_in_month(&self, year: i32, month: u32) -> usize {
        let next_month = if month == 12 { 1 } else { month + 1 };
        let next_year = if month == 12 { year + 1 } else { year };

        let first_of_next = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
        let last_of_current = first_of_next.pred_opt().unwrap();

        last_of_current.day() as usize
    }

    fn month_name(&self, month: u32) -> &'static str {
        match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown",
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

    pub fn move_to_week_start(&mut self) {
        let weekday = self.selected_date.weekday().num_days_from_sunday();
        self.selected_date = self.selected_date
            .checked_sub_signed(chrono::Duration::days(weekday as i64))
            .unwrap_or(self.selected_date);
    }

    pub fn move_to_week_end(&mut self) {
        let weekday = self.selected_date.weekday().num_days_from_sunday();
        let days_to_saturday = 6 - weekday;
        self.selected_date = self.selected_date
            .checked_add_signed(chrono::Duration::days(days_to_saturday as i64))
            .unwrap_or(self.selected_date);
    }

    pub fn move_to_prev_month(&mut self) {
        let current_day = self.selected_date.day();
        let month = self.selected_date.month();
        let year = self.selected_date.year();

        let (prev_month, prev_year) = if month == 1 {
            (12, year - 1)
        } else {
            (month - 1, year)
        };

        // Get the number of days in the previous month
        let days_in_prev_month = self.days_in_month(prev_year, prev_month);
        let target_day = current_day.min(days_in_prev_month as u32);

        self.selected_date = NaiveDate::from_ymd_opt(prev_year, prev_month, target_day)
            .unwrap_or(self.selected_date);
    }

    pub fn move_to_next_month(&mut self) {
        let current_day = self.selected_date.day();
        let month = self.selected_date.month();
        let year = self.selected_date.year();

        let (next_month, next_year) = if month == 12 {
            (1, year + 1)
        } else {
            (month + 1, year)
        };

        // Get the number of days in the next month
        let days_in_next_month = self.days_in_month(next_year, next_month);
        let target_day = current_day.min(days_in_next_month as u32);

        self.selected_date = NaiveDate::from_ymd_opt(next_year, next_month, target_day)
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

    pub fn get_selected_event_id(&self, state: &CalendarState) -> Option<String> {
        let events = state.events_on_date(self.selected_date);
        self.selected_event_index
            .and_then(|idx| events.get(idx))
            .map(|event| event.id.clone())
    }

    pub fn clear_event_selection(&mut self) {
        self.selected_event_index = None;
    }
}

impl Default for MonthView {
    fn default() -> Self {
        Self::new()
    }
}
