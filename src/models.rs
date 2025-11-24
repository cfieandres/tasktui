use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Task status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Active,
    Next,
    Waiting,
    Done,
    Archived,
}

impl Status {
    pub fn as_str(&self) -> &str {
        match self {
            Status::Active => "active",
            Status::Next => "next",
            Status::Waiting => "waiting",
            Status::Done => "done",
            Status::Archived => "archived",
        }
    }
}

/// Item type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    Task,
    Goal,
    Note,
}

/// Priority level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    pub fn emoji(&self) -> &str {
        match self {
            Priority::High => "🔴",
            Priority::Medium => "🟠",
            Priority::Low => "⚪",
        }
    }
}

/// YAML Frontmatter structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    pub title: String,
    pub status: Status,
    #[serde(default = "default_priority")]
    pub priority: Priority,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_goal_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

fn default_priority() -> Priority {
    Priority::Medium
}

/// Complete task item (frontmatter + body)
#[derive(Debug, Clone)]
pub struct TaskItem {
    pub frontmatter: Frontmatter,
    pub body: String,
    pub file_path: std::path::PathBuf,
}

impl TaskItem {
    /// Create a new task item
    pub fn new(title: String, item_type: ItemType) -> Self {
        let id = Uuid::new_v4();
        Self {
            frontmatter: Frontmatter {
                id,
                item_type,
                title,
                status: Status::Active,
                priority: Priority::Medium,
                tags: Vec::new(),
                due_date: None,
                parent_goal_id: None,
                created_at: Utc::now(),
            },
            body: String::new(),
            file_path: std::path::PathBuf::new(),
        }
    }

    /// Check if task matches a tag filter
    pub fn has_tag(&self, tag: &str) -> bool {
        self.frontmatter.tags.iter().any(|t| t == tag)
    }

    /// Check if task is due today
    pub fn is_due_today(&self) -> bool {
        if let Some(due_date) = &self.frontmatter.due_date {
            let today = Utc::now().format("%Y-%m-%d").to_string();
            due_date.starts_with(&today)
        } else {
            false
        }
    }

    /// Get display title with priority emoji
    pub fn display_title(&self) -> String {
        format!("{} {}", self.frontmatter.priority.emoji(), self.frontmatter.title)
    }
}

/// Filter criteria for listing tasks
#[derive(Debug, Clone, Default)]
pub struct TaskFilter {
    pub status: Option<Status>,
    pub tags: Vec<String>,
    pub item_type: Option<ItemType>,
    pub limit: Option<usize>,
}

impl TaskFilter {
    pub fn matches(&self, item: &TaskItem) -> bool {
        // Status filter
        if let Some(status) = &self.status {
            if &item.frontmatter.status != status {
                return false;
            }
        }

        // Type filter
        if let Some(item_type) = &self.item_type {
            if &item.frontmatter.item_type != item_type {
                return false;
            }
        }

        // Tags filter (item must have ALL specified tags)
        if !self.tags.is_empty() {
            for tag in &self.tags {
                if !item.has_tag(tag) {
                    return false;
                }
            }
        }

        true
    }
}
