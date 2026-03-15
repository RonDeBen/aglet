pub mod store;

use self::store::{RunManifest, RunStatus, RunStore, StepRecord, StepStatus};
use crate::utils::fs::{slugify, sortable_timestamp};
use crate::utils::git;
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

    /// Parent run ID (orchestrator spawning this as a child run)
    #[arg(long)]
    pub parent_run: Option<String>,

    /// Optional team YAML (not used in MVP)
    #[arg(long)]
    pub team: Option<String>,

    /// Provider used for this run
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

        // ── Worktree setup ────────────────────────────────────────────────────
        // Each run gets an isolated branch `aglet/<run-id>` and a linked
        // worktree at `.aglet/worktrees/<run-id>/`. Child runs fork from their
        // parent's branch so they inherit its state.
        let worktree_path = root.join(".aglet").join("worktrees").join(&run_id);
        let branch = format!("aglet/{}", run_id);

        let use_worktree = git::is_git_repo(&root);
        if use_worktree {
            let from = self
                .parent_run
                .as_deref()
                .map(|pid| format!("aglet/{}", pid));
            git::create_worktree(&root, &worktree_path, &branch, from.as_deref())?;
            log::debug!("created worktree {} on branch {}", worktree_path.display(), branch);
        }

        // ── Capture before-SHA ────────────────────────────────────────────────
        let checkpoint_before = if use_worktree {
            git::current_sha(&worktree_path)?
        } else {
            None
        };

        // ── Run the step ──────────────────────────────────────────────────────
        let context = ContextHints {
            task: self.task.clone(),
            evidence_refs: vec![],
        };
        let prompt = format!(
            "Task: {}\n\nProduce a concise plan and initial implementation notes.",
            self.task
        );
        let output = provider.infer(&prompt, &context).await?;

        // Write provider output as a file in the worktree so the diff is visible.
        // Real agents will write whatever files they need — this is the stub stand-in.
        if use_worktree {
            let out_file = worktree_path.join("AGENT_OUTPUT.md");
            std::fs::write(
                &out_file,
                format!("# {}\n\n{}\n", self.task, output.text),
            )?;
        }

        // ── Commit + capture after-SHA ────────────────────────────────────────
        let checkpoint_after = if use_worktree {
            git::commit_all(
                &worktree_path,
                &format!("aglet: step {} — {}", step_id, self.task),
            )?
        } else {
            None
        };

        // ── Persist objects + records ─────────────────────────────────────────
        let input_ref = store.write_object(&format!("{}-input", step_id), "md", &prompt)?;
        let output_ref =
            store.write_object(&format!("{}-output", step_id), "md", &output.text)?;

        let step = StepRecord {
            id: step_id.clone(),
            run_id: run_id.clone(),
            parent_step_ids: vec![],
            created_at: now,
            status: StepStatus::Completed,
            provider: provider_key.into(),
            role: "orchestrator".into(),
            kind: "planning".into(),
            labels: vec!["initial".into()],
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
            checkpoint_before,
            checkpoint_after,
        };
        store.write_step(&step)?;

        let manifest = RunManifest {
            id: run_id.clone(),
            task: self.task.clone(),
            created_at: now,
            status: RunStatus::Completed,
            parent_run_id: self.parent_run.clone(),
            team: self.team.clone(),
            worktree: if use_worktree {
                Some(worktree_path.to_string_lossy().to_string())
            } else {
                None
            },
            root_step_id: step_id.clone(),
            head_step_id: step_id.clone(),
        };
        store.write_manifest(&manifest)?;

        log::info!(
            "run {} created  branch: {}  provider: {}",
            run_id,
            branch,
            provider_key
        );
        Ok(())
    }
}
