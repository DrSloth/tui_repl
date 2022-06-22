use std::{io, ops::ControlFlow};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};

use tui_repl::{Repl, util as tui_repl_util};

fn main() -> std::io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    render(&mut terminal)?;

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

fn render<B: Backend>(term: &mut Terminal<B>) -> io::Result<()> {
    let mut texts = Vec::new();
    let mut repl = Repl::new();
    repl.text_mut().push('>');

    loop {
        term.draw(|f| {
            let block = Block::default()
                .borders(Borders::ALL & !Borders::RIGHT)
                .border_style(Style::default().fg(Color::White))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(Color::Blue));

            let size = f.size();
            let chunks = Layout::default()
            .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
                .split(size);
            f.render_widget(
                block
                    .clone()
                    .style(Style::default().bg(Color::Red))
                    .borders(Borders::ALL & !Borders::LEFT),
                chunks[1],
            );

            let out_block = block.inner(chunks[1]);
            let text = tui_repl_util::get_visible_text(texts.join("\n").as_ref(), out_block.height as usize);
            f.render_widget(
                Paragraph::new(text),
                out_block,
            );
            let repl_block = block.inner(chunks[0]);
            f.render_widget(block, chunks[0]);
            f.render_widget(&mut repl, repl_block);
            let (cursor_x, cursor_y) = repl.cursor_pos_in(repl_block);
            f.set_cursor(cursor_x + repl_block.left(), cursor_y + repl_block.top());
        })?;

        let mut executor = |cmd: String, out: &mut String| {
            let s = run_command(cmd, &mut texts);
            out.push_str(s);
            out.push_str("\n>");
            Ok(())
        };

        if let Event::Key(key) = event::read()? {
            match repl.feed_key_event(&mut executor, key)? {
                ControlFlow::Break(()) => return Ok(()),
                _ => (),
            }
        }
    }
}

fn run_command(cmd: String, texts: &mut Vec<String>) -> &'static str {
    let parts = cmd.split(' ').filter(|s| !s.is_empty()).collect::<Vec<_>>();
    match parts.get(0).map(|s| *s) {
        Some("add") => {
            texts.push(
                parts
                    .get(1..)
                    .map(|strs| strs.join(" "))
                    .unwrap_or_default(),
            );
            "\n>> Added text"
        }
        Some("rm" | "remove") => {
            if let Some(Ok(i)) = parts.get(1).map(|idx| idx.parse::<usize>()) {
                if i < texts.len() {
                    texts.remove(i);
                    return "\n>> Removed text";
                }
            }

            "\n>> Failed to remove text"
        }
        _ => "",
    }
}
