//! AI integration layer with BYOK support

pub mod anthropic;
pub mod ollama;
pub mod openai;
pub mod provider;

pub use provider::{AIError, AIProvider, CompletionOptions};
