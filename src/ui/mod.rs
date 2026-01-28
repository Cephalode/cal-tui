/// UI components module
use crossterm::{
    event::{self, Event, KeyCode},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    DatePrompt,
}

pub struct App {
    pub should_quit: bool,
    pub state: CalendarState,
    pub month_view: MonthView,
    pub input_mode: InputMode,
    pub input_buffer: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            state: CalendarState::new(),
            month_view: MonthView::new(),
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
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
                    self.handle_key(key.code);
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
    }

    fn handle_key(&mut self, key: KeyCode) {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_mode(key),
            InputMode::DatePrompt => self.handle_date_prompt_mode(key),
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
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
