use super::{app::App, THEME};
use crate::models::Status;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
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
    let items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("F", THEME.accent_style()),
            Span::raw("ilters"),
        ])),
        ListItem::new(""),
        ListItem::new(if app.active_filter.is_none() {
            Line::from(Span::styled("● All", THEME.primary))
        } else {
            Line::from(Span::raw("○ All"))
        }),
        ListItem::new(if app.active_filter.as_deref() == Some("work") {
            Line::from(Span::styled("● Work", THEME.primary))
        } else {
            Line::from(Span::raw("○ Work"))
        }),
        ListItem::new(if app.active_filter.as_deref() == Some("personal") {
            Line::from(Span::styled("● Personal", THEME.primary))
        } else {
            Line::from(Span::raw("○ Personal"))
        }),
    ];

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

    // Active section
    items.push(ListItem::new(Line::from(vec![
        Span::styled("  Active Tasks", THEME.accent_style()),
        Span::styled("  ━━━━━━━━━━━━━", THEME.border_style()),
    ])));
    items.push(ListItem::new(""));

    for (idx, task) in active_tasks.iter().enumerate() {
        let is_selected = idx == app.selected_index && app.selected_index < active_tasks.len();
        items.push(create_task_item(task, is_selected));
    }

    // Next section
    if !next_tasks.is_empty() {
        items.push(ListItem::new(""));
        items.push(ListItem::new(Line::from(vec![
            Span::styled("  Next Tasks", THEME.dim_style()),
            Span::styled("  ━━━━━━━━━━━", THEME.border_style()),
        ])));
        items.push(ListItem::new(""));

        for task in next_tasks.iter() {
            items.push(create_task_item(task, false));
        }
    }

    // Done section
    if !done_tasks.is_empty() {
        items.push(ListItem::new(""));
        items.push(ListItem::new(Line::from(vec![
            Span::styled("  Done Today", THEME.dim_style()),
            Span::styled("  ━━━━━━━━━━━", THEME.border_style()),
        ])));
        items.push(ListItem::new(""));

        for task in done_tasks.iter().take(3) {
            items.push(create_task_item(task, false));
        }
    }

    let list = List::new(items);
    frame.render_widget(list, area);
}

fn create_task_item(task: &crate::models::TaskItem, is_selected: bool) -> ListItem {
    let mut lines = vec![];

    // Title line with priority
    let title_line = if is_selected {
        Line::from(vec![
            Span::styled(" ▸ ", THEME.primary),
            Span::styled(&task.frontmatter.priority.emoji(), THEME.normal_style()),
            Span::styled(format!(" {}", task.frontmatter.title), THEME.highlight_style()),
        ])
    } else {
        Line::from(vec![
            Span::raw("   "),
            Span::styled(&task.frontmatter.priority.emoji(), THEME.normal_style()),
            Span::styled(format!(" {}", task.frontmatter.title), THEME.normal_style()),
        ])
    };
    lines.push(title_line);

    // Separator and info line
    lines.push(Line::from(Span::styled(
        "   ┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈",
        THEME.border_style(),
    )));

    // Tags and due date
    let mut info_spans = vec![Span::raw("      ")];

    if !task.frontmatter.tags.is_empty() {
        let tags = task.frontmatter.tags
            .iter()
            .map(|t| format!("#{}", t))
            .collect::<Vec<_>>()
            .join(" ");
        info_spans.push(Span::styled(tags, THEME.tag_style()));
        info_spans.push(Span::raw("  "));
    }

    if let Some(due) = &task.frontmatter.due_date {
        info_spans.push(Span::styled(format!("📅 {}", due), THEME.dim_style()));
    }

    lines.push(Line::from(info_spans));
    lines.push(Line::from("")); // Spacing

    ListItem::new(lines)
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let help_items = vec![
        Span::styled("↑↓", THEME.accent_style()),
        Span::raw(" nav  "),
        Span::styled("n", THEME.accent_style()),
        Span::raw(" new  "),
        Span::styled("d", THEME.accent_style()),
        Span::raw(" done  "),
        Span::styled("1", THEME.accent_style()),
        Span::raw(" work  "),
        Span::styled("2", THEME.accent_style()),
        Span::raw(" personal  "),
        Span::styled("0", THEME.accent_style()),
        Span::raw(" all  "),
        Span::styled("tab", THEME.accent_style()),
        Span::raw(" view  "),
        Span::styled("q", THEME.accent_style()),
        Span::raw(" quit"),
    ];

    let footer = Paragraph::new(Line::from(help_items))
        .block(Block::default().borders(Borders::TOP).border_style(THEME.border_style()));

    frame.render_widget(footer, area);
}
