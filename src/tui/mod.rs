mod app;
mod colors;
mod kanban;
mod compact;

pub use app::{App, ViewMode};
pub use colors::THEME;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

/// Run the TUI application
pub fn run(data_dir: std::path::PathBuf) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(data_dir)?;

    // Run app loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| app.render(f))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Tab => app.toggle_view(),
                    KeyCode::Char('n') => app.show_new_task_dialog(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous_task(),
                    KeyCode::Down | KeyCode::Char('j') => app.next_task(),
                    KeyCode::Enter => app.toggle_task_selection(),
                    KeyCode::Char('d') => app.mark_task_done()?,
                    KeyCode::Char('a') => app.archive_task()?,
                    KeyCode::Char('r') => app.refresh_tasks()?,
                    KeyCode::Char('1') => app.filter_by_tag("work"),
                    KeyCode::Char('2') => app.filter_by_tag("personal"),
                    KeyCode::Char('0') => app.clear_filters(),
                    _ => {}
                }
            }
        }
    }
}
