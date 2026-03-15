use crate::commands::policy::store::{PolicyMode, PolicyStore};
use crate::commands::run::store::RunStore;
use crate::commands::CommandContext;
use crate::error::Result;
use crate::execute::Execute;
use crate::workspace::store::WorkspaceStore;
use chrono::Utc;
use clap::{Args, ValueEnum};

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum PoliciesMode {
    /// All policies rendered with full text — use for orchestrator runs
    All,
    /// Only always-active policies
    Mandatory,
}

#[derive(Args)]
pub struct ContextCommand {
    /// Task the agent will be working on
    #[arg(long)]
    pub task: String,

    /// Run ID to include step history from (short ID or full ID)
    #[arg(long)]
    pub run: Option<String>,

    /// Policy inclusion mode.
    /// Default: always-active policies in full, optional policies compact (agent self-selects).
    /// --policies all: all policies in full (for orchestrators).
    /// --policies mandatory: only always-active policies.
    #[arg(long, value_enum)]
    pub policies: Option<PoliciesMode>,
}

#[async_trait::async_trait]
impl Execute for ContextCommand {
    async fn execute(&self, ctx: CommandContext) -> Result<()> {
        let root = ctx.project_root.path;
        let aglet = root.join(".aglet");
        let now = Utc::now();

        let workspace_store = WorkspaceStore::new(aglet.clone());
        let policy_store = PolicyStore::new(aglet.clone());
        let run_store = RunStore::new(aglet.clone());

        println!(
            "<!-- aglet context  task: {}  {} -->",
            self.task,
            now.format("%Y-%m-%d %H:%M UTC")
        );
        println!();

        // ── Workspace ─────────────────────────────────────────────────────────
        if let Some(obj_ref) = workspace_store.read_ref("workspace-overview")? {
            if let Some(content) = workspace_store.read_object(&obj_ref)? {
                println!("# Workspace");
                println!();
                println!("{}", content);
                println!();
            }
        }

        // ── Policies ──────────────────────────────────────────────────────────
        let all_policies = policy_store.list()?;
        let mode = self.policies;

        let always: Vec<_> = all_policies
            .iter()
            .filter(|e| e.document.mode == PolicyMode::Always)
            .collect();

        let optional: Vec<_> = all_policies
            .iter()
            .filter(|e| e.document.mode == PolicyMode::Optional)
            .collect();

        let has_always = !always.is_empty();
        let has_optional = !optional.is_empty() && mode != Some(PoliciesMode::Mandatory);

        if has_always || has_optional {
            println!("# Policies");
            println!();
        }

        if has_always {
            if has_optional {
                println!("## Always Active");
                println!();
            }
            for entry in &always {
                render_policy_full(&entry.document);
            }
        }

        if has_optional {
            match mode {
                Some(PoliciesMode::All) => {
                    println!("## Consider Applying");
                    println!();
                    println!("_Review each and apply those relevant to your task._");
                    println!();
                    for entry in &optional {
                        render_policy_full(&entry.document);
                    }
                }
                None => {
                    println!("## Consider Applying");
                    println!();
                    println!("_Review each and apply those relevant to your task._");
                    println!();
                    for entry in &optional {
                        render_policy_compact(&entry.document);
                    }
                }
                Some(PoliciesMode::Mandatory) => {}
            }
        }

        // ── Run history ───────────────────────────────────────────────────────
        if let Some(ref run_query) = self.run {
            if let Some(run) = run_store.find_run(run_query)? {
                let steps = run_store.list_steps_for_run(&run.id)?;
                if !steps.is_empty() {
                    println!("# Run History");
                    println!();
                    println!("run: {}  —  {}", run.id, run.task);
                    println!();
                    for step in &steps {
                        let diff_note = if step.checkpoint_before.is_some()
                            && step.checkpoint_after.is_some()
                        {
                            " *(has diff)*"
                        } else {
                            ""
                        };
                        println!(
                            "- **{}** {}/{}  ({}){}: {}",
                            step.created_at.format("%H:%M"),
                            step.role,
                            step.kind,
                            step.provider,
                            diff_note,
                            step.summary,
                        );
                    }
                    println!();
                }
            }
        }

        // ── Messages ──────────────────────────────────────────────────────────
        // Inbox lives at .aglet/inbox/<run-id>/ once message passing is wired up.
        if let Some(ref run_query) = self.run {
            let inbox = aglet.join("inbox").join(run_query);
            if inbox.exists() {
                println!("# Messages");
                println!();
                // TODO: render inbox messages when message passing is implemented
                println!();
            }
        }

        // ── Task ──────────────────────────────────────────────────────────────
        println!("# Task");
        println!();
        println!("{}", self.task);
        println!();

        Ok(())
    }
}

fn render_policy_full(doc: &crate::commands::policy::store::PolicyDocument) {
    println!("### {}", doc.name);
    println!("{}", doc.summary);
    println!();

    if !doc.rules.is_empty() {
        println!("**Rules**");
        for (i, rule) in doc.rules.iter().enumerate() {
            println!("{}. {}", i + 1, rule);
        }
        println!();
    }

    if !doc.examples_good.is_empty() {
        println!("**Do**");
        for ex in &doc.examples_good {
            println!("- `{}`", ex);
        }
        println!();
    }

    if !doc.examples_bad.is_empty() {
        println!("**Don't**");
        for ex in &doc.examples_bad {
            println!("- `{}`", ex);
        }
        println!();
    }

    if let Some(ref rationale) = doc.rationale {
        println!("_{}_", rationale);
        println!();
    }

    println!("---");
    println!();
}

fn render_policy_compact(doc: &crate::commands::policy::store::PolicyDocument) {
    println!("**{}** — {}", doc.name, doc.summary);
    if !doc.applies_when.is_empty() {
        println!(
            "- apply when: {}",
            doc.applies_when.join(" / ")
        );
    }
    if !doc.skip_when.is_empty() {
        println!(
            "- skip when: {}",
            doc.skip_when.join(" / ")
        );
    }
    println!();
}
