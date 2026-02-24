//! AI Command Palette for AI-assisted terminal commands
//!
//! Provides a UI overlay that allows users to interact with AI
//! for command suggestions, explanations, and assistance.

use crate::ai::openai::{OpenAIConfig, OpenAIProvider};
use crate::ai::provider::AIProvider;
use std::time::Instant;

/// State of the AI command palette
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteState {
    /// Palette is hidden
    Hidden,
    /// Palette is open and waiting for input
    Open,
    /// Processing AI request
    Processing,
    /// Displaying AI response
    ShowingResponse,
}

/// AI command palette for AI-assisted commands
pub struct AICommandPalette {
    /// Current state of the palette
    pub state: PaletteState,
    /// User input buffer
    pub input: String,
    /// AI response buffer
    pub response: String,
    /// Cursor position in input buffer
    pub cursor_pos: usize,
    /// OpenAI provider (optional - may not be configured)
    provider: Option<OpenAIProvider>,
    /// Timestamp when processing started (for timeout)
    processing_start: Option<Instant>,
    /// Error message if any
    pub error: Option<String>,
}

impl AICommandPalette {
    /// Create a new AI command palette
    pub fn new() -> Self {
        Self {
            state: PaletteState::Hidden,
            input: String::new(),
            response: String::new(),
            cursor_pos: 0,
            provider: None,
            processing_start: None,
            error: None,
        }
    }

    /// Initialize the AI provider
    pub fn initialize_provider(&mut self) -> Result<(), String> {
        match OpenAIProvider::from_keyring(None) {
            Ok(provider) => {
                self.provider = Some(provider);
                Ok(())
            }
            Err(e) => Err(format!("Failed to initialize AI provider: {}", e)),
        }
    }

    /// Open the palette
    pub fn open(&mut self) {
        self.state = PaletteState::Open;
        self.input.clear();
        self.response.clear();
        self.cursor_pos = 0;
        self.error = None;
    }

    /// Close the palette
    pub fn close(&mut self) {
        self.state = PaletteState::Hidden;
        self.input.clear();
        self.response.clear();
        self.cursor_pos = 0;
        self.error = None;
    }

    /// Toggle the palette (open if closed, close if open)
    pub fn toggle(&mut self) {
        match self.state {
            PaletteState::Hidden => self.open(),
            _ => self.close(),
        }
    }

    /// Check if the palette is visible
    pub fn is_visible(&self) -> bool {
        self.state != PaletteState::Hidden
    }

    /// Handle character input
    pub fn handle_char(&mut self, c: char) {
        if self.state == PaletteState::Open {
            self.input.insert(self.cursor_pos, c);
            self.cursor_pos += c.len_utf8();
        }
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        if self.state == PaletteState::Open && self.cursor_pos > 0 {
            // Find the byte position before cursor
            let prev_pos = self.input[..self.cursor_pos]
                .chars()
                .rev()
                .next()
                .map(|c| self.cursor_pos - c.len_utf8())
                .unwrap_or(0);

            self.input.remove(prev_pos);
            self.cursor_pos = prev_pos;
        }
    }

    /// Handle Enter key - submit the command
    pub fn handle_enter(&mut self) {
        if self.state == PaletteState::Open && !self.input.is_empty() {
            self.submit_command();
        } else if self.state == PaletteState::ShowingResponse {
            // Close after viewing response
            self.close();
        }
    }

    /// Handle escape key
    pub fn handle_escape(&mut self) {
        self.close();
    }

    /// Move cursor left
    pub fn cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            let prev_char_len = self.input[..self.cursor_pos]
                .chars()
                .rev()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(0);
            self.cursor_pos -= prev_char_len;
        }
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        if self.cursor_pos < self.input.len() {
            let next_char_len = self.input[self.cursor_pos..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(0);
            self.cursor_pos += next_char_len;
        }
    }

    /// Submit the command to AI
    fn submit_command(&mut self) {
        if self.provider.is_none() {
            // Try to initialize provider
            if let Err(e) = self.initialize_provider() {
                self.error = Some(e);
                return;
            }
        }

        if let Some(provider) = &self.provider {
            self.state = PaletteState::Processing;
            self.processing_start = Some(Instant::now());
            self.response.clear();
            self.error = None;

            // For now, we'll handle this synchronously in a blocking way
            // In a real implementation, this would be async with proper UI feedback
            let prompt = format!(
                "You are a terminal assistant. The user asks: {}\n\nProvide a helpful response that could be a command, explanation, or guidance.",
                self.input
            );

            // Note: This is a blocking call. In production, this should be async
            // and the UI should show a loading indicator
            match tokio::runtime::Handle::try_current() {
                Ok(handle) => {
                    // We're in a tokio runtime, use it
                    let provider_clone = OpenAIProvider::new(OpenAIConfig {
                        api_key: provider.api_key().to_string(),
                        model: provider.model().to_string(),
                    });

                    // This is a simplified synchronous wrapper
                    // In production, we'd use proper async/await with UI updates
                    self.response = "AI integration requires async runtime. This is a placeholder.".to_string();
                    self.state = PaletteState::ShowingResponse;
                }
                Err(_) => {
                    self.error = Some("No async runtime available".to_string());
                    self.state = PaletteState::Open;
                }
            }
        } else {
            self.error = Some("AI provider not configured. Please set up OpenAI API key.".to_string());
        }
    }

    /// Get suggested commands based on context
    pub fn get_suggestions(&self, _context: &str) -> Vec<String> {
        // TODO: Implement context-aware suggestions
        vec![
            "explain last command".to_string(),
            "suggest fix for error".to_string(),
            "generate command for...".to_string(),
        ]
    }

    /// Update processing state (call this in render loop)
    pub fn update(&mut self) {
        // Check for timeout
        if let Some(start) = self.processing_start {
            if start.elapsed().as_secs() > 30 {
                self.error = Some("AI request timed out".to_string());
                self.state = PaletteState::Open;
                self.processing_start = None;
            }
        }
    }
}

impl Default for AICommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_creation() {
        let palette = AICommandPalette::new();
        assert_eq!(palette.state, PaletteState::Hidden);
        assert!(palette.input.is_empty());
        assert!(palette.response.is_empty());
    }

    #[test]
    fn test_palette_toggle() {
        let mut palette = AICommandPalette::new();
        assert_eq!(palette.state, PaletteState::Hidden);

        palette.toggle();
        assert_eq!(palette.state, PaletteState::Open);

        palette.toggle();
        assert_eq!(palette.state, PaletteState::Hidden);
    }

    #[test]
    fn test_palette_input() {
        let mut palette = AICommandPalette::new();
        palette.open();

        palette.handle_char('h');
        palette.handle_char('e');
        palette.handle_char('l');
        palette.handle_char('p');

        assert_eq!(palette.input, "help");
        assert_eq!(palette.cursor_pos, 4);
    }

    #[test]
    fn test_palette_backspace() {
        let mut palette = AICommandPalette::new();
        palette.open();

        palette.handle_char('t');
        palette.handle_char('e');
        palette.handle_char('s');
        palette.handle_char('t');
        assert_eq!(palette.input, "test");

        palette.handle_backspace();
        assert_eq!(palette.input, "tes");
        assert_eq!(palette.cursor_pos, 3);
    }

    #[test]
    fn test_palette_cursor_movement() {
        let mut palette = AICommandPalette::new();
        palette.open();

        palette.handle_char('a');
        palette.handle_char('b');
        palette.handle_char('c');
        assert_eq!(palette.cursor_pos, 3);

        palette.cursor_left();
        assert_eq!(palette.cursor_pos, 2);

        palette.cursor_right();
        assert_eq!(palette.cursor_pos, 3);
    }

    #[test]
    fn test_palette_suggestions() {
        let palette = AICommandPalette::new();
        let suggestions = palette.get_suggestions("test context");
        assert!(!suggestions.is_empty());
        assert!(suggestions.contains(&"explain last command".to_string()));
    }
}
