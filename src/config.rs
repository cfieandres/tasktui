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

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub workstreams: Vec<Workstream>,
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
}
