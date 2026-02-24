//! AI Command Palette for AI-assisted terminal commands
//!
//! Provides a UI overlay that allows users to interact with AI
//! for command suggestions, explanations, and assistance.

use crate::ai::openai::{OpenAIConfig, OpenAIProvider};
use crate::ai::provider::{AIProvider, CompletionOptions};
use std::sync::{Arc, Mutex};
use std::thread;
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
    pub response: Arc<Mutex<String>>,
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
            response: Arc::new(Mutex::new(String::new())),
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
        if let Ok(mut response) = self.response.lock() {
            response.clear();
        }
        self.cursor_pos = 0;
        self.error = None;
    }

    /// Close the palette
    pub fn close(&mut self) {
        self.state = PaletteState::Hidden;
        self.input.clear();
        if let Ok(mut response) = self.response.lock() {
            response.clear();
        }
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
            if let Ok(mut response) = self.response.lock() {
                response.clear();
            }
            self.error = None;

            // Clone necessary data for the async thread
            let prompt = format!(
                "You are a terminal assistant. The user asks: {}\n\nProvide a helpful response. If suggesting a command, put it in a code block. Keep responses concise.",
                self.input
            );
            let api_key = provider.api_key().to_string();
            let model = provider.model().to_string();
            let response_arc = Arc::clone(&self.response);

            // Spawn a thread with tokio runtime to handle the async API call
            thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let config = OpenAIConfig {
                        api_key,
                        model,
                    };
                    let async_provider = OpenAIProvider::new(config);

                    let opts = CompletionOptions {
                        max_tokens: Some(500),
                        temperature: Some(0.7),
                    };

                    match async_provider.complete(&prompt, Some(opts)).await {
                        Ok(result) => {
                            if let Ok(mut response) = response_arc.lock() {
                                *response = result;
                            }
                        }
                        Err(e) => {
                            if let Ok(mut response) = response_arc.lock() {
                                *response = format!("Error: {}", e);
                            }
                        }
                    }
                });
            });
        } else {
            self.error = Some("AI provider not configured. Please set up OpenAI API key using: warp-foss config set-openai-key <key>".to_string());
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
            } else {
                // Check if response is ready
                if let Ok(response) = self.response.lock() {
                    if !response.is_empty() {
                        self.state = PaletteState::ShowingResponse;
                        self.processing_start = None;
                    }
                }
            }
        }
    }

    /// Get the current response text
    pub fn get_response(&self) -> String {
        if let Ok(response) = self.response.lock() {
            response.clone()
        } else {
            String::new()
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
        if let Ok(response) = palette.response.lock() {
            assert!(response.is_empty());
        }; // Semicolon to drop the guard
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

    #[test]
    fn test_get_response() {
        let palette = AICommandPalette::new();
        {
            if let Ok(mut response) = palette.response.lock() {
                *response = "Test response".to_string();
            }
        }
        assert_eq!(palette.get_response(), "Test response");
    }
}
