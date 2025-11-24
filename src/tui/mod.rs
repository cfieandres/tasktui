mod app;
mod colors;
mod kanban;
mod compact;
mod settings;
mod projects;
mod project_gantt;

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
                // Handle dialog inputs first
                if app.show_new_task {
                    match key.code {
                        KeyCode::Esc => app.cancel_new_task_dialog(),
                        KeyCode::Enter => app.create_new_task()?,
                        KeyCode::Backspace => { app.new_task_title.pop(); }
                        KeyCode::Char(c) => app.new_task_title.push(c),
                        _ => {}
                    }
                } else if app.show_new_project {
                    match key.code {
                        KeyCode::Esc => app.cancel_new_project_dialog(),
                        KeyCode::Enter => app.create_new_project()?,
                        KeyCode::Backspace => { app.new_project_title.pop(); }
                        KeyCode::Char(c) => app.new_project_title.push(c),
                        _ => {}
                    }
                } else if app.settings_editing {
                    match key.code {
                        KeyCode::Esc => app.settings_cancel_edit(),
                        KeyCode::Enter => app.settings_confirm_edit()?,
                        KeyCode::Backspace => { app.settings_edit_text.pop(); }
                        KeyCode::Char(c) => app.settings_edit_text.push(c),
                        _ => {}
                    }
                } else {
                    // View-specific handling
                    match app.view_mode {
                        ViewMode::Settings => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => app.close_settings(),
                            KeyCode::Up | KeyCode::Char('k') => app.settings_prev(),
                            KeyCode::Down | KeyCode::Char('j') => app.settings_next(),
                            KeyCode::Enter => app.settings_start_edit(),
                            KeyCode::Char('x') | KeyCode::Delete => app.settings_delete()?,
                            _ => {}
                        },
                        ViewMode::Projects => match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Esc => app.close_projects(),
                            KeyCode::Up | KeyCode::Char('k') => app.projects_prev(),
                            KeyCode::Down | KeyCode::Char('j') => app.projects_next(),
                            KeyCode::Enter => app.open_project_gantt(),
                            KeyCode::Char('n') => app.show_new_project_dialog(),
                            _ => {}
                        },
                        ViewMode::ProjectGantt => match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Esc => app.close_project_gantt(),
                            KeyCode::Up | KeyCode::Char('k') => app.gantt_prev(),
                            KeyCode::Down | KeyCode::Char('j') => app.gantt_next(),
                            KeyCode::Left | KeyCode::Char('h') => app.gantt_scroll_left(),
                            KeyCode::Right | KeyCode::Char('l') => app.gantt_scroll_right(),
                            _ => {}
                        },
                        _ => {
                            // Global keys for Compact and Kanban views
                            match key.code {
                                KeyCode::Char('q') => return Ok(()),
                                KeyCode::Tab => app.toggle_view(),
                                KeyCode::Char('n') => app.show_new_task_dialog(),
                                KeyCode::Char('r') => app.refresh_tasks()?,
                                KeyCode::Char('s') => app.open_settings(),
                                KeyCode::Char('p') => app.open_projects(),
                                KeyCode::Char('0') => app.clear_filters(),
                                _ => {
                                    // Check for dynamic workstream shortcuts
                                    if let KeyCode::Char(c) = key.code {
                                        if let Some(ws) = app.config.get_workstream_by_key(c) {
                                            app.filter_by_tag(&ws.name.clone());
                                        } else {
                                            handle_view_keys(app, key.code)?;
                                        }
                                    } else {
                                        handle_view_keys(app, key.code)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn handle_view_keys(app: &mut App, code: KeyCode) -> Result<()> {
    match app.view_mode {
        ViewMode::Compact => match code {
            KeyCode::Up | KeyCode::Char('k') => app.previous_task(),
            KeyCode::Down | KeyCode::Char('j') => app.next_task(),
            KeyCode::Enter => app.toggle_task_selection(),
            KeyCode::Char('d') => app.mark_task_done()?,
            KeyCode::Char('a') => app.archive_task()?,
            _ => {}
        },
        ViewMode::Kanban => match code {
            KeyCode::Up | KeyCode::Char('k') => app.kanban_move_up(),
            KeyCode::Down | KeyCode::Char('j') => app.kanban_move_down(),
            KeyCode::Left | KeyCode::Char('h') => app.kanban_move_left(),
            KeyCode::Right | KeyCode::Char('l') => app.kanban_move_right(),
            KeyCode::Char('d') => app.kanban_mark_done()?,
            KeyCode::Char('a') => app.kanban_archive_task()?,
            _ => {}
        },
        _ => {} // Other views handled above
    }
    Ok(())
}
