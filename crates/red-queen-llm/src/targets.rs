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
