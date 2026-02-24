//! Search functionality for terminal content

use regex::Regex;
use std::ops::Range;

/// Represents a search match in the terminal
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    /// Row index (in scrollback+grid space)
    pub row: usize,
    /// Column range of the match
    pub cols: Range<usize>,
}

/// Search state for the terminal
#[derive(Debug, Clone)]
pub struct SearchState {
    /// Current search pattern (as regex)
    pattern: Option<Regex>,
    /// All matches found
    matches: Vec<SearchMatch>,
    /// Index of currently selected match
    current_match_index: Option<usize>,
    /// Whether search mode is active
    pub active: bool,
    /// Search query string
    pub query: String,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            pattern: None,
            matches: Vec::new(),
            current_match_index: None,
            active: false,
            query: String::new(),
        }
    }
}

impl SearchState {
    /// Create a new search state
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the search pattern
    pub fn set_pattern(&mut self, query: &str) -> Result<(), regex::Error> {
        if query.is_empty() {
            self.clear();
            return Ok(());
        }

        // Build regex pattern (case-insensitive by default)
        let pattern = Regex::new(&format!("(?i){}", query))?;
        self.pattern = Some(pattern);
        self.query = query.to_string();
        self.matches.clear();
        self.current_match_index = None;
        Ok(())
    }

    /// Clear the search
    pub fn clear(&mut self) {
        self.pattern = None;
        self.matches.clear();
        self.current_match_index = None;
        self.active = false;
        self.query.clear();
    }

    /// Find all matches in the given text lines
    ///
    /// # Arguments
    /// * `lines` - Iterator of (row_index, text) pairs
    pub fn find_matches<'a, I>(&mut self, lines: I)
    where
        I: Iterator<Item = (usize, &'a str)>,
    {
        self.matches.clear();

        if let Some(ref pattern) = self.pattern {
            for (row, text) in lines {
                for mat in pattern.find_iter(text) {
                    self.matches.push(SearchMatch {
                        row,
                        cols: mat.start()..mat.end(),
                    });
                }
            }
        }

        // Select first match if available
        self.current_match_index = if self.matches.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    /// Get the number of matches
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Get the current match (1-indexed for display)
    pub fn current_match_number(&self) -> Option<usize> {
        self.current_match_index.map(|i| i + 1)
    }

    /// Navigate to the next match
    pub fn next_match(&mut self) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }

        let next_index = match self.current_match_index {
            Some(i) => (i + 1) % self.matches.len(),
            None => 0,
        };

        self.current_match_index = Some(next_index);
        self.matches.get(next_index)
    }

    /// Navigate to the previous match
    pub fn prev_match(&mut self) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }

        let prev_index = match self.current_match_index {
            Some(i) => {
                if i == 0 {
                    self.matches.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        self.current_match_index = Some(prev_index);
        self.matches.get(prev_index)
    }

    /// Get the currently selected match
    pub fn current_match(&self) -> Option<&SearchMatch> {
        self.current_match_index.and_then(|i| self.matches.get(i))
    }

    /// Check if a cell at (row, col) is part of the current match
    pub fn is_current_match(&self, row: usize, col: usize) -> bool {
        if let Some(ref current) = self.current_match() {
            current.row == row && current.cols.contains(&col)
        } else {
            false
        }
    }

    /// Check if a cell at (row, col) is part of any match
    pub fn is_match(&self, row: usize, col: usize) -> bool {
        self.matches.iter().any(|m| m.row == row && m.cols.contains(&col))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_state_creation() {
        let state = SearchState::new();
        assert!(!state.active);
        assert!(state.query.is_empty());
        assert_eq!(state.match_count(), 0);
    }

    #[test]
    fn test_set_pattern() {
        let mut state = SearchState::new();
        state.set_pattern("test").unwrap();
        assert!(state.pattern.is_some());
        assert_eq!(state.query, "test");
    }

    #[test]
    fn test_find_matches() {
        let mut state = SearchState::new();
        state.set_pattern("test").unwrap();

        let lines = vec![
            (0, "this is a test string"),
            (1, "another test"),
            (2, "no match here"),
        ];

        state.find_matches(lines.iter().map(|(r, t)| (*r, *t)));

        assert_eq!(state.match_count(), 2);
        assert_eq!(state.current_match_number(), Some(1));
    }

    #[test]
    fn test_next_prev_match() {
        let mut state = SearchState::new();
        state.set_pattern("test").unwrap();

        let lines = vec![
            (0, "test 1"),
            (1, "test 2"),
            (2, "test 3"),
        ];

        state.find_matches(lines.iter().map(|(r, t)| (*r, *t)));

        // Initial match should be first
        let m1 = state.current_match().unwrap();
        assert_eq!(m1.row, 0);

        // Next should be second
        let m2 = state.next_match().unwrap();
        assert_eq!(m2.row, 1);

        // Next should be third
        let m3 = state.next_match().unwrap();
        assert_eq!(m3.row, 2);

        // Next should wrap to first
        let m4 = state.next_match().unwrap();
        assert_eq!(m4.row, 0);

        // Prev should go back to third
        let m5 = state.prev_match().unwrap();
        assert_eq!(m5.row, 2);
    }

    #[test]
    fn test_regex_pattern() {
        let mut state = SearchState::new();
        state.set_pattern("te.*t").unwrap();

        let lines = vec![
            (0, "test text"),
        ];

        state.find_matches(lines.iter().map(|(r, t)| (*r, *t)));

        // Should match the entire "test text" (greedy match)
        assert_eq!(state.match_count(), 1);
        let m = state.current_match().unwrap();
        assert_eq!(m.cols.start, 0);
        assert_eq!(m.cols.end, 9);
    }

    #[test]
    fn test_case_insensitive() {
        let mut state = SearchState::new();
        state.set_pattern("TEST").unwrap();

        let lines = vec![
            (0, "test Test TEST"),
        ];

        state.find_matches(lines.iter().map(|(r, t)| (*r, *t)));

        // Should match all three variations
        assert_eq!(state.match_count(), 3);
    }

    #[test]
    fn test_is_match() {
        let mut state = SearchState::new();
        state.set_pattern("test").unwrap();

        let lines = vec![
            (0, "this is a test"),
        ];

        state.find_matches(lines.iter().map(|(r, t)| (*r, *t)));

        // "test" starts at column 10
        assert!(state.is_match(0, 10));
        assert!(state.is_match(0, 11));
        assert!(state.is_match(0, 12));
        assert!(state.is_match(0, 13));
        assert!(!state.is_match(0, 9));
        assert!(!state.is_match(0, 14));
    }
}
