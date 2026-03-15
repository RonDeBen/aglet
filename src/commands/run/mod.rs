use super::CommandContext;
use crate::error::Result;
use crate::execute::Execute;
use crate::prov::{Commit, ProvStoreJson};
use chrono::Utc;
use clap::Args;
use uuid::Uuid;

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
}

#[async_trait::async_trait]
impl Execute for RunCommand {
    async fn execute(&self, ctx: CommandContext) -> Result<()> {
        let root = ctx.project_root.path;
        let agents_dir = root.join(".aglet");
        let store = ProvStoreJson::new(agents_dir);

        // Planner commit
        let planner = Commit {
            id: Uuid::new_v4().to_string(),
            parents: vec![],
            agent_id: "planner-1".into(),
            agent_version: "0.1.0".into(),
            team_id: self
                .worktree
                .clone()
                .unwrap_or_else(|| "default-team".into()),
            timestamp: Utc::now(),
            worktree: self
                .worktree
                .clone()
                .unwrap_or_else(|| "feature/agent-run".into()),
            scope_path: ".".into(),
            prompt_snippet: format!("Plan for task: {}", self.task),
            output_ref: "blob:plan-1".into(),
            output_summary: Some(format!("Plan for task: {}", self.task)),
            evidence_refs: vec!["map:v1".into()],
            tool_calls: vec![],
            decision_tags: vec!["plan:initial".into()],
            iteration: 1,
            budget: Default::default(),
            signature: "agent/planner-1@0.1".into(),
            human_ready: false,
            proposed_branch: None,
        };
        store.write_commit(&planner)?;

        // Coder commit (stub)
        let coder = Commit {
            id: Uuid::new_v4().to_string(),
            parents: vec![planner.id.clone()],
            agent_id: "coder-1".into(),
            agent_version: "0.1.0".into(),
            team_id: planner.team_id.clone(),
            timestamp: Utc::now(),
            worktree: planner.worktree.clone(),
            scope_path: ".".into(),
            prompt_snippet: format!("Implement first changes for task: {}", self.task),
            output_ref: "blob:code-1".into(),
            output_summary: Some("Created patch with suggestions".into()),
            evidence_refs: vec![],
            tool_calls: vec![],
            decision_tags: vec!["code:proposal".into()],
            iteration: 1,
            budget: Default::default(),
            signature: "agent/coder-1@0.1".into(),
            human_ready: false,
            proposed_branch: None,
        };
        store.write_commit(&coder)?;

        log::info!("agent run: created planner and coder commits (stubbed)");
        Ok(())
    }
}
