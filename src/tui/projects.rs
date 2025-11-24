use super::{app::App, THEME};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Main layout: header, content, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Footer
        ])
        .split(size);

    render_header(frame, chunks[0]);
    render_content(frame, chunks[1], app);
    render_footer(frame, chunks[2]);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let title = vec![
        Line::from(vec![
            Span::styled("  PROJECTS", THEME.title_style()),
        ]),
    ];

    let header = Paragraph::new(title)
        .block(Block::default().borders(Borders::BOTTOM).border_style(THEME.border_style()));

    frame.render_widget(header, area);
}

fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    let projects = app.get_projects();
    let mut items = Vec::new();

    if projects.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("  No projects yet. Press 'n' to create one.", THEME.dim_style()),
        ])));
    } else {
        for (idx, project) in projects.iter().enumerate() {
            let is_selected = idx == app.projects_selected;
            let project_id = project.frontmatter.id;

            // Calculate progress
            let progress = app.calculate_project_progress(project_id);
            let (total, done, active) = app.project_task_counts(project_id);

            // Progress bar (10 chars)
            let filled = (progress as usize) / 10;
            let empty = 10 - filled;
            let progress_bar = format!(
                "[{}{}]",
                "█".repeat(filled),
                "░".repeat(empty)
            );

            // Due date
            let due = project.frontmatter.end_date.as_deref()
                .or(project.frontmatter.due_date.as_deref())
                .unwrap_or("No due date");

            // Selection indicator and title
            let title_line = if is_selected {
                Line::from(vec![
                    Span::styled(" ▸ ", THEME.accent_style()),
                    Span::styled(&project.frontmatter.title, THEME.highlight_style()),
                ])
            } else {
                Line::from(vec![
                    Span::raw("   "),
                    Span::styled(&project.frontmatter.title, THEME.normal_style()),
                ])
            };

            // Info line with progress bar
            let info_line = Line::from(vec![
                Span::raw("     "),
                Span::styled(progress_bar, if progress >= 100 { THEME.accent_style() } else { THEME.dim_style() }),
                Span::styled(format!(" {}%", progress), THEME.dim_style()),
                Span::raw("   "),
                Span::styled(format!("Due: {}", due), THEME.dim_style()),
            ]);

            // Stats line
            let stats_line = Line::from(vec![
                Span::raw("     "),
                Span::styled(format!("{} tasks", total), THEME.dim_style()),
                Span::raw("  •  "),
                Span::styled(format!("{} done", done), THEME.dim_style()),
                Span::raw("  •  "),
                Span::styled(format!("{} active", active), THEME.dim_style()),
            ]);

            items.push(ListItem::new(vec![title_line, info_line, stats_line, Line::from("")]));
        }
    }

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(THEME.border_style()),
    );

    frame.render_widget(list, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let help_items = vec![
        Span::styled("↑↓", THEME.accent_style()),
        Span::raw(" nav  "),
        Span::styled("Enter", THEME.accent_style()),
        Span::raw(" gantt  "),
        Span::styled("n", THEME.accent_style()),
        Span::raw(" new project  "),
        Span::styled("Esc", THEME.accent_style()),
        Span::raw(" back  "),
        Span::styled("q", THEME.accent_style()),
        Span::raw(" quit"),
    ];

    let footer = Paragraph::new(Line::from(help_items))
        .block(Block::default().borders(Borders::TOP).border_style(THEME.border_style()));

    frame.render_widget(footer, area);
}
