use crate::models::{ItemType, Status, TaskFilter, TaskItem};
use crate::storage::Storage;
use anyhow::Result;
use ratatui::Frame;
use std::path::PathBuf;

use super::{kanban, compact, THEME};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Kanban,
    Compact,
}

pub struct App {
    pub storage: Storage,
    pub view_mode: ViewMode,
    pub tasks: Vec<TaskItem>,
    pub selected_index: usize,
    pub active_filter: Option<String>,
    pub show_new_task: bool,
}

impl App {
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        let storage = Storage::new(data_dir)?;
        let tasks = storage.load_all_tasks()?;

        Ok(Self {
            storage,
            view_mode: ViewMode::Compact,
            tasks,
            selected_index: 0,
            active_filter: None,
            show_new_task: false,
        })
    }

    pub fn toggle_view(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Kanban => ViewMode::Compact,
            ViewMode::Compact => ViewMode::Kanban,
        };
    }

    pub fn render(&mut self, frame: &mut Frame) {
        match self.view_mode {
            ViewMode::Kanban => kanban::render(frame, self),
            ViewMode::Compact => compact::render(frame, self),
        }
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
}
