/// Events module for managing calendar events
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

pub mod store;

/// Represents the frequency of a recurring event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecurrenceFrequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

/// Rules for recurring events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecurringRules {
    pub frequency: RecurrenceFrequency,
    pub interval: u32,
    pub count: Option<u32>,
    pub until: Option<NaiveDateTime>,
}

impl RecurringRules {
    pub fn new(frequency: RecurrenceFrequency, interval: u32) -> Self {
        Self {
            frequency,
            interval,
            count: None,
            until: None,
        }
    }

    pub fn with_count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }

    pub fn with_until(mut self, until: NaiveDateTime) -> Self {
        self.until = Some(until);
        self
    }
}

/// Category for organizing events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub color: Option<String>,
}

impl Category {
    pub fn new(name: String) -> Self {
        Self { name, color: None }
    }

    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }
}

/// Reminder for an event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reminder {
    pub minutes_before: u32,
    pub message: Option<String>,
}

impl Reminder {
    pub fn new(minutes_before: u32) -> Self {
        Self {
            minutes_before,
            message: None,
        }
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }
}

/// Main event structure with all required fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub title: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub description: Option<String>,
    pub location: Option<String>,
    pub recurring_rules: Option<RecurringRules>,
    pub category: Option<Category>,
    pub reminders: Vec<Reminder>,
}

impl Event {
    pub fn new(title: String, start_time: NaiveDateTime, end_time: NaiveDateTime) -> Self {
        Self {
            id: uuid::generate(),
            title,
            start_time,
            end_time,
            description: None,
            location: None,
            recurring_rules: None,
            category: None,
            reminders: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_recurring_rules(mut self, rules: RecurringRules) -> Self {
        self.recurring_rules = Some(rules);
        self
    }

    pub fn with_category(mut self, category: Category) -> Self {
        self.category = Some(category);
        self
    }

    pub fn add_reminder(mut self, reminder: Reminder) -> Self {
        self.reminders.push(reminder);
        self
    }

    /// Check if the event is recurring
    pub fn is_recurring(&self) -> bool {
        self.recurring_rules.is_some()
    }

    /// Get duration in minutes
    pub fn duration_minutes(&self) -> i64 {
        (self.end_time - self.start_time).num_minutes()
    }
}

mod uuid {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn generate() -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{:x}", timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_event_creation() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(11, 0, 0)
            .unwrap();

        let event = Event::new("Test Event".to_string(), start, end);

        assert_eq!(event.title, "Test Event");
        assert_eq!(event.start_time, start);
        assert_eq!(event.end_time, end);
        assert!(event.description.is_none());
        assert!(event.location.is_none());
        assert!(event.recurring_rules.is_none());
        assert!(event.category.is_none());
        assert!(event.reminders.is_empty());
        assert!(!event.id.is_empty());
    }

    #[test]
    fn test_event_with_builder_methods() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(11, 0, 0)
            .unwrap();

        let category = Category::new("Work".to_string()).with_color("blue".to_string());
        let reminder = Reminder::new(15).with_message("Meeting in 15 minutes".to_string());

        let event = Event::new("Team Meeting".to_string(), start, end)
            .with_description("Weekly team sync".to_string())
            .with_location("Conference Room A".to_string())
            .with_category(category.clone())
            .add_reminder(reminder.clone());

        assert_eq!(event.description, Some("Weekly team sync".to_string()));
        assert_eq!(event.location, Some("Conference Room A".to_string()));
        assert_eq!(event.category.as_ref().unwrap().name, "Work");
        assert_eq!(event.reminders.len(), 1);
        assert_eq!(event.reminders[0].minutes_before, 15);
    }

    #[test]
    fn test_recurring_event() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(11, 0, 0)
            .unwrap();

        let rules = RecurringRules::new(RecurrenceFrequency::Weekly, 1).with_count(10);

        let event = Event::new("Weekly Standup".to_string(), start, end)
            .with_recurring_rules(rules);

        assert!(event.is_recurring());
        assert_eq!(
            event.recurring_rules.as_ref().unwrap().frequency,
            RecurrenceFrequency::Weekly
        );
        assert_eq!(event.recurring_rules.as_ref().unwrap().count, Some(10));
    }

    #[test]
    fn test_event_duration() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(11, 30, 0)
            .unwrap();

        let event = Event::new("Test".to_string(), start, end);
        assert_eq!(event.duration_minutes(), 90);
    }

    #[test]
    fn test_event_serialization() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(11, 0, 0)
            .unwrap();

        let event = Event::new("Test Event".to_string(), start, end)
            .with_description("Test description".to_string());

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();

        assert_eq!(event.title, deserialized.title);
        assert_eq!(event.description, deserialized.description);
        assert_eq!(event.start_time, deserialized.start_time);
        assert_eq!(event.end_time, deserialized.end_time);
    }

    #[test]
    fn test_category_creation() {
        let category = Category::new("Personal".to_string()).with_color("red".to_string());

        assert_eq!(category.name, "Personal");
        assert_eq!(category.color, Some("red".to_string()));
    }

    #[test]
    fn test_reminder_creation() {
        let reminder = Reminder::new(30).with_message("Custom reminder".to_string());

        assert_eq!(reminder.minutes_before, 30);
        assert_eq!(reminder.message, Some("Custom reminder".to_string()));
    }

    #[test]
    fn test_recurring_rules_with_until() {
        let until = NaiveDate::from_ymd_opt(2024, 12, 31)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();

        let rules = RecurringRules::new(RecurrenceFrequency::Daily, 1).with_until(until);

        assert_eq!(rules.frequency, RecurrenceFrequency::Daily);
        assert_eq!(rules.interval, 1);
        assert_eq!(rules.until, Some(until));
        assert_eq!(rules.count, None);
    }
}
