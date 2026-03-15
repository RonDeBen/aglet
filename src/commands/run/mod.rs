mod store;

use self::store::{
    RunManifest, RunStatus, RunStore, StepRecord, StepStatus, slugify, sortable_timestamp,
};
use super::CommandContext;
use crate::adapters::claude::ClaudeAdapter;
use crate::adapters::openai::OpenAiAdapter;
use crate::adapters::{ContextHints, ModelProvider};
use crate::error::Result;
use crate::execute::Execute;
use chrono::Utc;
use clap::{Args, ValueEnum};

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum ProviderArg {
    Codex,
    Claude,
}

#[derive(Args)]
pub struct RunCommand {
    /// Task description
    #[arg(long)]
    pub task: String,

    /// Optional team YAML (not used in MVP)
    #[arg(long)]
    pub team: Option<String>,

    /// Optional worktree/branch name to bind
    #[arg(long)]
    pub worktree: Option<String>,

    /// Provider used for this run scaffold
    #[arg(long, value_enum, default_value_t = ProviderArg::Codex)]
    pub provider: ProviderArg,
}

#[async_trait::async_trait]
impl Execute for RunCommand {
    async fn execute(&self, ctx: CommandContext) -> Result<()> {
        let root = ctx.project_root.path;
        let store = RunStore::new(root.join(".aglet"));
        let now = Utc::now();
        let provider_key = match self.provider {
            ProviderArg::Codex => "codex",
            ProviderArg::Claude => "claude",
        };
        let provider: Box<dyn ModelProvider> = match self.provider {
            ProviderArg::Codex => Box::new(OpenAiAdapter::new(
                std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            )),
            ProviderArg::Claude => Box::new(ClaudeAdapter::new(
                std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
            )),
        };
        let timestamp = sortable_timestamp(now);
        let run_id = format!("{}-{}", timestamp, slugify(&self.task));
        let step_id = format!("{}-orchestrator-{}-001", timestamp, provider_key);
        let context = ContextHints {
            task: self.task.clone(),
            evidence_refs: vec![],
        };
        let prompt = format!(
            "Task: {}\n\nProduce a concise plan and initial implementation notes.",
            self.task
        );
        let output = provider.infer(&prompt, &context).await?;
        let input_ref = store.write_object(&format!("{}-input", step_id), "md", &prompt)?;
        let output_ref = store.write_object(&format!("{}-output", step_id), "md", &output.text)?;

        let step = StepRecord {
            id: step_id.clone(),
            run_id: run_id.clone(),
            parent_step_ids: vec![],
            created_at: now,
            status: StepStatus::Completed,
            provider: provider_key.into(),
            role: "orchestrator".into(),
            kind: "planning".into(),
            labels: vec!["initial".into(), "stubbed-execution".into()],
            summary: format!(
                "Initial {} planning step for task '{}'.",
                provider_key, self.task
            ),
            task_fragment: self.task.clone(),
            policy_refs: vec![],
            input_ref: Some(input_ref),
            output_ref: Some(output_ref),
            diff_ref: None,
            tokens_used: output.tokens_used,
        };
        store.write_step(&step)?;

        let manifest = RunManifest {
            id: run_id.clone(),
            task: self.task.clone(),
            created_at: now,
            status: RunStatus::Completed,
            team: self.team.clone(),
            worktree: self.worktree.clone(),
            root_step_id: step_id.clone(),
            head_step_id: step_id.clone(),
        };
        store.write_manifest(&manifest)?;

        log::info!(
            "agent run: created run {} with provider {}",
            run_id,
            provider_key
        );
        Ok(())
    }
}
