/// Event creation modal with vim-style editing
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use crossterm::event::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalField {
    Title,
    Date,
    Time,
    Duration,
    Category,
    Description,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    Normal,
    Insert,
    Command,
}

/// State for the event creation modal
#[derive(Debug, Clone)]
pub struct EventModal {
    pub active: bool,
    pub focused_field: ModalField,
    pub vim_mode: VimMode,

    // Form fields
    pub title: String,
    pub date_input: String,
    pub time_input: String,
    pub duration_input: String,
    pub category_input: String,
    pub description: String,

    // Vim editing state
    pub cursor_pos: usize,
    pub command_buffer: String,

    // Edit mode tracking
    pub editing_event_id: Option<String>,
}

impl EventModal {
    pub fn new() -> Self {
        Self {
            active: false,
            focused_field: ModalField::Title,
            vim_mode: VimMode::Insert,
            title: String::new(),
            date_input: String::new(),
            time_input: String::new(),
            duration_input: String::from("60"),
            category_input: String::new(),
            description: String::new(),
            cursor_pos: 0,
            command_buffer: String::new(),
            editing_event_id: None,
        }
    }

    pub fn open(&mut self, initial_date: NaiveDate) {
        self.active = true;
        self.focused_field = ModalField::Title;
        self.vim_mode = VimMode::Insert;
        self.date_input = initial_date.format("%Y-%m-%d").to_string();
        self.time_input = String::from("09:00");
        self.cursor_pos = 0;
        self.command_buffer.clear();
        self.editing_event_id = None;
    }

    pub fn open_for_edit(&mut self, event: &crate::events::Event) {
        self.active = true;
        self.focused_field = ModalField::Title;
        self.vim_mode = VimMode::Insert;
        self.title = event.title.clone();
        self.date_input = event.start_time.format("%Y-%m-%d").to_string();
        self.time_input = event.start_time.format("%H:%M").to_string();
        self.duration_input = event.duration_minutes().to_string();
        self.category_input = event.category.as_ref().map(|c| c.name.clone()).unwrap_or_default();
        self.description = event.description.clone().unwrap_or_default();
        self.cursor_pos = 0;
        self.command_buffer.clear();
        self.editing_event_id = Some(event.id.clone());
    }

    pub fn is_editing(&self) -> bool {
        self.editing_event_id.is_some()
    }

    pub fn close(&mut self) {
        self.active = false;
        self.clear();
    }

    pub fn clear(&mut self) {
        self.title.clear();
        self.date_input.clear();
        self.time_input.clear();
        self.duration_input = String::from("60");
        self.category_input.clear();
        self.description.clear();
        self.cursor_pos = 0;
        self.command_buffer.clear();
        self.editing_event_id = None;
    }

    pub fn current_field_content(&self) -> &str {
        match self.focused_field {
            ModalField::Title => &self.title,
            ModalField::Date => &self.date_input,
            ModalField::Time => &self.time_input,
            ModalField::Duration => &self.duration_input,
            ModalField::Category => &self.category_input,
            ModalField::Description => &self.description,
        }
    }

    pub fn current_field_content_mut(&mut self) -> &mut String {
        match self.focused_field {
            ModalField::Title => &mut self.title,
            ModalField::Date => &mut self.date_input,
            ModalField::Time => &mut self.time_input,
            ModalField::Duration => &mut self.duration_input,
            ModalField::Category => &mut self.category_input,
            ModalField::Description => &mut self.description,
        }
    }

    pub fn next_field(&mut self) {
        self.focused_field = match self.focused_field {
            ModalField::Title => ModalField::Date,
            ModalField::Date => ModalField::Time,
            ModalField::Time => ModalField::Duration,
            ModalField::Duration => ModalField::Category,
            ModalField::Category => ModalField::Description,
            ModalField::Description => ModalField::Title,
        };
        self.cursor_pos = self.current_field_content().len();
    }

    pub fn prev_field(&mut self) {
        self.focused_field = match self.focused_field {
            ModalField::Title => ModalField::Description,
            ModalField::Date => ModalField::Title,
            ModalField::Time => ModalField::Date,
            ModalField::Duration => ModalField::Time,
            ModalField::Category => ModalField::Duration,
            ModalField::Description => ModalField::Category,
        };
        self.cursor_pos = self.current_field_content().len();
    }

    pub fn handle_key(&mut self, key: KeyCode) -> ModalAction {
        match self.vim_mode {
            VimMode::Normal => self.handle_normal_mode(key),
            VimMode::Insert => self.handle_insert_mode(key),
            VimMode::Command => self.handle_command_mode(key),
        }
    }

    fn handle_normal_mode(&mut self, key: KeyCode) -> ModalAction {
        match key {
            KeyCode::Char('i') => {
                self.vim_mode = VimMode::Insert;
                ModalAction::None
            }
            KeyCode::Char('a') => {
                self.vim_mode = VimMode::Insert;
                let len = self.current_field_content().len();
                self.cursor_pos = self.cursor_pos.min(len).saturating_add(1).min(len);
                ModalAction::None
            }
            KeyCode::Char('A') => {
                self.vim_mode = VimMode::Insert;
                self.cursor_pos = self.current_field_content().len();
                ModalAction::None
            }
            KeyCode::Char('I') => {
                self.vim_mode = VimMode::Insert;
                self.cursor_pos = 0;
                ModalAction::None
            }
            KeyCode::Char('h') => {
                self.cursor_pos = self.cursor_pos.saturating_sub(1);
                ModalAction::None
            }
            KeyCode::Char('l') => {
                let len = self.current_field_content().len();
                self.cursor_pos = self.cursor_pos.saturating_add(1).min(len.saturating_sub(1));
                ModalAction::None
            }
            KeyCode::Char('0') => {
                self.cursor_pos = 0;
                ModalAction::None
            }
            KeyCode::Char('$') => {
                self.cursor_pos = self.current_field_content().len().saturating_sub(1);
                ModalAction::None
            }
            KeyCode::Char('x') => {
                let cursor = self.cursor_pos;
                let content = self.current_field_content_mut();
                if cursor < content.len() {
                    content.remove(cursor);
                    if cursor >= content.len() && cursor > 0 {
                        self.cursor_pos -= 1;
                    }
                }
                ModalAction::None
            }
            KeyCode::Char('d') => {
                self.current_field_content_mut().clear();
                self.cursor_pos = 0;
                ModalAction::None
            }
            KeyCode::Char(':') => {
                self.vim_mode = VimMode::Command;
                self.command_buffer.clear();
                ModalAction::None
            }
            KeyCode::Char('j') | KeyCode::Down | KeyCode::Tab => {
                self.next_field();
                ModalAction::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.prev_field();
                ModalAction::None
            }
            KeyCode::Esc => ModalAction::Cancel,
            _ => ModalAction::None,
        }
    }

    fn handle_insert_mode(&mut self, key: KeyCode) -> ModalAction {
        match key {
            KeyCode::Esc => {
                self.vim_mode = VimMode::Normal;
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
                ModalAction::None
            }
            KeyCode::Char(c) => {
                let cursor = self.cursor_pos;
                let content = self.current_field_content_mut();
                if cursor <= content.len() {
                    content.insert(cursor, c);
                    self.cursor_pos += 1;
                }
                ModalAction::None
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    let cursor = self.cursor_pos;
                    self.current_field_content_mut().remove(cursor - 1);
                    self.cursor_pos -= 1;
                }
                ModalAction::None
            }
            KeyCode::Delete => {
                let cursor = self.cursor_pos;
                let content = self.current_field_content_mut();
                if cursor < content.len() {
                    content.remove(cursor);
                }
                ModalAction::None
            }
            KeyCode::Left => {
                self.cursor_pos = self.cursor_pos.saturating_sub(1);
                ModalAction::None
            }
            KeyCode::Right => {
                let len = self.current_field_content().len();
                self.cursor_pos = self.cursor_pos.saturating_add(1).min(len);
                ModalAction::None
            }
            KeyCode::Tab => {
                self.next_field();
                ModalAction::None
            }
            KeyCode::BackTab => {
                self.prev_field();
                ModalAction::None
            }
            _ => ModalAction::None,
        }
    }

    fn handle_command_mode(&mut self, key: KeyCode) -> ModalAction {
        match key {
            KeyCode::Esc => {
                self.vim_mode = VimMode::Normal;
                self.command_buffer.clear();
                ModalAction::None
            }
            KeyCode::Enter => {
                let action = self.execute_command();
                self.command_buffer.clear();
                self.vim_mode = VimMode::Normal;
                action
            }
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
                ModalAction::None
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
                ModalAction::None
            }
            _ => ModalAction::None,
        }
    }

    fn execute_command(&mut self) -> ModalAction {
        match self.command_buffer.as_str() {
            "w" | "wq" => ModalAction::Save,
            "q" | "q!" => ModalAction::Cancel,
            _ => ModalAction::None,
        }
    }

    pub fn validate_and_create_event(&self) -> Result<EventData, String> {
        if self.title.trim().is_empty() {
            return Err("Title is required".to_string());
        }

        let date = NaiveDate::parse_from_str(&self.date_input, "%Y-%m-%d")
            .map_err(|_| "Invalid date format (use YYYY-MM-DD)".to_string())?;

        let time = NaiveTime::parse_from_str(&self.time_input, "%H:%M")
            .map_err(|_| "Invalid time format (use HH:MM)".to_string())?;

        let start_time = NaiveDateTime::new(date, time);

        let duration_minutes: i64 = self.duration_input.parse()
            .map_err(|_| "Invalid duration (must be a number in minutes)".to_string())?;

        if duration_minutes <= 0 {
            return Err("Duration must be positive".to_string());
        }

        let end_time = start_time + chrono::Duration::minutes(duration_minutes);

        Ok(EventData {
            title: self.title.clone(),
            start_time,
            end_time,
            category: if self.category_input.trim().is_empty() {
                None
            } else {
                Some(self.category_input.clone())
            },
            description: if self.description.trim().is_empty() {
                None
            } else {
                Some(self.description.clone())
            },
        })
    }
}

impl Default for EventModal {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalAction {
    None,
    Save,
    Cancel,
}

#[derive(Debug, Clone)]
pub struct EventData {
    pub title: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub category: Option<String>,
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_creation() {
        let modal = EventModal::new();
        assert!(!modal.active);
        assert_eq!(modal.vim_mode, VimMode::Insert);
    }

    #[test]
    fn test_modal_open() {
        let mut modal = EventModal::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        modal.open(date);

        assert!(modal.active);
        assert_eq!(modal.date_input, "2024-01-15");
        assert_eq!(modal.time_input, "09:00");
    }

    #[test]
    fn test_field_navigation() {
        let mut modal = EventModal::new();
        assert_eq!(modal.focused_field, ModalField::Title);

        modal.next_field();
        assert_eq!(modal.focused_field, ModalField::Date);

        modal.next_field();
        assert_eq!(modal.focused_field, ModalField::Time);

        modal.prev_field();
        assert_eq!(modal.focused_field, ModalField::Date);
    }

    #[test]
    fn test_vim_mode_transitions() {
        let mut modal = EventModal::new();
        modal.vim_mode = VimMode::Normal;

        modal.handle_key(KeyCode::Char('i'));
        assert_eq!(modal.vim_mode, VimMode::Insert);

        modal.handle_key(KeyCode::Esc);
        assert_eq!(modal.vim_mode, VimMode::Normal);

        modal.handle_key(KeyCode::Char(':'));
        assert_eq!(modal.vim_mode, VimMode::Command);
    }

    #[test]
    fn test_text_insertion() {
        let mut modal = EventModal::new();
        modal.vim_mode = VimMode::Insert;
        modal.focused_field = ModalField::Title;

        modal.handle_key(KeyCode::Char('T'));
        modal.handle_key(KeyCode::Char('e'));
        modal.handle_key(KeyCode::Char('s'));
        modal.handle_key(KeyCode::Char('t'));

        assert_eq!(modal.title, "Test");
        assert_eq!(modal.cursor_pos, 4);
    }

    #[test]
    fn test_backspace() {
        let mut modal = EventModal::new();
        modal.vim_mode = VimMode::Insert;
        modal.title = String::from("Test");
        modal.cursor_pos = 4;

        modal.handle_key(KeyCode::Backspace);
        assert_eq!(modal.title, "Tes");
        assert_eq!(modal.cursor_pos, 3);
    }

    #[test]
    fn test_command_execution() {
        let mut modal = EventModal::new();
        modal.vim_mode = VimMode::Command;

        modal.command_buffer = String::from("w");
        let action = modal.execute_command();
        assert_eq!(action, ModalAction::Save);

        modal.command_buffer = String::from("q");
        let action = modal.execute_command();
        assert_eq!(action, ModalAction::Cancel);
    }

    #[test]
    fn test_event_validation() {
        let mut modal = EventModal::new();
        modal.title = String::from("Test Event");
        modal.date_input = String::from("2024-01-15");
        modal.time_input = String::from("10:00");
        modal.duration_input = String::from("60");

        let result = modal.validate_and_create_event();
        assert!(result.is_ok());

        let event_data = result.unwrap();
        assert_eq!(event_data.title, "Test Event");
    }

    #[test]
    fn test_event_validation_empty_title() {
        let mut modal = EventModal::new();
        modal.title = String::from("");
        modal.date_input = String::from("2024-01-15");
        modal.time_input = String::from("10:00");

        let result = modal.validate_and_create_event();
        assert!(result.is_err());
    }

    #[test]
    fn test_event_validation_invalid_date() {
        let mut modal = EventModal::new();
        modal.title = String::from("Test");
        modal.date_input = String::from("invalid");
        modal.time_input = String::from("10:00");

        let result = modal.validate_and_create_event();
        assert!(result.is_err());
    }
}
