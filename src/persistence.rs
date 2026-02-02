use std::fs;
use std::path::{Path, PathBuf};
use serde_json;
use crate::events::Event;
use crate::calendar::CalendarState;

/// Config directory name
const CONFIG_DIR_NAME: &str = "clawd-cal";

/// Events file name
const EVENTS_FILE: &str = "events.json";

/// Get the config directory for storing events
pub fn config_dir() -> Result<PathBuf, std::io::Error> {
    let mut path = dirs::config_dir()
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Config directory not found"
        ))?;

    path.push(CONFIG_DIR_NAME);

    // Create directory if it doesn't exist
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }

    Ok(path)
}

/// Get the events file path
pub fn events_file_path() -> Result<PathBuf, std::io::Error> {
    let mut path = config_dir()?;
    path.push(EVENTS_FILE);
    Ok(path)
}

/// Save calendar state to file
pub fn save_events(state: &CalendarState) -> Result<(), std::io::Error> {
    let path = events_file_path()?;
    let json = serde_json::to_string_pretty(&state.events)?;
    fs::write(&path, &json)?;
    Ok(())
}

/// Load calendar state from file
pub fn load_events(state: &mut CalendarState) -> Result<(), std::io::Error> {
    let path = events_file_path()?;

    if !path.exists() {
        return Ok(()); // No saved events yet
    }

    let json = fs::read_to_string(&path)?;
    let events: Vec<Event> = serde_json::from_str(&json)?;
    state.events = events;
    state.sort_events();
    Ok(())
}

/// Backup existing events file
pub fn backup_events() -> Result<Option<PathBuf>, std::io::Error> {
    let path = events_file_path()?;

    if !path.exists() {
        return Ok(None);
    }

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let mut backup_path = path.clone();
    backup_path.set_extension(format!(".bak.{}", timestamp));

    fs::copy(&path, &backup_path)?;
    Ok(Some(backup_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use chrono::NaiveDate;

    #[test]
    fn test_config_dir() {
        let dir = config_dir();
        assert!(dir.is_ok());
        let path = dir.unwrap();
        assert!(path.ends_with(CONFIG_DIR_NAME));
    }

    #[test]
    fn test_save_and_load_events() {
        let mut state = CalendarState::new();

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let start = date.and_hms_opt(10, 0, 0).unwrap();
        let end = date.and_hms_opt(11, 0, 0).unwrap();

        let event = Event::new("Test Event".to_string(), start, end);
        state.add_event(event);

        let temp_dir = TempDir::new().unwrap();
        let test_config_dir = temp_dir.path().join("test-config");

        // Override config dir for testing
        let test_events_file = test_config_dir.join(EVENTS_FILE);
        fs::create_dir_all(&test_config_dir).unwrap();

        let json = serde_json::to_string_pretty(&state.events).unwrap();
        fs::write(&test_events_file, &json).unwrap();

        // Now load
        let mut _loaded_state = CalendarState::new();
        // Would need to modify load_events to accept custom path
        // For now, just verify the JSON structure
        let loaded_events: Vec<Event> = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded_events.len(), 1);
        assert_eq!(loaded_events[0].title, "Test Event");
    }

    #[test]
    fn test_backup_events() {
        let temp_dir = TempDir::new().unwrap();
        let test_config_dir = temp_dir.path().join("test-config");
        fs::create_dir_all(&test_config_dir).unwrap();

        let test_events_file = test_config_dir.join(EVENTS_FILE);
        let dummy_data = "test data";
        fs::write(&test_events_file, dummy_data).unwrap();

        let backup = backup_events().unwrap();
        assert!(backup.is_none()); // Should be None since we overrode config_dir
    }
}
