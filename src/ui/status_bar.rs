//! Status bar for displaying current directory and git branch
//!
//! Renders a status bar at the bottom of the terminal with:
//! - Current working directory
//! - Git branch (if in a git repository)
//! - Other useful information

use std::path::Path;
use std::process::Command;

/// Status bar information
#[derive(Debug, Clone)]
pub struct StatusBar {
    /// Current working directory
    pub current_dir: String,
    /// Git branch (None if not in a git repo)
    pub git_branch: Option<String>,
    /// Whether the status bar is visible
    pub visible: bool,
}

impl StatusBar {
    /// Create a new status bar
    pub fn new() -> Self {
        Self {
            current_dir: String::new(),
            git_branch: None,
            visible: true,
        }
    }

    /// Update the status bar with the current directory
    pub fn update(&mut self, dir: &str) {
        self.current_dir = dir.to_string();
        self.git_branch = Self::get_git_branch(dir);
    }

    /// Get the git branch for a directory
    fn get_git_branch(dir: &str) -> Option<String> {
        let path = Path::new(dir);

        // Try to get git branch using git command
        let output = Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(path)
            .output()
            .ok()?;

        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !branch.is_empty() && branch != "HEAD" {
                return Some(branch);
            }
        }

        None
    }

    /// Toggle status bar visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Check if status bar is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_status_bar_creation() {
        let status_bar = StatusBar::new();
        assert!(status_bar.current_dir.is_empty());
        assert!(status_bar.git_branch.is_none());
        assert!(status_bar.visible);
    }

    #[test]
    fn test_status_bar_toggle() {
        let mut status_bar = StatusBar::new();
        assert!(status_bar.is_visible());

        status_bar.toggle();
        assert!(!status_bar.is_visible());

        status_bar.toggle();
        assert!(status_bar.is_visible());
    }

    #[test]
    fn test_status_bar_update() {
        let mut status_bar = StatusBar::new();
        let current_dir = env::current_dir().unwrap();
        let dir_str = current_dir.to_string_lossy();

        status_bar.update(&dir_str);
        assert_eq!(status_bar.current_dir, dir_str);
    }

    #[test]
    fn test_git_branch_in_repo() {
        // This test assumes we're running in a git repository
        let current_dir = env::current_dir().unwrap();
        let dir_str = current_dir.to_string_lossy();

        let branch = StatusBar::get_git_branch(&dir_str);
        // In a git repo, we should get a branch name
        // (unless in detached HEAD state)
        if let Some(branch_name) = branch {
            assert!(!branch_name.is_empty());
        }
    }
}
