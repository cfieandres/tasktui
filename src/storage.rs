use crate::models::{Frontmatter, TaskItem, TaskFilter};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Storage manager for task files
pub struct Storage {
    pub data_dir: PathBuf,
}

impl Storage {
    /// Create a new storage manager
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        // Create data directory if it doesn't exist
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)
                .context("Failed to create data directory")?;
        }
        Ok(Self { data_dir })
    }

    /// Parse a markdown file with YAML frontmatter
    pub fn parse_file(&self, path: &Path) -> Result<TaskItem> {
        let content = fs::read_to_string(path)
            .context("Failed to read file")?;

        // Split frontmatter and body
        let parts: Vec<&str> = content.splitn(3, "---").collect();

        if parts.len() < 3 {
            anyhow::bail!("Invalid file format: missing frontmatter delimiters");
        }

        // Parse frontmatter (skip first empty part before first ---)
        let frontmatter: Frontmatter = serde_yaml::from_str(parts[1].trim())
            .context("Failed to parse frontmatter")?;

        // Get body (after second ---)
        let body = parts[2].trim().to_string();

        Ok(TaskItem {
            frontmatter,
            body,
            file_path: path.to_path_buf(),
        })
    }

    /// Serialize a task item to markdown with frontmatter
    pub fn serialize_task(&self, item: &TaskItem) -> Result<String> {
        let frontmatter = serde_yaml::to_string(&item.frontmatter)
            .context("Failed to serialize frontmatter")?;

        Ok(format!(
            "---\n{}---\n\n{}",
            frontmatter,
            item.body
        ))
    }

    /// Write a task item to disk
    pub fn write_task(&self, item: &TaskItem) -> Result<PathBuf> {
        let filename = format!("{}.md", item.frontmatter.id);
        let path = self.data_dir.join(&filename);

        let content = self.serialize_task(item)?;
        fs::write(&path, content)
            .context("Failed to write task file")?;

        Ok(path)
    }

    /// Load all tasks from the data directory
    pub fn load_all_tasks(&self) -> Result<Vec<TaskItem>> {
        let mut tasks = Vec::new();

        if !self.data_dir.exists() {
            return Ok(tasks);
        }

        for entry in fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                match self.parse_file(&path) {
                    Ok(task) => tasks.push(task),
                    Err(e) => {
                        eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(tasks)
    }

    /// List tasks with filtering
    pub fn list_tasks(&self, filter: &TaskFilter) -> Result<Vec<TaskItem>> {
        let mut tasks = self.load_all_tasks()?;

        // Apply filter
        tasks.retain(|task| filter.matches(task));

        // Sort by priority (high to low) then by created date
        tasks.sort_by(|a, b| {
            b.frontmatter.priority.cmp(&a.frontmatter.priority)
                .then_with(|| b.frontmatter.created_at.cmp(&a.frontmatter.created_at))
        });

        // Apply limit
        if let Some(limit) = filter.limit {
            tasks.truncate(limit);
        }

        Ok(tasks)
    }

    /// Delete a task file
    pub fn delete_task(&self, item: &TaskItem) -> Result<()> {
        fs::remove_file(&item.file_path)
            .context("Failed to delete task file")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ItemType, Status, Priority};
    use tempfile::TempDir;

    #[test]
    fn test_parse_and_write() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path().to_path_buf()).unwrap();

        let mut task = TaskItem::new("Test Task".to_string(), ItemType::Task);
        task.body = "This is a test task.".to_string();
        task.frontmatter.priority = Priority::High;
        task.frontmatter.tags = vec!["test".to_string(), "work".to_string()];

        let path = storage.write_task(&task).unwrap();
        let loaded = storage.parse_file(&path).unwrap();

        assert_eq!(loaded.frontmatter.title, "Test Task");
        assert_eq!(loaded.body, "This is a test task.");
        assert_eq!(loaded.frontmatter.priority, Priority::High);
    }
}
