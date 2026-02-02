use chrono::{Datelike, NaiveDate};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Clear},
    Frame,
};
use crate::calendar::CalendarState;
use crate::ui::category_colors;

pub struct YearView {
    pub selected_date: NaiveDate,
    pub selected_month_index: usize,
}

impl YearView {
    pub fn new() -> Self {
        let now = chrono::Local::now().date_naive();
        Self {
            selected_date: now,
            selected_month_index: 0,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, state: &CalendarState) {
        let block = Block::default()
            .title(format!(
                " Year {} ",
                self.selected_date.year()
            ))
            .borders(Borders::ALL);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // Create 4x3 grid of months (4 rows, 3 columns)
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),
                Constraint::Length(6),
                Constraint::Length(6),
                Constraint::Length(6),
            ])
            .split(inner_area);

        let months_per_row = 3;

        for (row_idx, row_chunk) in vertical_chunks.iter().enumerate() {
            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(33); months_per_row])
                .split(*row_chunk);

            for (col_idx, month_chunk) in horizontal_chunks.iter().enumerate() {
                let month_idx = row_idx * months_per_row + col_idx;
                if month_idx < 12 {
                    self.render_month(frame, *month_chunk, month_idx, state);
                }
            }
        }
    }

    fn render_month(&self, frame: &mut Frame, area: Rect, month_idx: usize, state: &CalendarState) {
        let year = self.selected_date.year();

        // Get first day of this month
        let month_start = NaiveDate::from_ymd_opt(year, month_idx as u32 + 1, 1).unwrap();

        // Count events in this month
        let month_end = if month_idx == 11 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, month_idx as u32 + 2, 1).unwrap()
        };

        let event_count = state
            .events_in_range(month_start, month_end)
            .len();

        let is_selected = month_idx == self.selected_month_index;

        let mut style = Style::default();
        if is_selected {
            style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
        }

        // Highlight current month
        let now = chrono::Local::now().date_naive();
        let is_current_month = now.year() == year && now.month() as usize == month_idx + 1;
        if is_current_month {
            style = style.fg(Color::Yellow);
        }

        let month_name = self.month_name(month_idx);
        let event_indicator = if event_count > 0 {
            format!(" ({})", event_count)
        } else {
            String::new()
        };

        // Clear background if selected
        if is_selected {
            frame.render_widget(Clear, area);
        }

        let content = format!("{}\n{}", month_name, event_indicator);

        let paragraph = Paragraph::new(content)
            .alignment(Alignment::Center)
            .style(style);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });

        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(paragraph, inner);
    }

    fn month_name(&self, month_idx: usize) -> &'static str {
        match month_idx {
            0 => "Jan",
            1 => "Feb",
            2 => "Mar",
            3 => "Apr",
            4 => "May",
            5 => "Jun",
            6 => "Jul",
            7 => "Aug",
            8 => "Sep",
            9 => "Oct",
            10 => "Nov",
            11 => "Dec",
            _ => "???",
        }
    }

    pub fn select_month(&mut self, month_idx: usize) {
        self.selected_month_index = month_idx;
        self.selected_date = NaiveDate::from_ymd_opt(
            self.selected_date.year(),
            month_idx as u32 + 1,
            1
        ).unwrap_or(self.selected_date);
    }

    pub fn move_selection(&mut self, months: i64) {
        let new_idx = if months > 0 {
            self.selected_month_index + 1
        } else {
            self.selected_month_index.saturating_sub(1)
        };

        if new_idx < 12 {
            self.selected_month_index = new_idx;
        }

        self.select_month(self.selected_month_index);
    }

    pub fn move_to_next_year(&mut self) {
        let new_year = self.selected_date.year() + 1;
        self.selected_date = NaiveDate::from_ymd_opt(new_year, 1, 1).unwrap();
        self.selected_month_index = 0;
    }

    pub fn move_to_prev_year(&mut self) {
        let new_year = self.selected_date.year() - 1;
        self.selected_date = NaiveDate::from_ymd_opt(new_year, 1, 1).unwrap();
        self.selected_month_index = 0;
    }

    pub fn jump_to_today(&mut self) {
        let now = chrono::Local::now().date_naive();
        self.selected_date = NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();
        self.selected_month_index = (now.month() - 1) as usize;
    }
}

impl Default for YearView {
    fn default() -> Self {
        Self::new()
    }
}
