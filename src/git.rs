use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Git sync manager
pub struct GitSync {
    repo_path: std::path::PathBuf,
}

impl GitSync {
    pub fn new(repo_path: std::path::PathBuf) -> Self {
        Self { repo_path }
    }

    /// Execute git pull --rebase --autostash
    pub fn pull(&self) -> Result<()> {
        let output = Command::new("git")
            .arg("pull")
            .arg("--rebase")
            .arg("--autostash")
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to execute git pull")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git pull failed: {}", stderr);
        }

        Ok(())
    }

    /// Execute git add, commit, and push
    pub fn commit_and_push(&self, message: &str) -> Result<()> {
        // Git add
        let output = Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to execute git add")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git add failed: {}", stderr);
        }

        // Git commit
        let output = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(message)
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to execute git commit")?;

        // It's ok if commit fails (nothing to commit)
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Check if it's just "nothing to commit"
            if !stderr.contains("nothing to commit") && !stderr.contains("no changes added") {
                eprintln!("Warning: Git commit had issues: {}", stderr);
            }
        }

        // Git push
        let output = Command::new("git")
            .arg("push")
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to execute git push")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git push failed: {}", stderr);
        }

        Ok(())
    }

    /// Full sync workflow: pull, then push with changes
    pub fn sync(&self, message: &str) -> Result<()> {
        // Pre-write: pull with rebase and autostash
        self.pull().context("Pre-sync pull failed")?;

        // Post-write: commit and push
        self.commit_and_push(message).context("Post-sync push failed")?;

        Ok(())
    }

    /// Check if we're in a git repository
    pub fn is_git_repo(&self) -> bool {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--git-dir")
            .current_dir(&self.repo_path)
            .output();

        matches!(output, Ok(output) if output.status.success())
    }

    /// Initialize a git repository if it doesn't exist
    pub fn init_if_needed(&self) -> Result<()> {
        if !self.is_git_repo() {
            let output = Command::new("git")
                .arg("init")
                .current_dir(&self.repo_path)
                .output()
                .context("Failed to execute git init")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Git init failed: {}", stderr);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_git_init() {
        let temp_dir = TempDir::new().unwrap();
        let git_sync = GitSync::new(temp_dir.path().to_path_buf());

        assert!(!git_sync.is_git_repo());
        git_sync.init_if_needed().unwrap();
        assert!(git_sync.is_git_repo());
    }
}
