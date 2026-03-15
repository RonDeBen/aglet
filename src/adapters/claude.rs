use super::{ContextHints, ModelOutput, ModelProvider};
use async_trait::async_trait;

pub struct ClaudeAdapter {
    pub api_key: String,
    pub base_url: String,
}

impl ClaudeAdapter {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com".into(),
        }
    }
}

#[async_trait]
impl ModelProvider for ClaudeAdapter {
    async fn infer(&self, prompt: &str, _ctx: &ContextHints) -> anyhow::Result<ModelOutput> {
        Ok(ModelOutput {
            text: format!("(claude stub) {}", prompt),
            tokens_used: None,
        })
    }

    async fn embed(&self, _text: &str) -> anyhow::Result<Vec<f32>> {
        Ok(vec![0.0f32; 16])
    }

    fn name(&self) -> &str {
        "claude"
    }
}
