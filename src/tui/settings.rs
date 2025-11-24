use super::{app::App, THEME};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
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
    render_footer(frame, chunks[2], app);

    // Render edit dialog if active
    if app.settings_editing {
        render_edit_dialog(frame, app);
    }
}

fn render_header(frame: &mut Frame, area: Rect) {
    let title = vec![
        Line::from(vec![
            Span::styled("  Settings - Workstreams", THEME.title_style()),
        ]),
    ];

    let header = Paragraph::new(title)
        .block(Block::default().borders(Borders::BOTTOM).border_style(THEME.border_style()));

    frame.render_widget(header, area);
}

fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    let mut items = Vec::new();

    // Add instruction
    items.push(ListItem::new(Line::from(vec![
        Span::styled("  Workstreams (press key to filter tasks):", THEME.dim_style()),
    ])));
    items.push(ListItem::new(""));

    // Add workstream items
    for (idx, ws) in app.config.workstreams.iter().enumerate() {
        let is_selected = idx == app.settings_selected;

        let line = if is_selected {
            Line::from(vec![
                Span::styled(" ▸ ", THEME.accent_style()),
                Span::styled(format!("[{}] ", ws.key), THEME.accent_style()),
                Span::styled(&ws.name, THEME.highlight_style()),
            ])
        } else {
            Line::from(vec![
                Span::raw("   "),
                Span::styled(format!("[{}] ", ws.key), THEME.dim_style()),
                Span::styled(&ws.name, THEME.normal_style()),
            ])
        };

        items.push(ListItem::new(line));
    }

    // Add "Add new" option
    items.push(ListItem::new(""));
    let add_new_selected = app.settings_selected == app.config.workstreams.len();
    let add_line = if add_new_selected {
        Line::from(vec![
            Span::styled(" ▸ ", THEME.accent_style()),
            Span::styled("[+] Add new workstream", THEME.highlight_style()),
        ])
    } else {
        Line::from(vec![
            Span::raw("   "),
            Span::styled("[+] Add new workstream", THEME.dim_style()),
        ])
    };
    items.push(ListItem::new(add_line));

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(THEME.border_style()),
    );

    frame.render_widget(list, area);
}

fn render_footer(frame: &mut Frame, area: Rect, _app: &App) {
    let help_items = vec![
        Span::styled("↑↓", THEME.accent_style()),
        Span::raw(" nav  "),
        Span::styled("Enter", THEME.accent_style()),
        Span::raw(" edit/add  "),
        Span::styled("x", THEME.accent_style()),
        Span::raw(" delete  "),
        Span::styled("Esc", THEME.accent_style()),
        Span::raw(" back"),
    ];

    let footer = Paragraph::new(Line::from(help_items))
        .block(Block::default().borders(Borders::TOP).border_style(THEME.border_style()));

    frame.render_widget(footer, area);
}

fn render_edit_dialog(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Center the dialog
    let dialog_width = 40.min(area.width.saturating_sub(4));
    let dialog_height = 5;
    let dialog_area = Rect {
        x: (area.width.saturating_sub(dialog_width)) / 2,
        y: (area.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the area behind the dialog
    frame.render_widget(Clear, dialog_area);

    // Determine title based on whether adding or editing
    let title = if app.settings_selected < app.config.workstreams.len() {
        " Rename Workstream "
    } else {
        " New Workstream "
    };

    // Create dialog content
    let input_text = format!("{}_", app.settings_edit_text);
    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw(" "),
            Span::styled(&input_text, THEME.normal_style()),
        ]),
    ];

    let dialog = Paragraph::new(content)
        .block(
            Block::default()
                .title(title)
                .title_style(THEME.accent_style())
                .borders(Borders::ALL)
                .border_style(THEME.border_focused_style())
        );

    frame.render_widget(dialog, dialog_area);
}
