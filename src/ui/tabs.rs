//! Tab management for multiple terminal sessions
//!
//! Provides tab functionality where each tab contains its own layout tree
//! with potentially multiple split panes.

use crate::terminal::pty::PtySession;
use uuid::Uuid;

use super::layout::{LayoutTree, Pane, Rect};

/// A single tab containing a layout tree of panes
pub struct Tab {
    /// Unique identifier for this tab
    pub id: Uuid,
    /// Tab title (derived from active pane or manually set)
    pub title: String,
    /// Layout tree containing all panes in this tab
    pub layout: LayoutTree,
    /// Whether this tab is active
    pub active: bool,
}

impl Tab {
    /// Create a new tab with an initial pane
    pub fn new(initial_pane: Pane) -> Self {
        let id = Uuid::new_v4();
        let title = Self::derive_title(&initial_pane);
        let layout = LayoutTree::new(initial_pane);

        Self {
            id,
            title,
            layout,
            active: false,
        }
    }

    /// Derive tab title from the focused pane
    pub fn derive_title(_pane: &Pane) -> String {
        // Try to get current directory from pane
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Terminal".to_string());

        // Extract just the last component of the path
        let title = cwd
            .rsplit('/')
            .next()
            .unwrap_or(&cwd)
            .to_string();

        if title.is_empty() {
            "Terminal".to_string()
        } else {
            title
        }
    }

    /// Update the tab title based on the focused pane
    pub fn update_title(&mut self) {
        if let Some(pane) = self.layout.focused_pane() {
            self.title = Self::derive_title(pane);
        }
    }

    /// Get the focused pane ID
    pub fn focused_pane_id(&self) -> Uuid {
        self.layout.focused_pane_id()
    }

    /// Get the number of panes in this tab
    pub fn pane_count(&self) -> usize {
        self.layout.pane_count()
    }
}

/// Manager for all tabs
pub struct TabManager {
    /// All tabs
    tabs: Vec<Tab>,
    /// Index of the currently active tab
    active_tab_index: usize,
    /// Tab bar height in pixels
    tab_bar_height: u32,
}

impl TabManager {
    /// Create a new tab manager with an initial tab
    pub fn new(initial_tab: Tab) -> Self {
        let mut tab = initial_tab;
        tab.active = true;

        Self {
            tabs: vec![tab],
            active_tab_index: 0,
            tab_bar_height: 32, // Default tab bar height
        }
    }

    /// Get the active tab
    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_tab_index)
    }

    /// Get the active tab mutably
    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active_tab_index)
    }

    /// Get the active tab index
    pub fn active_tab_index(&self) -> usize {
        self.active_tab_index
    }

    /// Get the number of tabs
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Get all tabs
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    /// Get tab bar height in pixels
    pub fn tab_bar_height(&self) -> u32 {
        self.tab_bar_height
    }

    /// Set tab bar height
    pub fn set_tab_bar_height(&mut self, height: u32) {
        self.tab_bar_height = height;
    }

    /// Create a new tab and switch to it
    pub fn new_tab(&mut self, initial_pane: Pane) -> usize {
        let tab = Tab::new(initial_pane);
        self.new_tab_from_tab(tab)
    }

    /// Add an existing Tab and switch to it
    pub fn new_tab_from_tab(&mut self, mut tab: Tab) -> usize {
        // Deactivate all other tabs
        for t in &mut self.tabs {
            t.active = false;
        }

        // Set new tab as active
        tab.active = true;

        // Add after current tab
        let insert_index = self.active_tab_index + 1;
        self.tabs.insert(insert_index, tab);
        self.active_tab_index = insert_index;

        insert_index
    }

    /// Close the current tab
    ///
    /// Returns Ok(()) if tab was closed, Err if this is the last tab
    pub fn close_current_tab(&mut self) -> Result<(), String> {
        if self.tabs.len() <= 1 {
            return Err("Cannot close the last tab".to_string());
        }

        // Remove the current tab
        self.tabs.remove(self.active_tab_index);

        // Adjust active index if needed
        if self.active_tab_index >= self.tabs.len() {
            self.active_tab_index = self.tabs.len() - 1;
        }

        // Activate the new current tab
        if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
            tab.active = true;
        }

        Ok(())
    }

    /// Switch to the next tab (circular)
    pub fn next_tab(&mut self) {
        if self.tabs.len() <= 1 {
            return;
        }

        // Deactivate current
        if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
            tab.active = false;
        }

        // Move to next (circular)
        self.active_tab_index = (self.active_tab_index + 1) % self.tabs.len();

        // Activate new
        if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
            tab.active = true;
        }
    }

    /// Switch to the previous tab (circular)
    pub fn prev_tab(&mut self) {
        if self.tabs.len() <= 1 {
            return;
        }

        // Deactivate current
        if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
            tab.active = false;
        }

        // Move to previous (circular)
        if self.active_tab_index == 0 {
            self.active_tab_index = self.tabs.len() - 1;
        } else {
            self.active_tab_index -= 1;
        }

        // Activate new
        if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
            tab.active = true;
        }
    }

    /// Switch to a specific tab by index (0-based)
    pub fn switch_to_tab(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }

        // Deactivate current
        if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
            tab.active = false;
        }

        // Switch
        self.active_tab_index = index;

        // Activate new
        if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
            tab.active = true;
        }

        true
    }

    /// Get content bounds (excluding tab bar)
    pub fn content_bounds(&self, window_width: u32, window_height: u32) -> Rect {
        Rect::new(
            0,
            self.tab_bar_height,
            window_width,
            window_height.saturating_sub(self.tab_bar_height),
        )
    }

    /// Update all tab titles based on their focused panes
    pub fn update_titles(&mut self) {
        for tab in &mut self.tabs {
            tab.update_title();
        }
    }

    /// Calculate layout for the active tab
    pub fn calculate_layout(&mut self, bounds: Rect) {
        if let Some(tab) = self.active_tab_mut() {
            tab.layout.calculate_layout(bounds);
        }
    }
}

/// Create a placeholder pane for testing
#[allow(dead_code)]
fn create_placeholder_pane() -> Pane {
    use crate::terminal::pty::PtyConfig;

    let pty = PtySession::spawn(PtyConfig::default()).unwrap();
    Pane::new(pty, 80, 24, Rect::new(0, 0, 800, 600))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::pty::PtyConfig;

    fn create_test_pane() -> Pane {
        let pty = PtySession::spawn(PtyConfig::default()).unwrap();
        Pane::new(pty, 80, 24, Rect::new(0, 0, 800, 600))
    }

    #[test]
    fn test_tab_creation() {
        let pane = create_test_pane();
        let tab = Tab::new(pane);
        assert!(!tab.id.is_nil());
        assert!(!tab.title.is_empty());
        assert_eq!(tab.pane_count(), 1);
    }

    #[test]
    fn test_tab_manager_creation() {
        let pane = create_test_pane();
        let tab = Tab::new(pane);
        let manager = TabManager::new(tab);

        assert_eq!(manager.tab_count(), 1);
        assert_eq!(manager.active_tab_index(), 0);
    }

    #[test]
    fn test_new_tab() {
        let pane1 = create_test_pane();
        let tab1 = Tab::new(pane1);
        let mut manager = TabManager::new(tab1);

        let pane2 = create_test_pane();
        manager.new_tab(pane2);

        assert_eq!(manager.tab_count(), 2);
        assert_eq!(manager.active_tab_index(), 1);
    }

    #[test]
    fn test_next_prev_tab() {
        let pane1 = create_test_pane();
        let tab1 = Tab::new(pane1);
        let mut manager = TabManager::new(tab1);

        let pane2 = create_test_pane();
        manager.new_tab(pane2);

        assert_eq!(manager.active_tab_index(), 1);

        manager.prev_tab();
        assert_eq!(manager.active_tab_index(), 0);

        manager.next_tab();
        assert_eq!(manager.active_tab_index(), 1);

        // Wrap around
        manager.next_tab();
        assert_eq!(manager.active_tab_index(), 0);
    }

    #[test]
    fn test_close_tab() {
        let pane1 = create_test_pane();
        let tab1 = Tab::new(pane1);
        let mut manager = TabManager::new(tab1);

        let pane2 = create_test_pane();
        manager.new_tab(pane2);

        assert_eq!(manager.tab_count(), 2);

        // Close current tab (second one)
        manager.close_current_tab().unwrap();
        assert_eq!(manager.tab_count(), 1);
        assert_eq!(manager.active_tab_index(), 0);

        // Cannot close last tab
        assert!(manager.close_current_tab().is_err());
    }

    #[test]
    fn test_switch_to_tab() {
        let pane1 = create_test_pane();
        let tab1 = Tab::new(pane1);
        let mut manager = TabManager::new(tab1);

        let pane2 = create_test_pane();
        manager.new_tab(pane2);

        let pane3 = create_test_pane();
        manager.new_tab(pane3);

        assert_eq!(manager.active_tab_index(), 2);

        manager.switch_to_tab(0);
        assert_eq!(manager.active_tab_index(), 0);

        manager.switch_to_tab(1);
        assert_eq!(manager.active_tab_index(), 1);

        // Invalid index
        assert!(!manager.switch_to_tab(10));
    }

    #[test]
    fn test_content_bounds() {
        let pane = create_test_pane();
        let tab = Tab::new(pane);
        let manager = TabManager::new(tab);

        let bounds = manager.content_bounds(800, 600);
        assert_eq!(bounds.x, 0);
        assert_eq!(bounds.y, 32); // tab bar height
        assert_eq!(bounds.width, 800);
        assert_eq!(bounds.height, 568); // 600 - 32
    }
}
