use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// A workstream (tag category) with a keyboard shortcut
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workstream {
    pub name: String,
    pub key: char, // '1'-'9'
}

/// A high-level goal or priority (GTD "Horizons of Focus")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub description: String,
    pub area: String,        // e.g., "work", "personal" - links to workstream
    pub priority: u8,        // 1-5, where 1 is highest priority
    #[serde(default)]
    pub active: bool,        // Whether this goal is currently active/relevant
}

impl Goal {
    pub fn new(description: String, area: String) -> Self {
        Self {
            description,
            area,
            priority: 3,     // Default to medium priority
            active: true,
        }
    }
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub workstreams: Vec<Workstream>,
    #[serde(default)]
    pub goals: Vec<Goal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openai_api_key: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            workstreams: vec![
                Workstream {
                    name: "work".to_string(),
                    key: '1',
                },
                Workstream {
                    name: "personal".to_string(),
                    key: '2',
                },
            ],
            goals: Vec::new(),
            openai_api_key: None,
        }
    }
}

impl AppConfig {
    /// Get the config file path for a data directory
    pub fn config_path(data_dir: &PathBuf) -> PathBuf {
        data_dir.join(".tasktui-config.yaml")
    }

    /// Load config from data directory, or create default if not found
    pub fn load(data_dir: &PathBuf) -> Result<Self> {
        let config_path = Self::config_path(data_dir);

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: AppConfig = serde_yaml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config
            let config = AppConfig::default();
            config.save(data_dir)?;
            Ok(config)
        }
    }

    /// Save config to data directory
    pub fn save(&self, data_dir: &PathBuf) -> Result<()> {
        let config_path = Self::config_path(data_dir);
        let content = serde_yaml::to_string(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    /// Add a new workstream with auto-assigned key
    pub fn add_workstream(&mut self, name: String) -> Option<char> {
        // Find next available key (3-9)
        let used_keys: Vec<char> = self.workstreams.iter().map(|w| w.key).collect();
        let next_key = ('3'..='9').find(|k| !used_keys.contains(k))?;

        self.workstreams.push(Workstream {
            name,
            key: next_key,
        });

        Some(next_key)
    }

    /// Rename a workstream
    pub fn rename_workstream(&mut self, old_name: &str, new_name: String) -> bool {
        if let Some(ws) = self.workstreams.iter_mut().find(|w| w.name == old_name) {
            ws.name = new_name;
            true
        } else {
            false
        }
    }

    /// Delete a workstream
    pub fn delete_workstream(&mut self, name: &str) -> bool {
        let initial_len = self.workstreams.len();
        self.workstreams.retain(|w| w.name != name);
        self.workstreams.len() < initial_len
    }

    /// Get workstream by key
    pub fn get_workstream_by_key(&self, key: char) -> Option<&Workstream> {
        self.workstreams.iter().find(|w| w.key == key)
    }

    /// Add a new goal
    pub fn add_goal(&mut self, description: String, area: String) {
        self.goals.push(Goal::new(description, area));
    }

    /// Update a goal's description
    pub fn update_goal(&mut self, index: usize, description: String) {
        if let Some(goal) = self.goals.get_mut(index) {
            goal.description = description;
        }
    }

    /// Update a goal's area
    pub fn update_goal_area(&mut self, index: usize, area: String) {
        if let Some(goal) = self.goals.get_mut(index) {
            goal.area = area;
        }
    }

    /// Cycle a goal's priority (1→2→3→4→5→1)
    pub fn cycle_goal_priority(&mut self, index: usize) {
        if let Some(goal) = self.goals.get_mut(index) {
            goal.priority = if goal.priority >= 5 { 1 } else { goal.priority + 1 };
        }
    }

    /// Toggle a goal's active state
    pub fn toggle_goal_active(&mut self, index: usize) {
        if let Some(goal) = self.goals.get_mut(index) {
            goal.active = !goal.active;
        }
    }

    /// Delete a goal
    pub fn delete_goal(&mut self, index: usize) {
        if index < self.goals.len() {
            self.goals.remove(index);
        }
    }

    /// Get active goals sorted by priority
    pub fn active_goals(&self) -> Vec<&Goal> {
        let mut goals: Vec<_> = self.goals.iter().filter(|g| g.active).collect();
        goals.sort_by_key(|g| g.priority);
        goals
    }

    /// Get goals for a specific area
    pub fn goals_by_area(&self, area: &str) -> Vec<&Goal> {
        self.goals.iter()
            .filter(|g| g.area.to_lowercase() == area.to_lowercase())
            .collect()
    }

    /// Format goals for LLM context
    pub fn goals_context(&self) -> String {
        let active = self.active_goals();
        if active.is_empty() {
            return String::new();
        }

        let mut context = String::from("Current priorities and goals:\n");
        for goal in active {
            let priority_stars = "★".repeat(6 - goal.priority as usize);
            context.push_str(&format!("- [{}] {}: {}\n", goal.area, priority_stars, goal.description));
        }
        context
    }
}
