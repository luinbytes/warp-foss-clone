//! OpenAI API provider implementation

use crate::ai::provider::{AIError, AIProvider, CompletionOptions};
use async_trait::async_trait;
use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

/// OpenAI provider configuration
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    /// API key (will be stored in keychain in production)
    pub api_key: String,
    /// Model to use (e.g., "gpt-4", "gpt-3.5-turbo")
    pub model: String,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "gpt-3.5-turbo".to_string(),
        }
    }
}

/// OpenAI API provider
pub struct OpenAIProvider {
    client: Client,
    config: OpenAIConfig,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new(config: OpenAIConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Create provider with API key from keyring
    pub fn from_keyring(model: Option<String>) -> Result<Self, AIError> {
        let api_key = Self::load_api_key()?;
        let config = OpenAIConfig {
            api_key,
            model: model.unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
        };
        Ok(Self::new(config))
    }

    /// Load API key from system keyring
    fn load_api_key() -> Result<String, AIError> {
        use keyring::Entry;

        let entry = Entry::new("warp-foss", "openai-api-key")
            .map_err(|e| AIError::Config(format!("Failed to access keyring: {}", e)))?;

        entry
            .get_password()
            .map_err(|e| AIError::Config(format!("Failed to get API key: {}", e)))
    }

    /// Save API key to system keyring
    pub fn save_api_key(api_key: &str) -> Result<(), AIError> {
        use keyring::Entry;

        let entry = Entry::new("warp-foss", "openai-api-key")
            .map_err(|e| AIError::Config(format!("Failed to access keyring: {}", e)))?;

        entry
            .set_password(api_key)
            .map_err(|e| AIError::Config(format!("Failed to save API key: {}", e)))
    }

    /// Delete API key from system keyring
    pub fn delete_api_key() -> Result<(), AIError> {
        use keyring::Entry;

        let entry = Entry::new("warp-foss", "openai-api-key")
            .map_err(|e| AIError::Config(format!("Failed to access keyring: {}", e)))?;

        entry
            .delete_credential()
            .map_err(|e| AIError::Config(format!("Failed to delete API key: {}", e)))
    }

    /// Get the API key
    pub fn api_key(&self) -> &str {
        &self.config.api_key
    }

    /// Get the model name
    pub fn model(&self) -> &str {
        &self.config.model
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn complete(
        &self,
        prompt: &str,
        opts: Option<CompletionOptions>,
    ) -> Result<String, AIError> {
        if self.config.api_key.is_empty() {
            return Err(AIError::Config("API key not configured".to_string()));
        }

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: opts.as_ref().and_then(|o| o.max_tokens),
            temperature: opts.as_ref().and_then(|o| o.temperature),
            stream: None,
        };

        let response = self
            .client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| AIError::Api(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AIError::Api(format!(
                "API error ({}): {}",
                status, body
            )));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| AIError::Api(format!("Failed to parse response: {}", e)))?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AIError::Api("No completion returned".to_string()))
    }

    async fn stream(
        &self,
        prompt: &str,
        opts: Option<CompletionOptions>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, AIError>> + Send>>, AIError> {
        // For now, return a simple implementation that completes and returns the full result
        // A proper streaming implementation would use Server-Sent Events (SSE)
        let result = self.complete(prompt, opts).await?;

        Ok(Box::pin(futures::stream::once(async move { Ok(result) })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_config_default() {
        let config = OpenAIConfig::default();
        assert!(config.api_key.is_empty());
        assert_eq!(config.model, "gpt-3.5-turbo");
    }

    #[test]
    fn test_openai_provider_creation() {
        let config = OpenAIConfig {
            api_key: "test-key".to_string(),
            model: "gpt-4".to_string(),
        };
        let _provider = OpenAIProvider::new(config);
    }

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            max_tokens: Some(100),
            temperature: Some(0.7),
            stream: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gpt-3.5-turbo"));
        assert!(json.contains("Hello"));
        assert!(json.contains("100"));
        assert!(json.contains("0.7"));
    }

    #[test]
    fn test_chat_response_deserialization() {
        let json = r#"{
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "Hello! How can I help you?"
                    }
                }
            ]
        }"#;

        let response: ChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.choices.len(), 1);
        assert_eq!(
            response.choices[0].message.content,
            "Hello! How can I help you?"
        );
    }
}
