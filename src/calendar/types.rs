use chrono::{NaiveDate, Datelike, Duration};
use serde::{Deserialize, Serialize};
use crate::events::Event;

/// Represents different view modes for the calendar
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    Day,
    Week,
    Month,
    Year,
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::Month
    }
}

/// Main state management for the calendar application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarState {
    pub current_date: NaiveDate,
    pub view_mode: ViewMode,
    pub events: Vec<Event>,
    pub selected_event_index: Option<usize>,
}

impl CalendarState {
    pub fn new() -> Self {
        Self {
            current_date: chrono::Local::now().date_naive(),
            view_mode: ViewMode::default(),
            events: Vec::new(),
            selected_event_index: None,
        }
    }

    /// Add an event to the calendar
    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
        self.sort_events();
    }

    /// Remove an event by ID
    pub fn remove_event(&mut self, event_id: &str) -> Option<Event> {
        if let Some(pos) = self.events.iter().position(|e| e.id == event_id) {
            Some(self.events.remove(pos))
        } else {
            None
        }
    }

    /// Get an event by ID
    pub fn get_event(&self, event_id: &str) -> Option<&Event> {
        self.events.iter().find(|e| e.id == event_id)
    }

    /// Get a mutable reference to an event by ID
    pub fn get_event_mut(&mut self, event_id: &str) -> Option<&mut Event> {
        self.events.iter_mut().find(|e| e.id == event_id)
    }

    /// Get all events for a specific date
    pub fn events_on_date(&self, date: NaiveDate) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|e| e.start_time.date() == date)
            .collect()
    }

    /// Get all events in a date range
    pub fn events_in_range(&self, start: NaiveDate, end: NaiveDate) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|e| {
                let event_date = e.start_time.date();
                event_date >= start && event_date <= end
            })
            .collect()
    }

    /// Change the view mode
    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    /// Navigate to next period based on current view mode
    pub fn next_period(&mut self) {
        self.current_date = match self.view_mode {
            ViewMode::Day => self.current_date + Duration::days(1),
            ViewMode::Week => self.current_date + Duration::weeks(1),
            ViewMode::Month => {
                let month = self.current_date.month();
                let year = self.current_date.year();
                if month == 12 {
                    NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
                }
            }
            ViewMode::Year => {
                NaiveDate::from_ymd_opt(self.current_date.year() + 1, 1, 1).unwrap()
            }
        };
    }

    /// Navigate to previous period based on current view mode
    pub fn previous_period(&mut self) {
        self.current_date = match self.view_mode {
            ViewMode::Day => self.current_date - Duration::days(1),
            ViewMode::Week => self.current_date - Duration::weeks(1),
            ViewMode::Month => {
                let month = self.current_date.month();
                let year = self.current_date.year();
                if month == 1 {
                    NaiveDate::from_ymd_opt(year - 1, 12, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(year, month - 1, 1).unwrap()
                }
            }
            ViewMode::Year => {
                NaiveDate::from_ymd_opt(self.current_date.year() - 1, 1, 1).unwrap()
            }
        };
    }

    /// Go to today's date
    pub fn go_to_today(&mut self) {
        self.current_date = chrono::Local::now().date_naive();
    }

    /// Select next event
    pub fn select_next_event(&mut self) {
        if self.events.is_empty() {
            self.selected_event_index = None;
            return;
        }

        self.selected_event_index = Some(match self.selected_event_index {
            Some(index) => {
                if index >= self.events.len() - 1 {
                    0
                } else {
                    index + 1
                }
            }
            None => 0,
        });
    }

    /// Select previous event
    pub fn select_previous_event(&mut self) {
        if self.events.is_empty() {
            self.selected_event_index = None;
            return;
        }

        self.selected_event_index = Some(match self.selected_event_index {
            Some(index) => {
                if index == 0 {
                    self.events.len() - 1
                } else {
                    index - 1
                }
            }
            None => self.events.len() - 1,
        });
    }

    /// Get currently selected event
    pub fn selected_event(&self) -> Option<&Event> {
        self.selected_event_index
            .and_then(|index| self.events.get(index))
    }

    /// Sort events by start time (public for use by persistence module)
    pub fn sort_events(&mut self) {
        self.events.sort_by(|a, b| a.start_time.cmp(&b.start_time));
    }

    /// Get total number of events
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

impl Default for CalendarState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event(title: &str, day: u32, hour: u32) -> Event {
        let start = NaiveDate::from_ymd_opt(2024, 1, day)
            .unwrap()
            .and_hms_opt(hour, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, day)
            .unwrap()
            .and_hms_opt(hour + 1, 0, 0)
            .unwrap();
        Event::new(title.to_string(), start, end)
    }

    #[test]
    fn test_calendar_state_creation() {
        let state = CalendarState::new();

        assert_eq!(state.view_mode, ViewMode::Month);
        assert_eq!(state.events.len(), 0);
        assert_eq!(state.selected_event_index, None);
    }

    #[test]
    fn test_add_event() {
        let mut state = CalendarState::new();
        let event = create_test_event("Test Event", 15, 10);

        state.add_event(event.clone());

        assert_eq!(state.events.len(), 1);
        assert_eq!(state.events[0].title, "Test Event");
    }

    #[test]
    fn test_remove_event() {
        let mut state = CalendarState::new();
        let event = create_test_event("Test Event", 15, 10);
        let event_id = event.id.clone();

        state.add_event(event);
        assert_eq!(state.events.len(), 1);

        let removed = state.remove_event(&event_id);
        assert!(removed.is_some());
        assert_eq!(state.events.len(), 0);
    }

    #[test]
    fn test_get_event() {
        let mut state = CalendarState::new();
        let event = create_test_event("Test Event", 15, 10);
        let event_id = event.id.clone();

        state.add_event(event);

        let found = state.get_event(&event_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Test Event");
    }

    #[test]
    fn test_events_on_date() {
        let mut state = CalendarState::new();

        state.add_event(create_test_event("Event 1", 15, 10));
        state.add_event(create_test_event("Event 2", 15, 14));
        state.add_event(create_test_event("Event 3", 16, 10));

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let events = state.events_on_date(date);

        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_events_in_range() {
        let mut state = CalendarState::new();

        state.add_event(create_test_event("Event 1", 10, 10));
        state.add_event(create_test_event("Event 2", 15, 10));
        state.add_event(create_test_event("Event 3", 20, 10));
        state.add_event(create_test_event("Event 4", 25, 10));

        let start = NaiveDate::from_ymd_opt(2024, 1, 12).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 22).unwrap();
        let events = state.events_in_range(start, end);

        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_view_mode_changes() {
        let mut state = CalendarState::new();

        assert_eq!(state.view_mode, ViewMode::Month);

        state.set_view_mode(ViewMode::Week);
        assert_eq!(state.view_mode, ViewMode::Week);

        state.set_view_mode(ViewMode::Day);
        assert_eq!(state.view_mode, ViewMode::Day);
    }

    #[test]
    fn test_next_period_day() {
        let mut state = CalendarState::new();
        state.current_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        state.set_view_mode(ViewMode::Day);

        state.next_period();
        assert_eq!(state.current_date, NaiveDate::from_ymd_opt(2024, 1, 16).unwrap());
    }

    #[test]
    fn test_next_period_month() {
        let mut state = CalendarState::new();
        state.current_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        state.set_view_mode(ViewMode::Month);

        state.next_period();
        assert_eq!(state.current_date, NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
    }

    #[test]
    fn test_next_period_year() {
        let mut state = CalendarState::new();
        state.current_date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        state.set_view_mode(ViewMode::Year);

        state.next_period();
        assert_eq!(state.current_date, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
    }

    #[test]
    fn test_previous_period_day() {
        let mut state = CalendarState::new();
        state.current_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        state.set_view_mode(ViewMode::Day);

        state.previous_period();
        assert_eq!(state.current_date, NaiveDate::from_ymd_opt(2024, 1, 14).unwrap());
    }

    #[test]
    fn test_previous_period_month() {
        let mut state = CalendarState::new();
        state.current_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        state.set_view_mode(ViewMode::Month);

        state.previous_period();
        assert_eq!(state.current_date, NaiveDate::from_ymd_opt(2023, 12, 1).unwrap());
    }

    #[test]
    fn test_event_selection() {
        let mut state = CalendarState::new();

        state.add_event(create_test_event("Event 1", 15, 10));
        state.add_event(create_test_event("Event 2", 16, 10));
        state.add_event(create_test_event("Event 3", 17, 10));

        assert_eq!(state.selected_event_index, None);

        state.select_next_event();
        assert_eq!(state.selected_event_index, Some(0));

        state.select_next_event();
        assert_eq!(state.selected_event_index, Some(1));

        state.select_previous_event();
        assert_eq!(state.selected_event_index, Some(0));
    }

    #[test]
    fn test_event_selection_wrapping() {
        let mut state = CalendarState::new();

        state.add_event(create_test_event("Event 1", 15, 10));
        state.add_event(create_test_event("Event 2", 16, 10));

        state.select_next_event();
        state.select_next_event();
        assert_eq!(state.selected_event_index, Some(1));

        // Should wrap to beginning
        state.select_next_event();
        assert_eq!(state.selected_event_index, Some(0));

        // Should wrap to end
        state.select_previous_event();
        assert_eq!(state.selected_event_index, Some(1));
    }

    #[test]
    fn test_selected_event() {
        let mut state = CalendarState::new();

        state.add_event(create_test_event("Event 1", 15, 10));
        state.add_event(create_test_event("Event 2", 16, 10));

        assert!(state.selected_event().is_none());

        state.select_next_event();
        let selected = state.selected_event();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().title, "Event 1");
    }

    #[test]
    fn test_event_sorting() {
        let mut state = CalendarState::new();

        state.add_event(create_test_event("Event 3", 17, 10));
        state.add_event(create_test_event("Event 1", 15, 10));
        state.add_event(create_test_event("Event 2", 16, 10));

        // Events should be sorted by start time
        assert_eq!(state.events[0].title, "Event 1");
        assert_eq!(state.events[1].title, "Event 2");
        assert_eq!(state.events[2].title, "Event 3");
    }

    #[test]
    fn test_event_count() {
        let mut state = CalendarState::new();

        assert_eq!(state.event_count(), 0);

        state.add_event(create_test_event("Event 1", 15, 10));
        assert_eq!(state.event_count(), 1);

        state.add_event(create_test_event("Event 2", 16, 10));
        assert_eq!(state.event_count(), 2);
    }

    #[test]
    fn test_calendar_state_serialization() {
        let mut state = CalendarState::new();
        state.add_event(create_test_event("Test Event", 15, 10));
        state.set_view_mode(ViewMode::Week);

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: CalendarState = serde_json::from_str(&json).unwrap();

        assert_eq!(state.view_mode, deserialized.view_mode);
        assert_eq!(state.events.len(), deserialized.events.len());
        assert_eq!(state.events[0].title, deserialized.events[0].title);
    }

    #[test]
    fn test_go_to_today() {
        let mut state = CalendarState::new();
        state.current_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();

        state.go_to_today();
        let today = chrono::Local::now().date_naive();
        assert_eq!(state.current_date, today);
    }
}
