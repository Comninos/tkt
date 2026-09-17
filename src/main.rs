mod app;
mod db;
mod goals;
mod theme;
mod ui;
mod week;

use std::io::{self, stdout};
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event};
use crossterm::{execute, ExecutableCommand, QueueableCommand};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, BeginSynchronizedUpdate, EndSynchronizedUpdate,
    EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::App;
use crate::db::Db;

fn main() -> Result<()> {
    let db = Db::open_default().context("open database")?;
    let mut app = App::new(db)?;

    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        // Batch the whole frame so theme swaps (and other full redraws) present
        // atomically instead of painting top-to-bottom. Unsupported terminals
        // ignore these sequences.
        terminal.backend_mut().queue(BeginSynchronizedUpdate)?;
        terminal.draw(|frame| ui::draw(frame, app))?;
        terminal.backend_mut().execute(EndSynchronizedUpdate)?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Resize(_, _) => {}
                ev => app.handle_event(ev)?,
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(out);
    Terminal::new(backend).context("create terminal")
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
