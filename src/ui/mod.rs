/// UI components module
mod event_modal;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use crate::calendar::CalendarState;
use crate::views::MonthView;
use crate::events::{Event as CalendarEvent, Category};
use event_modal::{EventModal, ModalAction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    DatePrompt,
    EventModal,
}

pub struct App {
    pub should_quit: bool,
    pub state: CalendarState,
    pub month_view: MonthView,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub event_modal: EventModal,
    pub error_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            state: CalendarState::new(),
            month_view: MonthView::new(),
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            event_modal: EventModal::new(),
            error_message: None,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Main loop
        while !self.should_quit {
            terminal.draw(|frame| {
                self.render(frame);
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key.code, key.modifiers);
                }
            }
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        use ratatui::{
            layout::{Constraint, Direction, Layout},
            widgets::{Block, Borders, Paragraph},
        };

        let area = frame.area();

        // If in date prompt mode, show a prompt at the bottom
        if self.input_mode == InputMode::DatePrompt {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(area);

            self.month_view.render(frame, chunks[0], &self.state);

            let prompt_text = format!("Go to date (YYYY-MM-DD): {}", self.input_buffer);
            let prompt = Paragraph::new(prompt_text)
                .block(Block::default().borders(Borders::ALL).title(" Jump to Date "));
            frame.render_widget(prompt, chunks[1]);
        } else {
            self.month_view.render(frame, area, &self.state);
        }

        // Render modal if active
        if self.input_mode == InputMode::EventModal && self.event_modal.active {
            self.render_event_modal(frame, area);
        }
    }

    fn render_event_modal(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        use ratatui::{
            layout::{Constraint, Direction, Layout, Rect},
            widgets::{Block, Borders, Paragraph, Clear},
            style::{Color, Style},
        };

        // Create centered modal area
        let modal_width = 60.min(area.width.saturating_sub(4));
        let modal_height = 20.min(area.height.saturating_sub(4));

        let modal_area = Rect {
            x: (area.width.saturating_sub(modal_width)) / 2,
            y: (area.height.saturating_sub(modal_height)) / 2,
            width: modal_width,
            height: modal_height,
        };

        // Clear the area behind the modal
        frame.render_widget(Clear, modal_area);

        // Modal block
        let mode_indicator = match self.event_modal.vim_mode {
            event_modal::VimMode::Normal => "NORMAL",
            event_modal::VimMode::Insert => "INSERT",
            event_modal::VimMode::Command => "COMMAND",
        };

        let title = format!(" Create Event - {} ", mode_indicator);
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        // Layout for form fields
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Date
                Constraint::Length(3), // Time
                Constraint::Length(3), // Duration
                Constraint::Length(3), // Category
                Constraint::Min(3),    // Description
                Constraint::Length(2), // Help text
            ])
            .split(inner_area);

        // Render each field
        self.render_field(frame, chunks[0], "Title", &self.event_modal.title, event_modal::ModalField::Title);
        self.render_field(frame, chunks[1], "Date (YYYY-MM-DD)", &self.event_modal.date_input, event_modal::ModalField::Date);
        self.render_field(frame, chunks[2], "Time (HH:MM)", &self.event_modal.time_input, event_modal::ModalField::Time);
        self.render_field(frame, chunks[3], "Duration (minutes)", &self.event_modal.duration_input, event_modal::ModalField::Duration);
        self.render_field(frame, chunks[4], "Category", &self.event_modal.category_input, event_modal::ModalField::Category);
        self.render_field(frame, chunks[5], "Description", &self.event_modal.description, event_modal::ModalField::Description);

        // Help text
        let help_text = if self.event_modal.vim_mode == event_modal::VimMode::Command {
            format!(":{}", self.event_modal.command_buffer)
        } else {
            "Ctrl+S/:w=Save | Esc/:q=Cancel | Tab=Next | i=Insert | hjkl=Navigate".to_string()
        };

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, chunks[6]);

        // Show error message if any
        if let Some(err) = &self.error_message {
            let error_area = Rect {
                x: modal_area.x + 2,
                y: modal_area.y + modal_area.height,
                width: modal_area.width.saturating_sub(4),
                height: 1,
            };
            let error_text = Paragraph::new(err.as_str())
                .style(Style::default().fg(Color::Red));
            frame.render_widget(error_text, error_area);
        }
    }

    fn render_field(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect, label: &str, content: &str, field: event_modal::ModalField) {
        use ratatui::{
            widgets::{Block, Borders, Paragraph},
            style::{Color, Style},
        };

        let is_focused = self.event_modal.focused_field == field;
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let block = Block::default()
            .title(label)
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Show cursor if focused and in insert mode
        let display_content = if is_focused && self.event_modal.vim_mode == event_modal::VimMode::Insert {
            let cursor_pos = self.event_modal.cursor_pos.min(content.len());
            let before = &content[..cursor_pos];
            let after = &content[cursor_pos..];
            format!("{}|{}", before, after)
        } else {
            content.to_string()
        };

        let paragraph = Paragraph::new(display_content);
        frame.render_widget(paragraph, inner);
    }

    fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_mode(key),
            InputMode::DatePrompt => self.handle_date_prompt_mode(key),
            InputMode::EventModal => self.handle_event_modal_mode(key, modifiers),
        }
    }

    fn handle_normal_mode(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Left => self.month_view.move_selection(-1),
            KeyCode::Right => self.month_view.move_selection(1),
            KeyCode::Up => self.month_view.move_selection(-7),
            KeyCode::Down => self.month_view.move_selection(7),
            // Vim-style navigation
            KeyCode::Char('h') => self.month_view.move_selection(-1),
            KeyCode::Char('l') => self.month_view.move_selection(1),
            KeyCode::Char('k') => self.month_view.move_selection(-7),
            KeyCode::Char('j') => self.month_view.move_selection(7),
            // Week navigation
            KeyCode::Char('w') => self.month_view.move_to_week_start(),
            KeyCode::Char('e') => self.month_view.move_to_week_end(),
            // Month navigation
            KeyCode::Char('b') => {
                self.month_view.move_to_prev_month();
                self.state.previous_period();
            },
            KeyCode::Char('f') => {
                self.month_view.move_to_next_month();
                self.state.next_period();
            },
            KeyCode::Char('n') => self.state.next_period(),
            KeyCode::Char('p') => self.state.previous_period(),
            // Jump commands
            KeyCode::Char('g') => {
                self.input_mode = InputMode::DatePrompt;
                self.input_buffer.clear();
            },
            KeyCode::Char('G') => {
                self.month_view.jump_to_today();
                self.state.go_to_today();
            },
            KeyCode::Char('t') => {
                self.state.go_to_today();
                self.month_view.select_date(self.state.current_date);
            },
            // Open event creation modal
            KeyCode::Char('o') => {
                self.event_modal.open(self.month_view.selected_date);
                self.input_mode = InputMode::EventModal;
                self.error_message = None;
            },
            _ => {}
        }
    }

    fn handle_date_prompt_mode(&mut self, key: KeyCode) {
        use chrono::NaiveDate;

        match key {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
            },
            KeyCode::Enter => {
                // Try to parse the date
                if let Ok(date) = NaiveDate::parse_from_str(&self.input_buffer, "%Y-%m-%d") {
                    self.month_view.select_date(date);
                    self.state.current_date = date;
                }
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
            },
            KeyCode::Backspace => {
                self.input_buffer.pop();
            },
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            },
            _ => {}
        }
    }

    fn handle_event_modal_mode(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        // Check for Ctrl+S to save
        if modifiers.contains(KeyModifiers::CONTROL) && key == KeyCode::Char('s') {
            self.save_event();
            return;
        }

        // Handle modal key events
        let action = self.event_modal.handle_key(key);

        match action {
            ModalAction::Save => {
                self.save_event();
            }
            ModalAction::Cancel => {
                self.event_modal.close();
                self.input_mode = InputMode::Normal;
                self.error_message = None;
            }
            ModalAction::None => {}
        }
    }

    fn save_event(&mut self) {
        match self.event_modal.validate_and_create_event() {
            Ok(event_data) => {
                let event = CalendarEvent::new(
                    event_data.title,
                    event_data.start_time,
                    event_data.end_time,
                );

                let event = if let Some(category_name) = event_data.category {
                    event.with_category(Category::new(category_name))
                } else {
                    event
                };

                let event = if let Some(description) = event_data.description {
                    event.with_description(description)
                } else {
                    event
                };

                self.state.add_event(event);
                self.event_modal.close();
                self.input_mode = InputMode::Normal;
                self.error_message = None;
            }
            Err(err) => {
                self.error_message = Some(err);
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
