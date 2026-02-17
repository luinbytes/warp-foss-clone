//! AI Provider abstraction for BYOK support

use async_trait::async_trait;
use futures::Stream;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AIError {
    #[error("API error: {0}")]
    Api(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Rate limited")]
    RateLimited,
}

#[derive(Debug, Clone)]
pub struct CompletionOptions {
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

/// Trait for AI providers - allows BYOK
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Get a completion from the AI
    async fn complete(&self, prompt: &str, opts: Option<CompletionOptions>) -> Result<String, AIError>;
    
    /// Stream a completion (for better UX)
    async fn stream(&self, prompt: &str, opts: Option<CompletionOptions>) -> Result<impl Stream<Item = Result<String, AIError>>, AIError>;
}
