pub mod history;

pub mod util;

use std::{
    fmt::{self, Debug, Formatter},
    io,
    ops::ControlFlow,
};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
    Terminal,
};

use history::History;

// TODO add manual scrolling support
// TODO add removing complete words with ctrl + backspace/ctrl + del

// TODO termion support
// TODO maybe optimize to copy less text around?

#[derive(Default)]
pub struct Repl<const HISTORY_SIZE: usize> {
    current_input: Vec<char>,
    cursor_pos: u16,
    history: History<HISTORY_SIZE>,
    text: String,
}

impl Repl<32> {
    pub fn new() -> Self {
        Self::new_with_history(History::new())
    }

    pub fn new_run_fullscreen(executor: impl CommandExecutor) -> io::Result<()> {
        let mut me = Self::new();
        me.run_fullscreen(executor)
    }
}

impl<const HISTORY_SIZE: usize> Repl<HISTORY_SIZE> {
    pub fn new_with_history(history: History<HISTORY_SIZE>) -> Self {
        Self {
            current_input: Default::default(),
            cursor_pos: 0,
            history,
            text: Default::default(),
        }
    }

    pub fn run_fullscreen(&mut self, executor: impl CommandExecutor) -> io::Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        self.run_on_terminal(&mut terminal, executor)?;

        // restore terminal
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    pub fn run_on_terminal<B: Backend>(
        &mut self,
        term: &mut Terminal<B>,
        mut executor: impl CommandExecutor,
    ) -> io::Result<()> {
        loop {
            term.draw(|f| {
                let size = f.size();
                let (cursor_x, cursor_y) = self.cursor_pos_in(size);
                f.set_cursor(cursor_x, cursor_y);
                f.render_widget(&mut *self, size);
            })?;

            if let Event::Key(key) = event::read()? {
                match self.feed_key_event(&mut executor, key)? {
                    ControlFlow::Break(_) => return Ok(()),
                    _ => (),
                }
            }
        }
    }

    pub fn feed_key_event(
        &mut self,
        executor: &mut impl CommandExecutor,
        key: KeyEvent,
    ) -> io::Result<ControlFlow<()>> {
        match key {
            KeyEvent {
                code: KeyCode::Char('d' | 'q' | 'x'),
                modifiers: KeyModifiers::CONTROL,
            } => return Ok(ControlFlow::Break(())),
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            } => {
                self.text.extend(self.current_input.drain(..));
                self.text.push_str("^C");
                self.cursor_pos = 0;
                executor.execute(String::new(), &mut self.text)?;
            }
            KeyEvent {
                code: code @ (KeyCode::Up | KeyCode::Down),
                modifiers: KeyModifiers::NONE,
            } => {
                self.current_input = (if code == KeyCode::Up {
                    self.history.prev()
                } else {
                    self.history.next()
                }).unwrap_or(&[]).iter().copied().collect();
            }
            KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
            } => self.set_cursor_pos(self.cursor_pos.saturating_sub(1)),
            KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
            } => self.set_cursor_pos(self.cursor_pos.saturating_add(1)),
            KeyEvent {
                code: KeyCode::Home,
                modifiers: _,
            } => {
                self.set_cursor_pos(self.current_input.len() as u16);
            }
            KeyEvent {
                code: KeyCode::End,
                modifiers: _,
            } => {
                self.set_cursor_pos(0);
            }
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
            } => self
                .current_input
                .insert(self.current_input().len() - self.cursor_pos as usize, c),
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::SHIFT,
            } => {
                for c in c.to_uppercase() {
                    self.current_input.insert(self.cursor_pos as usize, c)
                }
            }
            KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            } => {
                self.set_cursor_pos(self.cursor_pos);
                let rm_idx = self.current_input.len() - self.cursor_pos as usize;
                if rm_idx != 0 {
                    self.current_input.remove(rm_idx - 1);
                }
            }
            KeyEvent {
                code: KeyCode::Delete,
                modifiers: KeyModifiers::NONE,
            } => {
                self.set_cursor_pos(self.cursor_pos);
                if self.cursor_pos != 0 {
                    self.current_input.remove(self.current_input.len() - self.cursor_pos as usize);
                    self.cursor_pos = self.cursor_pos.saturating_sub(1);
                }
            }
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            } => self.submit(executor)?,
            _ => (),
        }

        Ok(ControlFlow::Continue(()))
    }

    pub fn history(&self) -> &History<HISTORY_SIZE> {
        &self.history
    }

    pub fn history_mut(&mut self) -> &mut History<HISTORY_SIZE> {
        &mut self.history
    }

    pub fn current_input(&self) -> &[char] {
        &self.current_input
    }

    pub fn current_input_mut(&mut self) -> &mut Vec<char> {
        &mut self.current_input
    }

    pub fn cursor_pos_in(&self, rect: Rect) -> (u16, u16) {
        let mut lines = self.text.lines().rev().peekable();
        let last_line_len = lines.peek().map(|s| s.len()).unwrap_or(0);
        let max_height = rect.height.saturating_sub(rect.top());
        if self.text.ends_with('\n') {
            (
                self.current_input
                    .len()
                    .saturating_sub(self.cursor_pos as usize) as u16,
                (lines.count() as u16).clamp(0, max_height),
            )
        } else {
            (
                (last_line_len + self.current_input.len()).saturating_sub(self.cursor_pos as usize)
                    as u16,
                (lines.count().saturating_sub(1) as u16).clamp(0, max_height),
            )
        }
    }

    pub fn set_cursor_pos(&mut self, pos: u16) {
        self.cursor_pos = pos.clamp(0, self.current_input.len() as u16)
    }

    pub fn text(&self) -> &str {
        self.text.as_ref()
    }

    pub fn text_mut(&mut self) -> &mut String {
        &mut self.text
    }

    pub fn submit(&mut self, executor: &mut impl CommandExecutor) -> io::Result<()> {
        self.set_cursor_pos(0);
        self.history.push(self.current_input.iter().copied().collect());
        self.text.extend(self.current_input.iter());
        executor.execute(self.current_input.drain(..).collect(), &mut self.text)
    }
}

impl<const HISTORY_SIZE: usize> Debug for Repl<HISTORY_SIZE> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("Repl")
            .field("current_input", &self.current_input)
            .field("cursor_pos", &self.cursor_pos)
            .field("history", &self.history)
            .field("text", &self.text)
            .finish()
    }
}

pub trait CommandExecutor {
    fn execute<'a>(&mut self, command: String, repl_buffer: &mut String) -> io::Result<()>;
}

impl CommandExecutor for () {
    fn execute<'a>(&mut self, _command: String, _repl_buffer: &mut String) -> io::Result<()> {
        Ok(())
    }
}

impl<F: FnMut(String, &mut String) -> io::Result<()>> CommandExecutor for F {
    fn execute<'a>(&mut self, command: String, repl_buffer: &mut String) -> io::Result<()> {
        self(command, repl_buffer)
    }
}

impl<const HISTORY_SIZE: usize> Widget for &mut Repl<HISTORY_SIZE> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let max_height = area.height.saturating_sub(area.top());

        let prev_len = self.text.len();
        self.text.extend(self.current_input.iter());

        let p = Paragraph::new(util::get_visible_text(&self.text, max_height as usize));
        p.render(area, buf);
        self.text.truncate(prev_len);
    }
}
