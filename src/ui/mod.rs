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

pub struct App {
    pub should_quit: bool,
    pub state: CalendarState,
    pub month_view: MonthView,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            state: CalendarState::new(),
            month_view: MonthView::new(),
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
        let area = frame.area();
        self.month_view.render(frame, area, &self.state);
    }

    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Left => self.month_view.move_selection(-1),
            KeyCode::Right => self.month_view.move_selection(1),
            KeyCode::Up => self.month_view.move_selection(-7),
            KeyCode::Down => self.month_view.move_selection(7),
            KeyCode::Char('h') => self.month_view.move_selection(-1),
            KeyCode::Char('l') => self.month_view.move_selection(1),
            KeyCode::Char('k') => self.month_view.move_selection(-7),
            KeyCode::Char('j') => self.month_view.move_selection(7),
            KeyCode::Char('n') => self.state.next_period(),
            KeyCode::Char('p') => self.state.previous_period(),
            KeyCode::Char('t') => {
                self.state.go_to_today();
                self.month_view.select_date(self.state.current_date);
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
