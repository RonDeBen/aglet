use super::{ContextHints, ModelOutput, ModelProvider};
use async_trait::async_trait;

pub struct OpenAiAdapter {
    pub api_key: String,
    pub base_url: String, // allow swapping endpoints
}

impl OpenAiAdapter {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".into(),
        }
    }
}

#[async_trait]
impl ModelProvider for OpenAiAdapter {
    async fn infer(&self, prompt: &str, _ctx: &ContextHints) -> anyhow::Result<ModelOutput> {
        // TODO: implement real request to completions or chat completions
        // this is a stub for now returning echo
        Ok(ModelOutput {
            text: format!("(openai stub) {}", prompt),
            tokens_used: None,
        })
    }

    async fn embed(&self, _text: &str) -> anyhow::Result<Vec<f32>> {
        // stub embedding
        Ok(vec![0.0f32; 16])
    }

    fn name(&self) -> &str {
        "openai"
    }
}
