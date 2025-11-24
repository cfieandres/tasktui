use crate::config::AppConfig;
use crate::llm::{EnrichedTask, TaskEnricher};
use crate::models::{ItemType, Priority, Status, TaskItem};
use crate::storage::Storage;
use anyhow::Result;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::path::PathBuf;

use uuid::Uuid;
use super::{kanban, compact, settings, projects, project_gantt, THEME};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Kanban,
    Compact,
    Settings,
    Projects,
    ProjectGantt,
}

/// Column indices for Kanban view
pub const KANBAN_COL_ACTIVE: usize = 0;
pub const KANBAN_COL_NEXT: usize = 1;
pub const KANBAN_COL_WAITING: usize = 2;
pub const KANBAN_COL_DONE: usize = 3;

pub struct App {
    pub storage: Storage,
    pub config: AppConfig,
    pub data_dir: PathBuf,
    pub view_mode: ViewMode,
    pub tasks: Vec<TaskItem>,
    pub selected_index: usize,
    pub active_filter: Option<String>,
    pub show_new_task: bool,
    pub new_task_title: String,
    // Kanban navigation state
    pub kanban_column: usize,
    pub kanban_row: usize,
    // Settings view state
    pub settings_selected: usize,
    pub settings_editing: bool,
    pub settings_edit_text: String,
    // Projects view state
    pub projects_selected: usize,
    pub current_project_id: Option<Uuid>,
    pub gantt_selected: usize,
    pub gantt_scroll_offset: i32,
    pub show_new_project: bool,
    pub new_project_title: String,
    // LLM enricher for natural language task parsing
    enricher: TaskEnricher,
}

impl App {
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        let storage = Storage::new(data_dir.clone())?;
        let config = AppConfig::load(&data_dir)?;
        let tasks = storage.load_all_tasks()?;

        // Initialize LLM enricher with API key from config (if present)
        let enricher = TaskEnricher::new(config.openai_api_key.clone());

        Ok(Self {
            storage,
            config,
            data_dir,
            view_mode: ViewMode::Compact,
            tasks,
            selected_index: 0,
            active_filter: None,
            show_new_task: false,
            new_task_title: String::new(),
            kanban_column: KANBAN_COL_ACTIVE,
            kanban_row: 0,
            settings_selected: 0,
            settings_editing: false,
            settings_edit_text: String::new(),
            projects_selected: 0,
            current_project_id: None,
            gantt_selected: 0,
            gantt_scroll_offset: 0,
            show_new_project: false,
            new_project_title: String::new(),
            enricher,
        })
    }

    pub fn toggle_view(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Kanban => ViewMode::Compact,
            ViewMode::Compact => ViewMode::Kanban,
            ViewMode::Settings => ViewMode::Compact,
            ViewMode::Projects => ViewMode::Compact,
            ViewMode::ProjectGantt => ViewMode::Projects,
        };
    }

    pub fn open_settings(&mut self) {
        self.view_mode = ViewMode::Settings;
        self.settings_selected = 0;
        self.settings_editing = false;
        self.settings_edit_text.clear();
    }

    pub fn close_settings(&mut self) {
        self.view_mode = ViewMode::Compact;
    }

    pub fn render(&mut self, frame: &mut Frame) {
        match self.view_mode {
            ViewMode::Kanban => kanban::render(frame, self),
            ViewMode::Compact => compact::render(frame, self),
            ViewMode::Settings => settings::render(frame, self),
            ViewMode::Projects => projects::render(frame, self),
            ViewMode::ProjectGantt => project_gantt::render(frame, self),
        }

        // Render new task dialog if open
        if self.show_new_task {
            self.render_new_task_dialog(frame);
        }

        // Render new project dialog if open
        if self.show_new_project {
            self.render_new_project_dialog(frame);
        }
    }

    fn render_new_task_dialog(&self, frame: &mut Frame) {
        let area = frame.area();

        // Center the dialog
        let dialog_width = 50.min(area.width.saturating_sub(4));
        let dialog_height = 5;
        let dialog_area = Rect {
            x: (area.width.saturating_sub(dialog_width)) / 2,
            y: (area.height.saturating_sub(dialog_height)) / 2,
            width: dialog_width,
            height: dialog_height,
        };

        // Clear the area behind the dialog
        frame.render_widget(Clear, dialog_area);

        // Create dialog content
        let input_text = format!("{}_", self.new_task_title);
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
                    .title(" New Task ")
                    .title_style(THEME.accent_style())
                    .borders(Borders::ALL)
                    .border_style(THEME.border_focused_style())
            );

        frame.render_widget(dialog, dialog_area);
    }

    fn render_new_project_dialog(&self, frame: &mut Frame) {
        let area = frame.area();

        // Center the dialog
        let dialog_width = 50.min(area.width.saturating_sub(4));
        let dialog_height = 5;
        let dialog_area = Rect {
            x: (area.width.saturating_sub(dialog_width)) / 2,
            y: (area.height.saturating_sub(dialog_height)) / 2,
            width: dialog_width,
            height: dialog_height,
        };

        // Clear the area behind the dialog
        frame.render_widget(Clear, dialog_area);

        // Create dialog content
        let input_text = format!("{}_", self.new_project_title);
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
                    .title(" New Project ")
                    .title_style(THEME.accent_style())
                    .borders(Borders::ALL)
                    .border_style(THEME.border_focused_style())
            );

        frame.render_widget(dialog, dialog_area);
    }

    pub fn next_task(&mut self) {
        if !self.filtered_tasks().is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_tasks().len();
        }
    }

    pub fn previous_task(&mut self) {
        let filtered = self.filtered_tasks();
        if !filtered.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = filtered.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn toggle_task_selection(&mut self) {
        // Future: expand/collapse task details
    }

    pub fn show_new_task_dialog(&mut self) {
        self.show_new_task = true;
        self.new_task_title.clear();
    }

    pub fn cancel_new_task_dialog(&mut self) {
        self.show_new_task = false;
        self.new_task_title.clear();
    }

    pub fn create_new_task(&mut self) -> Result<()> {
        if self.new_task_title.trim().is_empty() {
            self.show_new_task = false;
            return Ok(());
        }

        // Use LLM to enrich the raw input (will fallback to simple task if no API key)
        let enriched = self.enricher.enrich_sync(self.new_task_title.trim());

        // Create task with enriched data
        let mut task = TaskItem::new(enriched.title, ItemType::Task);

        // Apply enriched fields
        if let Some(due_date) = enriched.due_date {
            task.frontmatter.due_date = Some(due_date);
        }
        if let Some(priority) = enriched.priority {
            task.frontmatter.priority = match priority.to_lowercase().as_str() {
                "high" => Priority::High,
                "low" => Priority::Low,
                _ => Priority::Medium,
            };
        }
        if !enriched.tags.is_empty() {
            task.frontmatter.tags = enriched.tags;
        }
        if let Some(context) = enriched.context {
            task.body = context;
        }

        self.storage.write_task(&mut task)?;
        self.tasks.push(task);

        // Navigate to the new task (it's the last Active task since new tasks start as Active)
        let active_count = self.tasks.iter()
            .filter(|t| t.frontmatter.status == Status::Active)
            .count();
        self.selected_index = active_count.saturating_sub(1);

        // Also update Kanban view to show the new task
        self.kanban_column = KANBAN_COL_ACTIVE;
        let kanban_active_count = self.kanban_column_tasks().len();
        self.kanban_row = kanban_active_count.saturating_sub(1);

        self.show_new_task = false;
        self.new_task_title.clear();
        Ok(())
    }

    pub fn mark_task_done(&mut self) -> Result<()> {
        let filtered = self.filtered_tasks();
        if let Some(task) = filtered.get(self.selected_index) {
            let task_id = task.frontmatter.id;
            if let Some(task) = self.tasks.iter_mut().find(|t| t.frontmatter.id == task_id) {
                task.frontmatter.status = Status::Done;
                self.storage.write_task(task)?;
            }
        }
        Ok(())
    }

    pub fn archive_task(&mut self) -> Result<()> {
        let filtered = self.filtered_tasks();
        if let Some(task) = filtered.get(self.selected_index) {
            let task_id = task.frontmatter.id;
            if let Some(task) = self.tasks.iter_mut().find(|t| t.frontmatter.id == task_id) {
                task.frontmatter.status = Status::Archived;
                self.storage.write_task(task)?;
            }
        }
        Ok(())
    }

    pub fn refresh_tasks(&mut self) -> Result<()> {
        self.tasks = self.storage.load_all_tasks()?;
        Ok(())
    }

    pub fn filter_by_tag(&mut self, tag: &str) {
        self.active_filter = Some(tag.to_string());
        self.selected_index = 0;
    }

    pub fn clear_filters(&mut self) {
        self.active_filter = None;
        self.selected_index = 0;
    }

    pub fn filtered_tasks(&self) -> Vec<&TaskItem> {
        let mut tasks: Vec<&TaskItem> = self.tasks.iter().collect();

        if let Some(tag) = &self.active_filter {
            tasks.retain(|task| task.has_tag(tag));
        }

        tasks
    }

    pub fn tasks_by_status(&self, status: Status) -> Vec<&TaskItem> {
        let filtered = self.filtered_tasks();
        filtered.into_iter()
            .filter(|t| t.frontmatter.status == status)
            .collect()
    }

    /// Returns tasks in display order: Active → Next → Done (excludes Archived and Waiting for compact view)
    pub fn display_ordered_tasks(&self) -> Vec<&TaskItem> {
        let filtered = self.filtered_tasks();
        let mut result = Vec::new();

        // Active tasks first
        result.extend(filtered.iter().filter(|t| t.frontmatter.status == Status::Active).copied());
        // Next tasks
        result.extend(filtered.iter().filter(|t| t.frontmatter.status == Status::Next).copied());
        // Done tasks
        result.extend(filtered.iter().filter(|t| t.frontmatter.status == Status::Done).copied());

        result
    }

    /// Get count of tasks by status for navigation bounds
    pub fn task_counts(&self) -> (usize, usize, usize) {
        let filtered = self.filtered_tasks();
        let active = filtered.iter().filter(|t| t.frontmatter.status == Status::Active).count();
        let next = filtered.iter().filter(|t| t.frontmatter.status == Status::Next).count();
        let done = filtered.iter().filter(|t| t.frontmatter.status == Status::Done).count();
        (active, next, done)
    }

    // === Kanban Navigation Methods ===

    pub fn kanban_column_status(&self) -> Status {
        match self.kanban_column {
            KANBAN_COL_ACTIVE => Status::Active,
            KANBAN_COL_NEXT => Status::Next,
            KANBAN_COL_WAITING => Status::Waiting,
            KANBAN_COL_DONE => Status::Done,
            _ => Status::Active,
        }
    }

    pub fn kanban_column_tasks(&self) -> Vec<&TaskItem> {
        self.tasks_by_status(self.kanban_column_status())
    }

    pub fn kanban_move_left(&mut self) {
        if self.kanban_column == 0 {
            self.kanban_column = 3;
        } else {
            self.kanban_column -= 1;
        }
        // Clamp row to new column's task count
        let task_count = self.kanban_column_tasks().len();
        if self.kanban_row >= task_count {
            self.kanban_row = task_count.saturating_sub(1);
        }
    }

    pub fn kanban_move_right(&mut self) {
        self.kanban_column = (self.kanban_column + 1) % 4;
        // Clamp row to new column's task count
        let task_count = self.kanban_column_tasks().len();
        if self.kanban_row >= task_count {
            self.kanban_row = task_count.saturating_sub(1);
        }
    }

    pub fn kanban_move_up(&mut self) {
        let task_count = self.kanban_column_tasks().len();
        if task_count > 0 {
            if self.kanban_row == 0 {
                self.kanban_row = task_count - 1;
            } else {
                self.kanban_row -= 1;
            }
        }
    }

    pub fn kanban_move_down(&mut self) {
        let task_count = self.kanban_column_tasks().len();
        if task_count > 0 {
            self.kanban_row = (self.kanban_row + 1) % task_count;
        }
    }

    pub fn kanban_selected_task(&self) -> Option<&TaskItem> {
        self.kanban_column_tasks().get(self.kanban_row).copied()
    }

    pub fn kanban_mark_done(&mut self) -> Result<()> {
        if let Some(task) = self.kanban_selected_task() {
            let task_id = task.frontmatter.id;
            if let Some(task) = self.tasks.iter_mut().find(|t| t.frontmatter.id == task_id) {
                task.frontmatter.status = Status::Done;
                self.storage.write_task(task)?;
            }
            // Adjust row if we removed a task from current column
            let new_count = self.kanban_column_tasks().len();
            if self.kanban_row >= new_count && new_count > 0 {
                self.kanban_row = new_count - 1;
            }
        }
        Ok(())
    }

    pub fn kanban_archive_task(&mut self) -> Result<()> {
        if let Some(task) = self.kanban_selected_task() {
            let task_id = task.frontmatter.id;
            if let Some(task) = self.tasks.iter_mut().find(|t| t.frontmatter.id == task_id) {
                task.frontmatter.status = Status::Archived;
                self.storage.write_task(task)?;
            }
            // Adjust row if we removed a task from current column
            let new_count = self.kanban_column_tasks().len();
            if self.kanban_row >= new_count && new_count > 0 {
                self.kanban_row = new_count - 1;
            }
        }
        Ok(())
    }

    // === Settings View Methods ===

    pub fn settings_next(&mut self) {
        // +1 for the "Add new" option
        let max_items = self.config.workstreams.len() + 1;
        if max_items > 0 {
            self.settings_selected = (self.settings_selected + 1) % max_items;
        }
    }

    pub fn settings_prev(&mut self) {
        let max_items = self.config.workstreams.len() + 1;
        if max_items > 0 {
            if self.settings_selected == 0 {
                self.settings_selected = max_items - 1;
            } else {
                self.settings_selected -= 1;
            }
        }
    }

    pub fn settings_start_edit(&mut self) {
        if self.settings_selected < self.config.workstreams.len() {
            // Editing existing workstream
            self.settings_editing = true;
            self.settings_edit_text = self.config.workstreams[self.settings_selected].name.clone();
        } else {
            // Adding new workstream
            self.settings_editing = true;
            self.settings_edit_text.clear();
        }
    }

    pub fn settings_cancel_edit(&mut self) {
        self.settings_editing = false;
        self.settings_edit_text.clear();
    }

    pub fn settings_confirm_edit(&mut self) -> Result<()> {
        let new_name = self.settings_edit_text.trim().to_string();
        if new_name.is_empty() {
            self.settings_cancel_edit();
            return Ok(());
        }

        if self.settings_selected < self.config.workstreams.len() {
            // Rename existing
            self.config.workstreams[self.settings_selected].name = new_name;
        } else {
            // Add new
            self.config.add_workstream(new_name);
        }

        self.config.save(&self.data_dir)?;
        self.settings_editing = false;
        self.settings_edit_text.clear();
        Ok(())
    }

    pub fn settings_delete(&mut self) -> Result<()> {
        if self.settings_selected < self.config.workstreams.len() {
            self.config.workstreams.remove(self.settings_selected);
            self.config.save(&self.data_dir)?;
            // Adjust selection if needed
            if self.settings_selected >= self.config.workstreams.len() && self.settings_selected > 0 {
                self.settings_selected -= 1;
            }
        }
        Ok(())
    }

    pub fn save_config(&self) -> Result<()> {
        self.config.save(&self.data_dir)
    }

    // === Projects View Methods ===

    pub fn open_projects(&mut self) {
        self.view_mode = ViewMode::Projects;
        self.projects_selected = 0;
    }

    pub fn close_projects(&mut self) {
        self.view_mode = ViewMode::Compact;
    }

    pub fn get_projects(&self) -> Vec<&TaskItem> {
        self.tasks.iter()
            .filter(|t| t.is_project())
            .collect()
    }

    pub fn projects_next(&mut self) {
        let count = self.get_projects().len();
        if count > 0 {
            self.projects_selected = (self.projects_selected + 1) % count;
        }
    }

    pub fn projects_prev(&mut self) {
        let count = self.get_projects().len();
        if count > 0 {
            if self.projects_selected == 0 {
                self.projects_selected = count - 1;
            } else {
                self.projects_selected -= 1;
            }
        }
    }

    pub fn show_new_project_dialog(&mut self) {
        self.show_new_project = true;
        self.new_project_title.clear();
    }

    pub fn cancel_new_project_dialog(&mut self) {
        self.show_new_project = false;
        self.new_project_title.clear();
    }

    pub fn create_new_project(&mut self) -> Result<()> {
        if self.new_project_title.trim().is_empty() {
            self.show_new_project = false;
            return Ok(());
        }

        let mut project = TaskItem::new_project(self.new_project_title.trim().to_string());
        self.storage.write_task(&mut project)?;
        self.tasks.push(project);
        self.show_new_project = false;
        self.new_project_title.clear();

        // Select the new project
        self.projects_selected = self.get_projects().len().saturating_sub(1);
        Ok(())
    }

    pub fn open_project_gantt(&mut self) {
        let projects = self.get_projects();
        if let Some(project) = projects.get(self.projects_selected) {
            self.current_project_id = Some(project.frontmatter.id);
            self.view_mode = ViewMode::ProjectGantt;
            self.gantt_selected = 0;
            self.gantt_scroll_offset = 0;
        }
    }

    pub fn close_project_gantt(&mut self) {
        self.view_mode = ViewMode::Projects;
        self.current_project_id = None;
    }

    pub fn get_current_project(&self) -> Option<&TaskItem> {
        let project_id = self.current_project_id?;
        self.tasks.iter().find(|t| t.frontmatter.id == project_id)
    }

    pub fn get_project_tasks(&self) -> Vec<&TaskItem> {
        let Some(project_id) = self.current_project_id else {
            return Vec::new();
        };
        self.tasks.iter()
            .filter(|t| t.frontmatter.parent_goal_id == Some(project_id))
            .collect()
    }

    pub fn gantt_next(&mut self) {
        let count = self.get_project_tasks().len();
        if count > 0 {
            self.gantt_selected = (self.gantt_selected + 1) % count;
        }
    }

    pub fn gantt_prev(&mut self) {
        let count = self.get_project_tasks().len();
        if count > 0 {
            if self.gantt_selected == 0 {
                self.gantt_selected = count - 1;
            } else {
                self.gantt_selected -= 1;
            }
        }
    }

    pub fn gantt_scroll_left(&mut self) {
        self.gantt_scroll_offset = self.gantt_scroll_offset.saturating_sub(7); // Scroll by ~1 week
    }

    pub fn gantt_scroll_right(&mut self) {
        self.gantt_scroll_offset += 7;
    }

    /// Calculate project progress based on completed tasks
    pub fn calculate_project_progress(&self, project_id: Uuid) -> u8 {
        let tasks: Vec<_> = self.tasks.iter()
            .filter(|t| t.frontmatter.parent_goal_id == Some(project_id))
            .collect();

        if tasks.is_empty() {
            return 0;
        }

        let done = tasks.iter()
            .filter(|t| matches!(t.frontmatter.status, Status::Done | Status::Archived))
            .count();

        ((done as f64 / tasks.len() as f64) * 100.0) as u8
    }

    /// Count tasks by status for a project
    pub fn project_task_counts(&self, project_id: Uuid) -> (usize, usize, usize) {
        let tasks: Vec<_> = self.tasks.iter()
            .filter(|t| t.frontmatter.parent_goal_id == Some(project_id))
            .collect();

        let total = tasks.len();
        let done = tasks.iter()
            .filter(|t| matches!(t.frontmatter.status, Status::Done | Status::Archived))
            .count();
        let active = tasks.iter()
            .filter(|t| matches!(t.frontmatter.status, Status::Active | Status::Next))
            .count();

        (total, done, active)
    }
}
