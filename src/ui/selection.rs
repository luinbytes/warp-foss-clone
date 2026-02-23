//! Mouse selection and copy/paste support for the terminal.
//!
//! This module provides:
//! - Mouse tracking mode (CSI ?1000h and CSI ?1001h)
//! - Click and drag selection
//! - Copy to clipboard
//! - Paste from clipboard

use crate::terminal::grid::{Cell, Cursor};
use std::sync::{Arc, Mutex};
use anyhow::Result;

/// Mouse tracking mode flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MouseMode {
    /// Track button press and release events (CSI ?1000h)
    pub button_tracking: bool,
    /// Track button press events only (CSI ?1001h)
    pub button_press_only: bool,
}

impl MouseMode {
    /// Check if mouse tracking is enabled
    pub fn is_enabled(&self) -> bool {
        self.button_tracking || self.button_press_only
    }

    /// Enable button tracking (CSI ?1000h)
    pub fn enable_button_tracking(&mut self) {
        self.button_tracking = true;
    }

    /// Disable button tracking (CSI ?1000l)
    pub fn disable_button_tracking(&mut self) {
        self.button_tracking = false;
    }

    /// Enable button press only tracking (CSI ?1001h)
    pub fn enable_button_press_only(&mut self) {
        self.button_press_only = true;
    }

    /// Disable button press only tracking (CSI ?1001l)
    pub fn disable_button_press_only(&mut self) {
        self.button_press_only = false;
    }

    /// Disable all mouse tracking
    pub fn disable_all(&mut self) {
        self.button_tracking = false;
        self.button_press_only = false;
    }
}

/// Selection region defined by start and end positions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionRegion {
    /// Start position (inclusive)
    pub start: Cursor,
    /// End position (inclusive)
    pub end: Cursor,
    /// Whether this selection has been initialized (distinguishes from "no selection")
    pub active: bool,
}

impl SelectionRegion {
    /// Create a new selection region
    pub fn new(start: Cursor, end: Cursor) -> Self {
        // Normalize so start is always before end
        let (start, end) = if start.row < end.row || (start.row == end.row && start.col <= end.col) {
            (start, end)
        } else {
            (end, start)
        };
        // Explicitly created regions are always active
        Self { start, end, active: true }
    }

    /// Create an empty selection region
    pub fn empty() -> Self {
        Self {
            start: Cursor::origin(),
            end: Cursor::origin(),
            active: false,
        }
    }

    /// Check if the selection is empty (not active)
    pub fn is_empty(&self) -> bool {
        !self.active
    }

    /// Check if a position is within the selection
    pub fn contains(&self, pos: Cursor) -> bool {
        if self.is_empty() {
            return false;
        }
        if pos.row >= self.start.row && pos.row <= self.end.row {
            if pos.row == self.start.row && pos.col < self.start.col {
                return false;
            }
            if pos.row == self.end.row && pos.col > self.end.col {
                return false;
            }
            return true;
        }
        false
    }

    /// Get the range of columns for a given row
    pub fn cols_for_row(&self, row: usize) -> Option<(usize, usize)> {
        if self.is_empty() {
            return None;
        }
        if row < self.start.row || row > self.end.row {
            return None;
        }
        let start_col = if row == self.start.row { self.start.col } else { 0 };
        let end_col = if row == self.end.row { self.end.col } else { usize::MAX };
        Some((start_col, end_col))
    }
}

impl Default for SelectionRegion {
    fn default() -> Self {
        Self::empty()
    }
}

/// Selection state
#[derive(Debug, Clone)]
pub struct SelectionState {
    /// Current selection region (if any)
    pub region: SelectionRegion,
    /// Whether we're currently dragging to select
    pub selecting: bool,
    /// Mouse tracking mode
    pub mouse_mode: MouseMode,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            region: SelectionRegion::empty(),
            selecting: false,
            mouse_mode: MouseMode::default(),
        }
    }
}

impl SelectionState {
    /// Create a new selection state
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a selection at the given position
    pub fn start_selection(&mut self, pos: Cursor) {
        // Create a region with same start and end, but mark as inactive until dragged
        self.region = SelectionRegion {
            start: pos,
            end: pos,
            active: false, // Not active until user drags
        };
        self.selecting = true;
    }

    /// Update the selection to include a new position
    pub fn update_selection(&mut self, pos: Cursor) {
        if self.selecting {
            let (start, end) = if self.region.start.row < pos.row
                || (self.region.start.row == pos.row && self.region.start.col <= pos.col)
            {
                (self.region.start, pos)
            } else {
                (pos, self.region.start)
            };
            self.region = SelectionRegion {
                start,
                end,
                active: self.region.start != pos, // Active only if dragged to different position
            };
        }
    }

    /// End the selection
    pub fn end_selection(&mut self) {
        self.selecting = false;
    }

    /// Clear the selection
    pub fn clear(&mut self) {
        self.region = SelectionRegion::empty();
        self.selecting = false;
    }

    /// Check if there's an active selection
    pub fn has_selection(&self) -> bool {
        !self.region.is_empty()
    }
}

/// Extract selected text from the grid
pub fn extract_selected_text(grid: &[Vec<Cell>], selection: &SelectionRegion) -> String {
    if !selection.active || grid.is_empty() {
        return String::new();
    }

    let mut result = String::new();

    for row in selection.start.row..=selection.end.row.min(grid.len().saturating_sub(1)) {
        let row_data = &grid[row];
        let (start_col, end_col) = selection.cols_for_row(row).unwrap();

        // For multi-line selections, use the minimum column across rows for both start and end
        let effective_start = if selection.start.row != selection.end.row {
            selection.start.col.min(selection.end.col)
        } else {
            start_col
        };

        let effective_end = if selection.start.row != selection.end.row {
            selection.start.col.max(selection.end.col)
        } else {
            end_col
        };

        let actual_end = effective_end.min(row_data.len().saturating_sub(1));

        // Trim trailing whitespace on the last line only
        let trim_end = if row == selection.end.row {
            row_data[effective_start..=actual_end]
                .iter()
                .rev()
                .take_while(|c| c.char.is_whitespace())
                .count()
        } else {
            0
        };

        // Trim leading whitespace on the first line only
        let trim_start = if row == selection.start.row {
            row_data[effective_start..=actual_end]
                .iter()
                .take_while(|c| c.char.is_whitespace())
                .count()
        } else {
            0
        };

        let final_start = effective_start + trim_start;
        let final_end = if row == selection.end.row && trim_end > 0 {
            actual_end.saturating_sub(trim_end)
        } else {
            actual_end
        };

        for col in final_start..=final_end {
            let cell = &row_data[col];
            result.push(cell.char);
        }

        // Add newline between rows, but not after the last row
        if row < selection.end.row && row < grid.len().saturating_sub(1) {
            result.push('\n');
        }
    }

    result
}

/// Extract selected text including leading whitespace
pub fn extract_selected_text_preserve_ws(grid: &[Vec<Cell>], selection: &SelectionRegion) -> String {
    if !selection.active || grid.is_empty() {
        return String::new();
    }

    let mut result = String::new();

    for row in selection.start.row..=selection.end.row.min(grid.len().saturating_sub(1)) {
        let row_data = &grid[row];
        let (start_col, end_col) = selection.cols_for_row(row).unwrap();

        // For multi-line selections, use the same columns for all rows
        let effective_start = if selection.start.row != selection.end.row {
            selection.start.col.min(selection.end.col)
        } else {
            start_col
        };

        let effective_end = if selection.start.row != selection.end.row {
            selection.start.col.max(selection.end.col)
        } else {
            end_col
        };

        let actual_end = effective_end.min(row_data.len().saturating_sub(1));

        for col in effective_start..=actual_end {
            let cell = &row_data[col];
            result.push(cell.char);
        }

        // Add newline between rows, but not after the last row
        if row < selection.end.row && row < grid.len().saturating_sub(1) {
            result.push('\n');
        }
    }

    result
}

/// Clipboard manager
pub struct Clipboard {
    clipboard: Arc<Mutex<Option<arboard::Clipboard>>>,
}

impl Clipboard {
    /// Create a new clipboard manager
    pub fn new() -> Self {
        Self {
            clipboard: Arc::new(Mutex::new(None)),
        }
    }

    /// Initialize the clipboard (must be called from the main thread)
    pub fn init(&self) -> Result<()> {
        let mut inner = self.clipboard.lock().unwrap();
        *inner = Some(arboard::Clipboard::new()?);
        Ok(())
    }

    /// Copy text to the clipboard
    pub fn copy(&self, text: &str) -> Result<()> {
        let mut inner = self.clipboard.lock().unwrap();
        if let Some(clipboard) = inner.as_mut() {
            clipboard.set_text(text)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Clipboard not initialized"))
        }
    }

    /// Get text from the clipboard
    pub fn paste(&self) -> Result<String> {
        let mut inner = self.clipboard.lock().unwrap();
        if let Some(clipboard) = inner.as_mut() {
            let text = clipboard.get_text()?;
            Ok(text)
        } else {
            Err(anyhow::anyhow!("Clipboard not initialized"))
        }
    }

    /// Check if clipboard is available
    pub fn is_available(&self) -> bool {
        let inner = self.clipboard.lock().unwrap();
        inner.is_some()
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::grid::Cell;

    fn create_test_grid() -> Vec<Vec<Cell>> {
        vec![
            vec![
                Cell::new('H'),
                Cell::new('e'),
                Cell::new('l'),
                Cell::new('l'),
                Cell::new('o'),
                Cell::new(' '),
                Cell::new('W'),
                Cell::new('o'),
                Cell::new('r'),
                Cell::new('l'),
                Cell::new('d'),
            ],
            vec![
                Cell::new('T'),
                Cell::new('e'),
                Cell::new('s'),
                Cell::new('t'),
                Cell::new(' '),
                Cell::new('L'),
                Cell::new('i'),
                Cell::new('n'),
                Cell::new('e'),
                Cell::new(' '),
                Cell::new('2'),
            ],
            vec![
                Cell::new('A'),
                Cell::new('n'),
                Cell::new('o'),
                Cell::new('t'),
                Cell::new('h'),
                Cell::new('e'),
                Cell::new('r'),
            ],
        ]
    }

    #[test]
    fn test_selection_region_creation() {
        let start = Cursor::new(0, 0);
        let end = Cursor::new(1, 5);
        let region = SelectionRegion::new(start, end);

        assert_eq!(region.start, start);
        assert_eq!(region.end, end);
        assert!(!region.is_empty());
    }

    #[test]
    fn test_selection_region_normalization() {
        // Create with end before start
        let start = Cursor::new(1, 5);
        let end = Cursor::new(0, 0);
        let region = SelectionRegion::new(start, end);

        // Should be normalized
        assert_eq!(region.start, Cursor::new(0, 0));
        assert_eq!(region.end, Cursor::new(1, 5));
    }

    #[test]
    fn test_selection_region_same_position() {
        let pos = Cursor::new(5, 10);
        let region = SelectionRegion::new(pos, pos);

        // A single-cell selection is active (explicitly created)
        assert!(!region.is_empty());
        assert_eq!(region.start, pos);
        assert_eq!(region.end, pos);
    }

    #[test]
    fn test_selection_region_contains() {
        let region = SelectionRegion::new(
            Cursor::new(1, 3),
            Cursor::new(3, 7),
        );

        // Inside selection
        assert!(region.contains(Cursor::new(2, 5)));
        assert!(region.contains(Cursor::new(1, 4)));
        assert!(region.contains(Cursor::new(3, 6)));

        // Outside selection
        assert!(!region.contains(Cursor::new(0, 5)));
        assert!(!region.contains(Cursor::new(4, 5)));
        assert!(!region.contains(Cursor::new(1, 2)));
        assert!(!region.contains(Cursor::new(3, 8)));
    }

    #[test]
    fn test_selection_region_cols_for_row() {
        let region = SelectionRegion::new(
            Cursor::new(1, 3),
            Cursor::new(3, 7),
        );

        // Middle row - full range
        assert_eq!(region.cols_for_row(2), Some((0, usize::MAX)));

        // Start row - partial range
        assert_eq!(region.cols_for_row(1), Some((3, usize::MAX)));

        // End row - partial range
        assert_eq!(region.cols_for_row(3), Some((0, 7)));

        // Outside rows
        assert_eq!(region.cols_for_row(0), None);
        assert_eq!(region.cols_for_row(4), None);
    }

    #[test]
    fn test_extract_selected_text_single_char() {
        let grid = create_test_grid();
        let selection = SelectionRegion::new(
            Cursor::new(0, 0),
            Cursor::new(0, 0),
        );

        let text = extract_selected_text(&grid, &selection);
        assert_eq!(text, "H");
    }

    #[test]
    fn test_extract_selected_text_single_line() {
        let grid = create_test_grid();
        let selection = SelectionRegion::new(
            Cursor::new(0, 0),
            Cursor::new(0, 4),
        );

        let text = extract_selected_text(&grid, &selection);
        assert_eq!(text, "Hello");
    }

    #[test]
    fn test_extract_selected_text_multiple_lines() {
        let grid = create_test_grid();
        let selection = SelectionRegion::new(
            Cursor::new(0, 0),
            Cursor::new(1, 4),
        );

        let text = extract_selected_text(&grid, &selection);
        assert_eq!(text, "Hello\nTest");
    }

    #[test]
    fn test_extract_selected_text_trailing_whitespace() {
        let grid = create_test_grid();
        let selection = SelectionRegion::new(
            Cursor::new(0, 5),
            Cursor::new(0, 11),
        );

        let text = extract_selected_text(&grid, &selection);
        // Should trim trailing whitespace
        assert_eq!(text, "World");
    }

    #[test]
    fn test_extract_selected_text_preserve_ws() {
        let grid = create_test_grid();
        let selection = SelectionRegion::new(
            Cursor::new(0, 5),
            Cursor::new(0, 11),
        );

        let text = extract_selected_text_preserve_ws(&grid, &selection);
        // Should preserve whitespace
        assert_eq!(text, " World");
    }

    #[test]
    fn test_extract_selected_text_empty_selection() {
        let grid = create_test_grid();
        let selection = SelectionRegion::empty();

        let text = extract_selected_text(&grid, &selection);
        assert_eq!(text, "");
    }

    #[test]
    fn test_mouse_mode() {
        let mut mode = MouseMode::default();
        assert!(!mode.is_enabled());

        mode.enable_button_tracking();
        assert!(mode.is_enabled());
        assert!(mode.button_tracking);

        mode.disable_button_tracking();
        assert!(!mode.is_enabled());

        mode.enable_button_press_only();
        assert!(mode.is_enabled());
        assert!(mode.button_press_only);

        mode.disable_all();
        assert!(!mode.is_enabled());
    }

    #[test]
    fn test_selection_state() {
        let mut state = SelectionState::new();
        assert!(!state.has_selection());
        assert!(!state.selecting);

        state.start_selection(Cursor::new(2, 3));
        assert!(state.selecting);
        assert!(!state.has_selection()); // Single char is still considered empty

        state.update_selection(Cursor::new(5, 10));
        assert!(state.selecting);
        assert!(state.has_selection());

        state.end_selection();
        assert!(!state.selecting);
        assert!(state.has_selection());

        state.clear();
        assert!(!state.selecting);
        assert!(!state.has_selection());
    }
}
