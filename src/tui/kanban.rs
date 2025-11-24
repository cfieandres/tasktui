use super::{app::App, THEME};
use crate::models::Status;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Main layout: header, board, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Board
            Constraint::Length(3),  // Footer
        ])
        .split(size);

    // Render header
    render_header(frame, chunks[0], app);

    // Render kanban board
    render_board(frame, chunks[1], app);

    // Render footer
    render_footer(frame, chunks[2], app);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let title = vec![
        Line::from(vec![
            Span::styled("         ▀█▀ ▄▀█ █▀ █▄▀ ▀█▀ █ █ █", THEME.title_style()),
        ]),
        Line::from(vec![
            Span::styled("          █  █▀█ ▄█ █ █  █  █▄█ █", THEME.title_style()),
        ]),
    ];

    let header = Paragraph::new(title)
        .block(Block::default().borders(Borders::BOTTOM).border_style(THEME.border_style()));

    frame.render_widget(header, area);
}

fn render_board(frame: &mut Frame, area: Rect, app: &App) {
    // Split into 4 columns
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    render_column(frame, columns[0], "ACTIVE", Status::Active, app);
    render_column(frame, columns[1], "NEXT", Status::Next, app);
    render_column(frame, columns[2], "WAITING", Status::Waiting, app);
    render_column(frame, columns[3], "DONE", Status::Done, app);
}

fn render_column(frame: &mut Frame, area: Rect, title: &str, status: Status, app: &App) {
    let tasks = app.tasks_by_status(status);

    let items: Vec<ListItem> = tasks
        .iter()
        .map(|task| {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled(&task.frontmatter.priority.emoji(), THEME.normal_style()),
                    Span::styled(format!(" {}", task.frontmatter.title), THEME.normal_style()),
                ]),
            ];

            // Add tags
            if !task.frontmatter.tags.is_empty() {
                let tags = task.frontmatter.tags
                    .iter()
                    .map(|t| format!("#{}", t))
                    .collect::<Vec<_>>()
                    .join(" ");
                lines.push(Line::from(Span::styled(tags, THEME.tag_style())));
            }

            // Add due date
            if let Some(due) = &task.frontmatter.due_date {
                lines.push(Line::from(Span::styled(
                    format!("📅 {}", due),
                    THEME.dim_style(),
                )));
            }

            lines.push(Line::from(""));

            ListItem::new(lines)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .title_style(THEME.accent_style())
            .borders(Borders::ALL)
            .border_style(THEME.border_style()),
    );

    frame.render_widget(list, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let mut help_items = vec![
        Span::styled("n", THEME.accent_style()),
        Span::raw(" new  "),
        Span::styled("d", THEME.accent_style()),
        Span::raw(" done  "),
        Span::styled("a", THEME.accent_style()),
        Span::raw(" archive  "),
        Span::styled("tab", THEME.accent_style()),
        Span::raw(" view  "),
        Span::styled("q", THEME.accent_style()),
        Span::raw(" quit"),
    ];

    if let Some(filter) = &app.active_filter {
        help_items.insert(0, Span::styled(format!(" Filter: {} ", filter), THEME.highlight_style()));
        help_items.insert(1, Span::raw("  "));
    }

    let footer = Paragraph::new(Line::from(help_items))
        .block(Block::default().borders(Borders::TOP).border_style(THEME.border_style()));

    frame.render_widget(footer, area);
}
