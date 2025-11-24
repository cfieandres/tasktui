use super::{app::App, THEME};
use chrono::{NaiveDate, Utc, Duration};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

const TASK_NAME_WIDTH: usize = 20;
const BAR_FULL: &str = "█";
const BAR_EMPTY: &str = "░";

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

    render_header(frame, chunks[0], app);
    render_gantt(frame, chunks[1], app);
    render_footer(frame, chunks[2]);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let project_name = app.get_current_project()
        .map(|p| p.frontmatter.title.as_str())
        .unwrap_or("Unknown Project");

    let title = vec![
        Line::from(vec![
            Span::styled(format!("  {} - Gantt View", project_name), THEME.title_style()),
        ]),
    ];

    let header = Paragraph::new(title)
        .block(Block::default().borders(Borders::BOTTOM).border_style(THEME.border_style()));

    frame.render_widget(header, area);
}

fn render_gantt(frame: &mut Frame, area: Rect, app: &App) {
    let tasks = app.get_project_tasks();
    let timeline_width = (area.width as usize).saturating_sub(TASK_NAME_WIDTH + 4);

    // Calculate date range
    let today = Utc::now().date_naive();
    let (min_date, max_date) = calculate_date_range(&tasks, today, app.gantt_scroll_offset);
    let total_days = (max_date - min_date).num_days().max(1) as usize;
    let days_per_char = (total_days as f64 / timeline_width as f64).max(1.0);

    let mut items = Vec::new();

    // Month header
    items.push(ListItem::new(create_month_header(min_date, max_date, timeline_width)));

    // Today marker position
    let today_col = date_to_col(today, min_date, days_per_char, timeline_width);

    if tasks.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("  No tasks in this project yet.", THEME.dim_style()),
        ])));
    } else {
        for (idx, task) in tasks.iter().enumerate() {
            let is_selected = idx == app.gantt_selected;

            // Task name (truncated)
            let mut name = task.frontmatter.title.clone();
            if name.len() > TASK_NAME_WIDTH - 3 {
                name.truncate(TASK_NAME_WIDTH - 6);
                name.push_str("...");
            }

            // Get task dates
            let start = parse_date(task.frontmatter.start_date.as_deref())
                .or_else(|| parse_date(task.frontmatter.due_date.as_deref()))
                .unwrap_or(today);

            let end = parse_date(task.frontmatter.end_date.as_deref())
                .or_else(|| parse_date(task.frontmatter.due_date.as_deref()))
                .unwrap_or(start + Duration::days(7));

            // Calculate bar position
            let start_col = date_to_col(start, min_date, days_per_char, timeline_width);
            let end_col = date_to_col(end, min_date, days_per_char, timeline_width);

            // Progress
            let progress = match task.frontmatter.status {
                crate::models::Status::Done | crate::models::Status::Archived => 100,
                _ => task.frontmatter.progress.unwrap_or(0) as usize,
            };

            // Render bar
            let bar = render_bar(start_col, end_col, progress, timeline_width, Some(today_col));

            // Selection indicator
            let name_span = if is_selected {
                vec![
                    Span::styled(" ▸ ", THEME.accent_style()),
                    Span::styled(format!("{:<width$}", name, width = TASK_NAME_WIDTH - 3), THEME.highlight_style()),
                ]
            } else {
                vec![
                    Span::raw("   "),
                    Span::styled(format!("{:<width$}", name, width = TASK_NAME_WIDTH - 3), THEME.normal_style()),
                ]
            };

            let mut line_spans = name_span;
            line_spans.push(Span::raw("│"));
            line_spans.push(Span::styled(bar, THEME.accent_style()));

            items.push(ListItem::new(Line::from(line_spans)));
        }
    }

    // Today indicator line
    let mut today_line = vec![
        Span::raw(" ".repeat(TASK_NAME_WIDTH)),
        Span::raw("│"),
    ];
    if today_col < timeline_width {
        let before = " ".repeat(today_col);
        let marker = "|← Today";
        today_line.push(Span::styled(format!("{}{}", before, marker), THEME.dim_style()));
    }
    items.push(ListItem::new(Line::from(today_line)));

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
        Span::styled("←→", THEME.accent_style()),
        Span::raw(" scroll  "),
        Span::styled("Esc", THEME.accent_style()),
        Span::raw(" back  "),
        Span::styled("q", THEME.accent_style()),
        Span::raw(" quit"),
    ];

    let footer = Paragraph::new(Line::from(help_items))
        .block(Block::default().borders(Borders::TOP).border_style(THEME.border_style()));

    frame.render_widget(footer, area);
}

fn calculate_date_range(tasks: &[&crate::models::TaskItem], today: NaiveDate, scroll_offset: i32) -> (NaiveDate, NaiveDate) {
    let mut min_date = today - Duration::days(7);
    let mut max_date = today + Duration::days(30);

    for task in tasks {
        if let Some(start) = parse_date(task.frontmatter.start_date.as_deref()) {
            if start < min_date {
                min_date = start;
            }
        }
        if let Some(end) = parse_date(task.frontmatter.end_date.as_deref())
            .or_else(|| parse_date(task.frontmatter.due_date.as_deref()))
        {
            if end > max_date {
                max_date = end;
            }
        }
    }

    // Apply scroll offset
    min_date = min_date + Duration::days(scroll_offset as i64);
    max_date = max_date + Duration::days(scroll_offset as i64);

    (min_date, max_date)
}

fn parse_date(date_str: Option<&str>) -> Option<NaiveDate> {
    date_str.and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
}

fn date_to_col(date: NaiveDate, min_date: NaiveDate, days_per_char: f64, max_col: usize) -> usize {
    let days = (date - min_date).num_days().max(0) as f64;
    let col = (days / days_per_char) as usize;
    col.min(max_col.saturating_sub(1))
}

fn render_bar(start_col: usize, end_col: usize, progress: usize, total_width: usize, today_col: Option<usize>) -> String {
    let mut result = vec![' '; total_width];

    let bar_length = end_col.saturating_sub(start_col).max(1);
    let filled = ((bar_length as f64 * progress as f64) / 100.0).round() as usize;

    for i in 0..bar_length {
        let col = start_col + i;
        if col < total_width {
            result[col] = if i < filled { '█' } else { '░' };
        }
    }

    // Insert today marker if it's in range
    if let Some(today) = today_col {
        if today < total_width && result[today] == ' ' {
            result[today] = '│';
        }
    }

    result.iter().collect()
}

fn create_month_header(min_date: NaiveDate, max_date: NaiveDate, width: usize) -> Line<'static> {
    let total_days = (max_date - min_date).num_days().max(1) as usize;
    let days_per_char = (total_days as f64 / width as f64).max(1.0);

    let mut header = " ".repeat(TASK_NAME_WIDTH);
    header.push('│');

    let mut current = min_date;
    let mut last_month = None;
    let mut result = String::new();

    for col in 0..width {
        let days_from_start = (col as f64 * days_per_char) as i64;
        let date = min_date + Duration::days(days_from_start);
        let month = date.format("%b").to_string();

        if last_month.as_ref() != Some(&month) {
            // New month boundary
            if col > 0 {
                result.push(' ');
            }
            result.push_str(&month);
            last_month = Some(month);
        } else {
            result.push(' ');
        }

        if result.len() >= width {
            break;
        }
    }

    // Truncate or pad to exact width
    result.truncate(width);
    while result.len() < width {
        result.push(' ');
    }

    header.push_str(&result);

    Line::from(vec![
        Span::raw(" ".repeat(TASK_NAME_WIDTH)),
        Span::styled("│", THEME.border_style()),
        Span::styled(result, THEME.dim_style()),
    ])
}
