use super::Event;
use chrono::NaiveDateTime;
use std::collections::HashMap;

/// Error types for event store operations
#[derive(Debug, Clone, PartialEq)]
pub enum EventStoreError {
    NotFound(String),
    AlreadyExists(String),
    InvalidDateRange,
}

impl std::fmt::Display for EventStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventStoreError::NotFound(id) => write!(f, "Event not found: {}", id),
            EventStoreError::AlreadyExists(id) => write!(f, "Event already exists: {}", id),
            EventStoreError::InvalidDateRange => write!(f, "Invalid date range: start must be before end"),
        }
    }
}

impl std::error::Error for EventStoreError {}

/// Trait defining the interface for event storage operations
pub trait EventStore {
    /// Create a new event
    fn create(&mut self, event: Event) -> Result<String, EventStoreError>;

    /// Read an event by ID
    fn read(&self, id: &str) -> Result<&Event, EventStoreError>;

    /// Update an existing event
    fn update(&mut self, event: Event) -> Result<(), EventStoreError>;

    /// Delete an event by ID
    fn delete(&mut self, id: &str) -> Result<Event, EventStoreError>;

    /// List all events
    fn list(&self) -> Vec<&Event>;

    /// Query events by date range
    fn query_by_date_range(&self, start: NaiveDateTime, end: NaiveDateTime) -> Result<Vec<&Event>, EventStoreError>;

    /// Query events by category
    fn query_by_category(&self, category_name: &str) -> Vec<&Event>;

    /// Query events by date range and category
    fn query_by_date_range_and_category(
        &self,
        start: NaiveDateTime,
        end: NaiveDateTime,
        category_name: &str,
    ) -> Result<Vec<&Event>, EventStoreError>;
}

/// In-memory implementation of EventStore
#[derive(Debug, Clone)]
pub struct InMemoryEventStore {
    events: HashMap<String, Event>,
}

impl InMemoryEventStore {
    /// Create a new empty event store
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}

impl EventStore for InMemoryEventStore {
    fn create(&mut self, event: Event) -> Result<String, EventStoreError> {
        if self.events.contains_key(&event.id) {
            return Err(EventStoreError::AlreadyExists(event.id.clone()));
        }
        let id = event.id.clone();
        self.events.insert(id.clone(), event);
        Ok(id)
    }

    fn read(&self, id: &str) -> Result<&Event, EventStoreError> {
        self.events
            .get(id)
            .ok_or_else(|| EventStoreError::NotFound(id.to_string()))
    }

    fn update(&mut self, event: Event) -> Result<(), EventStoreError> {
        if !self.events.contains_key(&event.id) {
            return Err(EventStoreError::NotFound(event.id.clone()));
        }
        self.events.insert(event.id.clone(), event);
        Ok(())
    }

    fn delete(&mut self, id: &str) -> Result<Event, EventStoreError> {
        self.events
            .remove(id)
            .ok_or_else(|| EventStoreError::NotFound(id.to_string()))
    }

    fn list(&self) -> Vec<&Event> {
        let mut events: Vec<&Event> = self.events.values().collect();
        events.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        events
    }

    fn query_by_date_range(&self, start: NaiveDateTime, end: NaiveDateTime) -> Result<Vec<&Event>, EventStoreError> {
        if start > end {
            return Err(EventStoreError::InvalidDateRange);
        }

        let mut events: Vec<&Event> = self.events
            .values()
            .filter(|e| e.start_time >= start && e.start_time <= end)
            .collect();

        events.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        Ok(events)
    }

    fn query_by_category(&self, category_name: &str) -> Vec<&Event> {
        let mut events: Vec<&Event> = self.events
            .values()
            .filter(|e| {
                e.category
                    .as_ref()
                    .map(|c| c.name == category_name)
                    .unwrap_or(false)
            })
            .collect();

        events.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        events
    }

    fn query_by_date_range_and_category(
        &self,
        start: NaiveDateTime,
        end: NaiveDateTime,
        category_name: &str,
    ) -> Result<Vec<&Event>, EventStoreError> {
        if start > end {
            return Err(EventStoreError::InvalidDateRange);
        }

        let mut events: Vec<&Event> = self.events
            .values()
            .filter(|e| {
                let in_range = e.start_time >= start && e.start_time <= end;
                let matches_category = e.category
                    .as_ref()
                    .map(|c| c.name == category_name)
                    .unwrap_or(false);
                in_range && matches_category
            })
            .collect();

        events.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{Category, Event, Reminder};
    use chrono::NaiveDate;

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
    fn test_create_event() {
        let mut store = InMemoryEventStore::new();
        let event = create_test_event("Test Event", 15, 10);
        let id = event.id.clone();

        let result = store.create(event);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), id);
    }

    #[test]
    fn test_create_duplicate_event() {
        let mut store = InMemoryEventStore::new();
        let event = create_test_event("Test Event", 15, 10);
        let event_clone = event.clone();

        store.create(event).unwrap();
        let result = store.create(event_clone);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EventStoreError::AlreadyExists(_)));
    }

    #[test]
    fn test_read_event() {
        let mut store = InMemoryEventStore::new();
        let event = create_test_event("Test Event", 15, 10);
        let id = event.id.clone();

        store.create(event).unwrap();
        let result = store.read(&id);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().title, "Test Event");
    }

    #[test]
    fn test_read_nonexistent_event() {
        let store = InMemoryEventStore::new();
        let result = store.read("nonexistent");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EventStoreError::NotFound(_)));
    }

    #[test]
    fn test_update_event() {
        let mut store = InMemoryEventStore::new();
        let mut event = create_test_event("Original Title", 15, 10);
        let id = event.id.clone();

        store.create(event.clone()).unwrap();

        event.title = "Updated Title".to_string();
        let result = store.update(event);

        assert!(result.is_ok());
        assert_eq!(store.read(&id).unwrap().title, "Updated Title");
    }

    #[test]
    fn test_update_nonexistent_event() {
        let mut store = InMemoryEventStore::new();
        let event = create_test_event("Test Event", 15, 10);

        let result = store.update(event);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EventStoreError::NotFound(_)));
    }

    #[test]
    fn test_delete_event() {
        let mut store = InMemoryEventStore::new();
        let event = create_test_event("Test Event", 15, 10);
        let id = event.id.clone();

        store.create(event).unwrap();
        let result = store.delete(&id);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().title, "Test Event");
        assert!(store.read(&id).is_err());
    }

    #[test]
    fn test_delete_nonexistent_event() {
        let mut store = InMemoryEventStore::new();
        let result = store.delete("nonexistent");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EventStoreError::NotFound(_)));
    }

    #[test]
    fn test_list_events() {
        let mut store = InMemoryEventStore::new();

        store.create(create_test_event("Event 3", 17, 10)).unwrap();
        store.create(create_test_event("Event 1", 15, 10)).unwrap();
        store.create(create_test_event("Event 2", 16, 10)).unwrap();

        let events = store.list();

        assert_eq!(events.len(), 3);
        // Should be sorted by start_time
        assert_eq!(events[0].title, "Event 1");
        assert_eq!(events[1].title, "Event 2");
        assert_eq!(events[2].title, "Event 3");
    }

    #[test]
    fn test_query_by_date_range() {
        let mut store = InMemoryEventStore::new();

        store.create(create_test_event("Event 1", 10, 10)).unwrap();
        store.create(create_test_event("Event 2", 15, 10)).unwrap();
        store.create(create_test_event("Event 3", 20, 10)).unwrap();
        store.create(create_test_event("Event 4", 25, 10)).unwrap();

        let start = NaiveDate::from_ymd_opt(2024, 1, 12)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 22)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();

        let result = store.query_by_date_range(start, end);

        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].title, "Event 2");
        assert_eq!(events[1].title, "Event 3");
    }

    #[test]
    fn test_query_by_date_range_invalid() {
        let store = InMemoryEventStore::new();

        let start = NaiveDate::from_ymd_opt(2024, 1, 22)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 12)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();

        let result = store.query_by_date_range(start, end);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EventStoreError::InvalidDateRange));
    }

    #[test]
    fn test_query_by_category() {
        let mut store = InMemoryEventStore::new();

        let work_category = Category::new("Work".to_string());
        let personal_category = Category::new("Personal".to_string());

        let event1 = create_test_event("Event 1", 15, 10)
            .with_category(work_category.clone());
        let event2 = create_test_event("Event 2", 16, 10)
            .with_category(personal_category.clone());
        let event3 = create_test_event("Event 3", 17, 10)
            .with_category(work_category.clone());
        let event4 = create_test_event("Event 4", 18, 10); // No category

        store.create(event1).unwrap();
        store.create(event2).unwrap();
        store.create(event3).unwrap();
        store.create(event4).unwrap();

        let work_events = store.query_by_category("Work");
        assert_eq!(work_events.len(), 2);
        assert_eq!(work_events[0].title, "Event 1");
        assert_eq!(work_events[1].title, "Event 3");

        let personal_events = store.query_by_category("Personal");
        assert_eq!(personal_events.len(), 1);
        assert_eq!(personal_events[0].title, "Event 2");

        let nonexistent = store.query_by_category("Nonexistent");
        assert_eq!(nonexistent.len(), 0);
    }

    #[test]
    fn test_query_by_date_range_and_category() {
        let mut store = InMemoryEventStore::new();

        let work_category = Category::new("Work".to_string());
        let personal_category = Category::new("Personal".to_string());

        let event1 = create_test_event("Event 1", 10, 10)
            .with_category(work_category.clone());
        let event2 = create_test_event("Event 2", 15, 10)
            .with_category(work_category.clone());
        let event3 = create_test_event("Event 3", 20, 10)
            .with_category(personal_category.clone());
        let event4 = create_test_event("Event 4", 25, 10)
            .with_category(work_category.clone());

        store.create(event1).unwrap();
        store.create(event2).unwrap();
        store.create(event3).unwrap();
        store.create(event4).unwrap();

        let start = NaiveDate::from_ymd_opt(2024, 1, 12)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 22)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();

        let result = store.query_by_date_range_and_category(start, end, "Work");

        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].title, "Event 2");
    }

    #[test]
    fn test_query_by_date_range_and_category_invalid_range() {
        let store = InMemoryEventStore::new();

        let start = NaiveDate::from_ymd_opt(2024, 1, 22)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 12)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();

        let result = store.query_by_date_range_and_category(start, end, "Work");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EventStoreError::InvalidDateRange));
    }

    #[test]
    fn test_empty_store() {
        let store = InMemoryEventStore::new();

        assert_eq!(store.list().len(), 0);
        assert_eq!(store.query_by_category("Any").len(), 0);
    }
}
