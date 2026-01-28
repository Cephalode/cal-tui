use chrono::{NaiveDate, Datelike};

/// Represents a calendar view
#[derive(Debug, Clone)]
pub struct Calendar {
    pub current_date: NaiveDate,
}

impl Calendar {
    pub fn new() -> Self {
        Self {
            current_date: chrono::Local::now().date_naive(),
        }
    }

    pub fn month(&self) -> u32 {
        self.current_date.month()
    }

    pub fn year(&self) -> i32 {
        self.current_date.year()
    }
}

impl Default for Calendar {
    fn default() -> Self {
        Self::new()
    }
}
