use super::{app::{App, SettingsSection}, THEME};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Main layout: header, tabs, content, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(3),  // Tabs
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Footer
        ])
        .split(size);

    render_header(frame, chunks[0]);
    render_tabs(frame, chunks[1], app);
    render_content(frame, chunks[2], app);
    render_footer(frame, chunks[3], app);

    // Render edit dialog if active
    if app.settings_editing {
        render_edit_dialog(frame, app);
    }
}

fn render_header(frame: &mut Frame, area: Rect) {
    let title = vec![
        Line::from(vec![
            Span::styled("  Settings", THEME.title_style()),
        ]),
    ];

    let header = Paragraph::new(title)
        .block(Block::default().borders(Borders::BOTTOM).border_style(THEME.border_style()));

    frame.render_widget(header, area);
}

fn render_tabs(frame: &mut Frame, area: Rect, app: &App) {
    let titles = vec!["Workstreams", "Goals & Priorities", "API Keys"];
    let selected = match app.settings_section {
        SettingsSection::Workstreams => 0,
        SettingsSection::Goals => 1,
        SettingsSection::ApiKeys => 2,
    };

    let tabs = Tabs::new(titles)
        .select(selected)
        .highlight_style(THEME.highlight_style())
        .block(Block::default().borders(Borders::BOTTOM).border_style(THEME.border_style()));

    frame.render_widget(tabs, area);
}

fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    match app.settings_section {
        SettingsSection::Workstreams => render_workstreams(frame, area, app),
        SettingsSection::Goals => render_goals(frame, area, app),
        SettingsSection::ApiKeys => render_api_keys(frame, area, app),
    }
}

fn render_workstreams(frame: &mut Frame, area: Rect, app: &App) {
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

fn render_goals(frame: &mut Frame, area: Rect, app: &App) {
    let mut items = Vec::new();

    // Add instruction
    items.push(ListItem::new(Line::from(vec![
        Span::styled("  Your high-level goals & priorities (GTD Horizons of Focus):", THEME.dim_style()),
    ])));
    items.push(ListItem::new(""));

    if app.config.goals.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("  No goals defined yet. Add your priorities!", THEME.dim_style()),
        ])));
        items.push(ListItem::new(""));
    }

    // Add goal items
    for (idx, goal) in app.config.goals.iter().enumerate() {
        let is_selected = idx == app.settings_selected;

        // Priority stars (more stars = higher priority)
        let priority_stars = "★".repeat(6 - goal.priority as usize);
        let priority_empty = "☆".repeat(goal.priority.saturating_sub(1) as usize);

        // Active indicator
        let active_indicator = if goal.active { "●" } else { "○" };

        let line = if is_selected {
            Line::from(vec![
                Span::styled(" ▸ ", THEME.accent_style()),
                Span::styled(active_indicator, if goal.active { THEME.accent_style() } else { THEME.dim_style() }),
                Span::raw(" "),
                Span::styled(priority_stars, THEME.accent_style()),
                Span::styled(priority_empty, THEME.dim_style()),
                Span::raw(" "),
                Span::styled(format!("[{}] ", goal.area), THEME.tag_style()),
                Span::styled(goal.description.clone(), THEME.highlight_style()),
            ])
        } else {
            Line::from(vec![
                Span::raw("   "),
                Span::styled(active_indicator, if goal.active { THEME.normal_style() } else { THEME.dim_style() }),
                Span::raw(" "),
                Span::styled(priority_stars, THEME.normal_style()),
                Span::styled(priority_empty, THEME.dim_style()),
                Span::raw(" "),
                Span::styled(format!("[{}] ", goal.area), THEME.tag_style()),
                Span::styled(goal.description.clone(), if goal.active { THEME.normal_style() } else { THEME.dim_style() }),
            ])
        };

        items.push(ListItem::new(line));
    }

    // Add "Add new" option
    items.push(ListItem::new(""));
    let add_new_selected = app.settings_selected == app.config.goals.len();
    let add_line = if add_new_selected {
        Line::from(vec![
            Span::styled(" ▸ ", THEME.accent_style()),
            Span::styled("[+] Add new goal/priority", THEME.highlight_style()),
        ])
    } else {
        Line::from(vec![
            Span::raw("   "),
            Span::styled("[+] Add new goal/priority", THEME.dim_style()),
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

fn render_api_keys(frame: &mut Frame, area: Rect, app: &App) {
    let mut items = Vec::new();

    // Add instruction
    items.push(ListItem::new(Line::from(vec![
        Span::styled("  Configure API keys for LLM features:", THEME.dim_style()),
    ])));
    items.push(ListItem::new(""));

    // OpenAI API Key
    let is_selected = app.settings_selected == 0;
    let has_key = app.config.openai_api_key.is_some();

    let key_display = if let Some(key) = &app.config.openai_api_key {
        if key.len() > 8 {
            format!("{}...{}", &key[..4], &key[key.len()-4..])
        } else {
            "****".to_string()
        }
    } else {
        "(not set)".to_string()
    };

    let status_indicator = if has_key { "✓" } else { "○" };
    let status_style = if has_key { THEME.accent_style() } else { THEME.dim_style() };

    let line = if is_selected {
        Line::from(vec![
            Span::styled(" ▸ ", THEME.accent_style()),
            Span::styled(status_indicator, status_style),
            Span::raw(" "),
            Span::styled("OpenAI API Key: ", THEME.highlight_style()),
            Span::styled(key_display, THEME.dim_style()),
        ])
    } else {
        Line::from(vec![
            Span::raw("   "),
            Span::styled(status_indicator, status_style),
            Span::raw(" "),
            Span::styled("OpenAI API Key: ", THEME.normal_style()),
            Span::styled(key_display, THEME.dim_style()),
        ])
    };

    items.push(ListItem::new(line));

    // Add help text
    items.push(ListItem::new(""));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("  The API key enables natural language task parsing.", THEME.dim_style()),
    ])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("  Get your key at: ", THEME.dim_style()),
        Span::styled("https://platform.openai.com/api-keys", THEME.accent_style()),
    ])));

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(THEME.border_style()),
    );

    frame.render_widget(list, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let help_items = match app.settings_section {
        SettingsSection::Workstreams => vec![
            Span::styled("Tab", THEME.accent_style()),
            Span::raw(" section  "),
            Span::styled("↑↓", THEME.accent_style()),
            Span::raw(" nav  "),
            Span::styled("Enter", THEME.accent_style()),
            Span::raw(" edit  "),
            Span::styled("x", THEME.accent_style()),
            Span::raw(" delete  "),
            Span::styled("Esc", THEME.accent_style()),
            Span::raw(" back"),
        ],
        SettingsSection::Goals => vec![
            Span::styled("Tab", THEME.accent_style()),
            Span::raw(" section  "),
            Span::styled("↑↓", THEME.accent_style()),
            Span::raw(" nav  "),
            Span::styled("Enter", THEME.accent_style()),
            Span::raw(" edit  "),
            Span::styled("P", THEME.accent_style()),
            Span::raw(" priority  "),
            Span::styled("Space", THEME.accent_style()),
            Span::raw(" toggle  "),
            Span::styled("x", THEME.accent_style()),
            Span::raw(" delete  "),
            Span::styled("Esc", THEME.accent_style()),
            Span::raw(" back"),
        ],
        SettingsSection::ApiKeys => vec![
            Span::styled("Tab", THEME.accent_style()),
            Span::raw(" section  "),
            Span::styled("Enter", THEME.accent_style()),
            Span::raw(" edit  "),
            Span::styled("x", THEME.accent_style()),
            Span::raw(" clear  "),
            Span::styled("Esc", THEME.accent_style()),
            Span::raw(" back"),
        ],
    };

    let footer = Paragraph::new(Line::from(help_items))
        .block(Block::default().borders(Borders::TOP).border_style(THEME.border_style()));

    frame.render_widget(footer, area);
}

fn render_edit_dialog(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Center the dialog
    let dialog_width = 60.min(area.width.saturating_sub(4));
    let dialog_height = if app.settings_section == SettingsSection::Goals { 7 } else { 5 };
    let dialog_area = Rect {
        x: (area.width.saturating_sub(dialog_width)) / 2,
        y: (area.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the area behind the dialog
    frame.render_widget(Clear, dialog_area);

    // Determine title and content based on section
    match app.settings_section {
        SettingsSection::Workstreams => {
            let title = if app.settings_selected < app.config.workstreams.len() {
                " Rename Workstream "
            } else {
                " New Workstream "
            };

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
        SettingsSection::Goals => {
            let title = if app.settings_selected < app.config.goals.len() {
                " Edit Goal "
            } else {
                " New Goal "
            };

            let input_text = format!("{}_", app.settings_edit_text);
            let content = vec![
                Line::from(""),
                Line::from(vec![
                    Span::raw(" Area: "),
                    Span::styled(format!("[{}]", app.settings_edit_area), THEME.tag_style()),
                    Span::styled(" (press Tab to change)", THEME.dim_style()),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw(" Goal: "),
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
        SettingsSection::ApiKeys => {
            let title = " Edit OpenAI API Key ";

            let input_text = format!("{}_", app.settings_edit_text);
            let content = vec![
                Line::from(""),
                Line::from(vec![
                    Span::raw(" "),
                    Span::styled(&input_text, THEME.normal_style()),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" Leave empty to clear the key", THEME.dim_style()),
                ]),
            ];

            // Use taller dialog for API key
            let api_dialog_area = Rect {
                x: dialog_area.x,
                y: dialog_area.y,
                width: dialog_area.width,
                height: 7,
            };

            frame.render_widget(Clear, api_dialog_area);

            let dialog = Paragraph::new(content)
                .block(
                    Block::default()
                        .title(title)
                        .title_style(THEME.accent_style())
                        .borders(Borders::ALL)
                        .border_style(THEME.border_focused_style())
                );

            frame.render_widget(dialog, api_dialog_area);
        }
    }
}
