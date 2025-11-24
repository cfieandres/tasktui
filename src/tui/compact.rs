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

    // Main layout: header, content, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Footer
        ])
        .split(size);

    // Render header
    render_header(frame, chunks[0], app);

    // Render content with sidebar
    render_content(frame, chunks[1], app);

    // Render footer
    render_footer(frame, chunks[2], app);
}

fn render_header(frame: &mut Frame, area: Rect, _app: &App) {
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

fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    // Split into sidebar and main content
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(12), // Sidebar
            Constraint::Min(0),     // Main
        ])
        .split(area);

    render_sidebar(frame, chunks[0], app);
    render_task_list(frame, chunks[1], app);
}

fn render_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    let mut items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("F", THEME.accent_style()),
            Span::raw("ilters"),
        ])),
        ListItem::new(""),
        ListItem::new(if app.active_filter.is_none() {
            Line::from(Span::styled("● All", THEME.accent_style()))
        } else {
            Line::from(Span::raw("○ All"))
        }),
    ];

    // Add dynamic workstream filters
    for ws in &app.config.workstreams {
        let is_active = app.active_filter.as_deref() == Some(&ws.name);
        // Capitalize first letter for display
        let display_name = ws.name.chars().next()
            .map(|c| c.to_uppercase().to_string() + &ws.name[1..])
            .unwrap_or_else(|| ws.name.clone());

        if is_active {
            items.push(ListItem::new(Line::from(Span::styled(
                format!("● {}", display_name),
                THEME.accent_style(),
            ))));
        } else {
            items.push(ListItem::new(Line::from(Span::raw(format!("○ {}", display_name)))));
        }
    }

    let sidebar = List::new(items)
        .block(
            Block::default()
                .borders(Borders::RIGHT)
                .border_style(THEME.border_style())
        );

    frame.render_widget(sidebar, area);
}

fn render_task_list(frame: &mut Frame, area: Rect, app: &App) {
    let filtered = app.filtered_tasks();

    // Group tasks by status
    let active_tasks: Vec<_> = filtered.iter()
        .filter(|t| t.frontmatter.status == Status::Active)
        .collect();
    let next_tasks: Vec<_> = filtered.iter()
        .filter(|t| t.frontmatter.status == Status::Next)
        .collect();
    let done_tasks: Vec<_> = filtered.iter()
        .filter(|t| t.frontmatter.status == Status::Done)
        .collect();

    let mut items = Vec::new();
    let mut current_offset: usize = 0;

    // Active section
    items.push(ListItem::new(Line::from(vec![
        Span::styled("  Active Tasks", THEME.accent_style()),
        Span::styled(format!(" ({})", active_tasks.len()), THEME.dim_style()),
    ])));

    for (idx, task) in active_tasks.iter().enumerate() {
        let is_selected = current_offset + idx == app.selected_index;
        items.push(create_task_item(task, is_selected));
    }
    current_offset += active_tasks.len();

    // Next section
    if !next_tasks.is_empty() {
        items.push(ListItem::new(""));
        items.push(ListItem::new(Line::from(vec![
            Span::styled("  Next Tasks", THEME.dim_style()),
            Span::styled(format!(" ({})", next_tasks.len()), THEME.dim_style()),
        ])));

        for (idx, task) in next_tasks.iter().enumerate() {
            let is_selected = current_offset + idx == app.selected_index;
            items.push(create_task_item(task, is_selected));
        }
        current_offset += next_tasks.len();
    }

    // Done section (show up to 10)
    if !done_tasks.is_empty() {
        items.push(ListItem::new(""));
        let showing = done_tasks.len().min(10);
        let remaining = done_tasks.len().saturating_sub(10);
        let label = if remaining > 0 {
            format!("  Done ({} shown, +{} more)", showing, remaining)
        } else {
            format!("  Done ({})", done_tasks.len())
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(label, THEME.dim_style()),
        ])));

        for (idx, task) in done_tasks.iter().take(10).enumerate() {
            let is_selected = current_offset + idx == app.selected_index;
            items.push(create_task_item(task, is_selected));
        }
    }

    let list = List::new(items);
    frame.render_widget(list, area);
}

fn create_task_item(task: &crate::models::TaskItem, is_selected: bool) -> ListItem {
    // Single line with title, tags, and due date
    let mut spans = Vec::new();

    if is_selected {
        spans.push(Span::styled(" ▸ ", THEME.accent_style()));
        spans.push(Span::styled(task.frontmatter.priority.emoji(), THEME.normal_style()));
        spans.push(Span::styled(format!(" {}", task.frontmatter.title), THEME.highlight_style()));
    } else {
        spans.push(Span::raw("   "));
        spans.push(Span::styled(task.frontmatter.priority.emoji(), THEME.normal_style()));
        spans.push(Span::styled(format!(" {}", task.frontmatter.title), THEME.normal_style()));
    }

    // Add tags inline
    if !task.frontmatter.tags.is_empty() {
        let tags = task.frontmatter.tags
            .iter()
            .map(|t| format!("#{}", t))
            .collect::<Vec<_>>()
            .join(" ");
        spans.push(Span::raw("  "));
        spans.push(Span::styled(tags, THEME.tag_style()));
    }

    // Add due date inline
    if let Some(due) = &task.frontmatter.due_date {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(format!("📅 {}", due), THEME.dim_style()));
    }

    ListItem::new(Line::from(spans))
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let mut help_items = vec![
        Span::styled("↑↓", THEME.accent_style()),
        Span::raw(" nav  "),
        Span::styled("n", THEME.accent_style()),
        Span::raw(" new  "),
        Span::styled("d", THEME.accent_style()),
        Span::raw(" done  "),
    ];

    // Add dynamic workstream shortcuts
    for ws in &app.config.workstreams {
        help_items.push(Span::styled(ws.key.to_string(), THEME.accent_style()));
        help_items.push(Span::raw(format!(" {}  ", ws.name)));
    }

    help_items.extend([
        Span::styled("0", THEME.accent_style()),
        Span::raw(" all  "),
        Span::styled("p", THEME.accent_style()),
        Span::raw(" projects  "),
        Span::styled("s", THEME.accent_style()),
        Span::raw(" settings  "),
        Span::styled("tab", THEME.accent_style()),
        Span::raw(" view  "),
        Span::styled("q", THEME.accent_style()),
        Span::raw(" quit"),
    ]);

    let footer = Paragraph::new(Line::from(help_items))
        .block(Block::default().borders(Borders::TOP).border_style(THEME.border_style()));

    frame.render_widget(footer, area);
}
