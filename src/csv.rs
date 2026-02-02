use csv::ReaderBuilder;
use csv::WriterBuilder;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use crate::events::{Event, Category};
use crate::calendar::CalendarState;

#[derive(Debug)]
pub enum CsvError {
    Io(io::Error),
    Csv(csv::Error),
    Parse(String),
}

impl From<io::Error> for CsvError {
    fn from(err: io::Error) -> Self {
        CsvError::Io(err)
    }
}

impl From<csv::Error> for CsvError {
    fn from(err: csv::Error) -> Self {
        CsvError::Csv(err)
    }
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::Io(e) => write!(f, "IO error: {}", e),
            CsvError::Csv(e) => write!(f, "CSV error: {}", e),
            CsvError::Parse(s) => write!(f, "Parse error: {}", s),
        }
    }
}

/// Export all events to a CSV file
pub fn export_to_csv(state: &CalendarState, path: &Path) -> Result<(), CsvError> {
    let file = File::create(path)?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(file);

    writer.write_record(&["title", "start", "end", "description", "category"])?;

    for event in &state.events {
        let start_str = event.start_time.format("%Y-%m-%d %H:%M").to_string();
        let end_str = event.end_time.format("%Y-%m-%d %H:%M").to_string();
        let desc = event.description.as_ref().map(|s| s.as_str()).unwrap_or("");
        let cat = event.category.as_ref().map(|c| c.name.as_str()).unwrap_or("");

        let row = vec![
            &event.title,
            &start_str,
            &end_str,
            desc,
            cat,
        ];
        writer.write_record(&row)?;
    }

    writer.flush()?;
    Ok(())
}

/// Import events from a CSV file
pub fn import_from_csv(state: &mut CalendarState, path: &Path, replace: bool) -> Result<usize, CsvError> {
    let file = File::open(path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let headers = reader.headers()?.clone();

    // Get column indices
    let title_idx = headers.iter().position(|h| h == "title");
    let start_idx = headers.iter().position(|h| h == "start");
    let end_idx = headers.iter().position(|h| h == "end");
    let desc_idx = headers.iter().position(|h| h == "description");
    let cat_idx = headers.iter().position(|h| h == "category");

    if title_idx.is_none() || start_idx.is_none() || end_idx.is_none() {
        return Err(CsvError::Parse(
            "CSV must have headers: title, start, end".to_string()
        ));
    }

    let mut count = 0;

    if replace {
        state.events.clear();
    }

    for result in reader.records() {
        let record = result?;

        let title_idx_var = title_idx.unwrap();
        let title = record.get(title_idx_var).unwrap_or("").to_string();
        if title.is_empty() {
            continue; // Skip empty events
        }

        let start_idx_var = start_idx.unwrap();
        let start_str = record.get(start_idx_var).unwrap_or("").to_string();
        
        let end_idx_var = end_idx.unwrap();
        let end_str = record.get(end_idx_var).unwrap_or("").to_string();

        let start_time = chrono::NaiveDateTime::parse_from_str(
            &start_str,
            "%Y-%m-%d %H:%M",
        ).map_err(|_| CsvError::Parse(format!("Invalid start time: {}", start_str)))?;

        let end_time = chrono::NaiveDateTime::parse_from_str(
            &end_str,
            "%Y-%m-%d %H:%M",
        ).map_err(|_| CsvError::Parse(format!("Invalid end time: {}", end_str)))?;

        let description = if let Some(idx) = desc_idx {
            record.get(idx)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        } else {
            None
        };

        let category = if let Some(idx) = cat_idx {
            record.get(idx)
                .filter(|s| !s.is_empty())
                .map(|s| Category::new(s.to_string()))
        } else {
            None
        };

        let mut event = Event::new(title, start_time, end_time);

        if let Some(desc) = description {
            event = event.with_description(desc);
        }

        if let Some(cat) = category {
            event = event.with_category(cat);
        }

        state.add_event(event);
        count += 1;
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_csv() {
        let mut state = CalendarState::new();

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let start = date.and_hms_opt(10, 0, 0).unwrap();
        let end = date.and_hms_opt(11, 0, 0).unwrap();

        let event = Event::new("Test Event".to_string(), start, end);
        state.add_event(event);

        let temp_file = NamedTempFile::new().unwrap();
        export_to_csv(&state, temp_file.path()).unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("title,start,end,description,category"));
        assert!(content.contains("Test Event"));
    }

    #[test]
    fn test_import_csv() {
        let mut state = CalendarState::new();

        let temp_file = NamedTempFile::new().unwrap();
        let csv_content = "title,start,end,description,category
\"Test Event\",2024-01-15 10:00,2024-01-15 11:00,\"Test description\",\"Work\"";
        std::fs::write(temp_file.path(), csv_content).unwrap();

        let count = import_from_csv(&mut state, temp_file.path(), true).unwrap();
        assert_eq!(count, 1);
        assert_eq!(state.events.len(), 1);
        assert_eq!(state.events[0].title, "Test Event");
    }

    #[test]
    fn test_import_csv_merge() {
        let mut state = CalendarState::new();

        // Add existing event
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let start = date.and_hms_opt(9, 0, 0).unwrap();
        let end = date.and_hms_opt(10, 0, 0).unwrap();
        state.add_event(Event::new("Existing Event".to_string(), start, end));

        let temp_file = NamedTempFile::new().unwrap();
        let csv_content = "title,start,end,description,category
\"New Event\",2024-01-15 11:00,2024-01-15 12:00,\"Imported\",\"Personal\"";
        std::fs::write(temp_file.path(), csv_content).unwrap();

        let count = import_from_csv(&mut state, temp_file.path(), false).unwrap();
        assert_eq!(count, 1);
        assert_eq!(state.events.len(), 2); // Should merge
    }

    #[test]
    fn test_import_csv_replace() {
        let mut state = CalendarState::new();

        // Add existing event
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let start = date.and_hms_opt(9, 0, 0).unwrap();
        let end = date.and_hms_opt(10, 0, 0).unwrap();
        state.add_event(Event::new("Existing Event".to_string(), start, end));

        let temp_file = NamedTempFile::new().unwrap();
        let csv_content = "title,start,end,description,category
\"New Event\",2024-01-15 11:00,2024-01-15 12:00,\"Imported\",\"Personal\"";
        std::fs::write(temp_file.path(), csv_content).unwrap();

        let count = import_from_csv(&mut state, temp_file.path(), true).unwrap();
        assert_eq!(count, 1);
        assert_eq!(state.events.len(), 1); // Should replace
        assert_eq!(state.events[0].title, "New Event");
    }
}
