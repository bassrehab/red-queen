//! Target adapters for LLM APIs.

use async_trait::async_trait;
use thiserror::Error;

/// Error type for target operations.
#[derive(Error, Debug)]
pub enum TargetError {
    #[error("API error: {0}")]
    Api(String),
    #[error("Rate limit exceeded")]
    RateLimit,
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}

/// LLM target trait.
#[async_trait]
pub trait LLMTarget: Send + Sync {
    /// Query the target with a prompt.
    async fn query(&self, prompt: &str) -> Result<String, TargetError>;

    /// Get the target name.
    fn name(&self) -> &str;
}

/// OpenAI API target.
pub struct OpenAITarget {
    client: reqwest::Client,
    model: String,
    api_key: String,
}

impl OpenAITarget {
    /// Create a new OpenAI target.
    pub fn new(model: &str, api_key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            model: model.to_string(),
            api_key: api_key.to_string(),
        }
    }

    /// Create from environment variable.
    pub fn from_env(model: &str) -> Result<Self, TargetError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| TargetError::Api("OPENAI_API_KEY not set".to_string()))?;
        Ok(Self::new(model, &api_key))
    }
}

#[async_trait]
impl LLMTarget for OpenAITarget {
    async fn query(&self, prompt: &str) -> Result<String, TargetError> {
        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": self.model,
                "messages": [
                    {"role": "user", "content": prompt}
                ],
                "max_tokens": 1000
            }))
            .send()
            .await?;

        if response.status() == 429 {
            return Err(TargetError::RateLimit);
        }

        let body: serde_json::Value = response.json().await?;

        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    fn name(&self) -> &str {
        &self.model
    }
}

/// Anthropic API target.
pub struct AnthropicTarget {
    client: reqwest::Client,
    model: String,
    api_key: String,
}

impl AnthropicTarget {
    /// Create a new Anthropic target.
    pub fn new(model: &str, api_key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            model: model.to_string(),
            api_key: api_key.to_string(),
        }
    }

    /// Create from environment variable.
    pub fn from_env(model: &str) -> Result<Self, TargetError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| TargetError::Api("ANTHROPIC_API_KEY not set".to_string()))?;
        Ok(Self::new(model, &api_key))
    }
}

#[async_trait]
impl LLMTarget for AnthropicTarget {
    async fn query(&self, prompt: &str) -> Result<String, TargetError> {
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&serde_json::json!({
                "model": self.model,
                "max_tokens": 1024,
                "messages": [
                    {"role": "user", "content": prompt}
                ]
            }))
            .send()
            .await?;

        if response.status() == 429 {
            return Err(TargetError::RateLimit);
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TargetError::Api(format!(
                "Anthropic API error {}: {}",
                status, body
            )));
        }

        let body: serde_json::Value = response.json().await?;

        // Anthropic returns content as an array of blocks
        let content = body["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|block| block["text"].as_str())
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    fn name(&self) -> &str {
        &self.model
    }
}

/// Google Gemini API target.
pub struct GeminiTarget {
    client: reqwest::Client,
    model: String,
    api_key: String,
}

impl GeminiTarget {
    /// Create a new Gemini target.
    pub fn new(model: &str, api_key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            model: model.to_string(),
            api_key: api_key.to_string(),
        }
    }

    /// Create from environment variable.
    pub fn from_env(model: &str) -> Result<Self, TargetError> {
        let api_key = std::env::var("GOOGLE_API_KEY")
            .map_err(|_| TargetError::Api("GOOGLE_API_KEY not set".to_string()))?;
        Ok(Self::new(model, &api_key))
    }
}

#[async_trait]
impl LLMTarget for GeminiTarget {
    async fn query(&self, prompt: &str) -> Result<String, TargetError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .header("content-type", "application/json")
            .json(&serde_json::json!({
                "contents": [
                    {
                        "parts": [
                            {"text": prompt}
                        ]
                    }
                ],
                "generationConfig": {
                    "maxOutputTokens": 1024
                }
            }))
            .send()
            .await?;

        if response.status() == 429 {
            return Err(TargetError::RateLimit);
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TargetError::Api(format!(
                "Gemini API error {}: {}",
                status, body
            )));
        }

        let body: serde_json::Value = response.json().await?;

        // Gemini returns candidates[0].content.parts[0].text
        let content = body["candidates"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|candidate| candidate["content"]["parts"].as_array())
            .and_then(|parts| parts.first())
            .and_then(|part| part["text"].as_str())
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    fn name(&self) -> &str {
        &self.model
    }
}

/// Mock target for testing.
pub struct MockTarget {
    responses: Vec<String>,
    index: std::sync::atomic::AtomicUsize,
}

impl MockTarget {
    pub fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub fn always_refuse() -> Self {
        Self::new(vec!["I cannot help with that request.".to_string()])
    }

    pub fn always_comply() -> Self {
        Self::new(vec!["Sure, here's the information...".to_string()])
    }
}

#[async_trait]
impl LLMTarget for MockTarget {
    async fn query(&self, _prompt: &str) -> Result<String, TargetError> {
        let idx = self
            .index
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            % self.responses.len();
        Ok(self.responses[idx].clone())
    }

    fn name(&self) -> &str {
        "mock"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_target_always_refuse() {
        let target = MockTarget::always_refuse();
        let response = target.query("test prompt").await.unwrap();
        assert!(response.contains("cannot"));
    }

    #[tokio::test]
    async fn test_mock_target_always_comply() {
        let target = MockTarget::always_comply();
        let response = target.query("test prompt").await.unwrap();
        assert!(response.contains("Sure"));
    }

    #[tokio::test]
    async fn test_mock_target_cycles_responses() {
        let target = MockTarget::new(vec!["first".to_string(), "second".to_string()]);

        let r1 = target.query("test").await.unwrap();
        let r2 = target.query("test").await.unwrap();
        let r3 = target.query("test").await.unwrap();

        assert_eq!(r1, "first");
        assert_eq!(r2, "second");
        assert_eq!(r3, "first"); // cycles back
    }

    #[tokio::test]
    async fn test_mock_target_name() {
        let target = MockTarget::always_refuse();
        assert_eq!(target.name(), "mock");
    }

    #[test]
    fn test_openai_target_new() {
        let target = OpenAITarget::new("gpt-4", "test-key");
        assert_eq!(target.name(), "gpt-4");
    }

    #[test]
    fn test_anthropic_target_new() {
        let target = AnthropicTarget::new("claude-sonnet-4-20250514", "test-key");
        assert_eq!(target.name(), "claude-sonnet-4-20250514");
    }

    #[test]
    fn test_gemini_target_new() {
        let target = GeminiTarget::new("gemini-2.0-flash", "test-key");
        assert_eq!(target.name(), "gemini-2.0-flash");
    }

    #[test]
    fn test_openai_from_env_missing_key() {
        // Ensure the env var is not set for this test
        std::env::remove_var("OPENAI_API_KEY");
        let result = OpenAITarget::from_env("gpt-4");
        assert!(result.is_err());
    }

    #[test]
    fn test_anthropic_from_env_missing_key() {
        std::env::remove_var("ANTHROPIC_API_KEY");
        let result = AnthropicTarget::from_env("claude-sonnet-4-20250514");
        assert!(result.is_err());
    }

    #[test]
    fn test_gemini_from_env_missing_key() {
        std::env::remove_var("GOOGLE_API_KEY");
        let result = GeminiTarget::from_env("gemini-2.0-flash");
        assert!(result.is_err());
    }

    #[test]
    fn test_target_error_display() {
        let err = TargetError::Api("test error".to_string());
        assert!(err.to_string().contains("test error"));

        let err = TargetError::RateLimit;
        assert!(err.to_string().contains("Rate limit"));
    }
}
