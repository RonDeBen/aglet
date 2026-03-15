#![allow(dead_code)]

pub mod claude;
pub mod openai;

use async_trait::async_trait;

#[derive(Debug)]
pub struct ModelOutput {
    pub text: String,
    pub tokens_used: Option<u64>,
}

#[derive(Debug)]
pub struct ContextHints {
    pub task: String,
    pub evidence_refs: Vec<String>,
}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn infer(&self, prompt: &str, ctx: &ContextHints) -> anyhow::Result<ModelOutput>;
    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>>;
    fn name(&self) -> &str;
}
