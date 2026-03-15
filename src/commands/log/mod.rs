use crate::commands::run::store::{RunManifest, RunStatus, RunStore, StepRecord, StepStatus};
use crate::commands::CommandContext;
use crate::error::Result;
use crate::execute::Execute;
use crate::utils::fs::short_id;
use chrono::{DateTime, Utc};
use clap::Args;
use std::collections::HashMap;

#[derive(Args)]
pub struct LogCommand {
    /// Show only runs, without listing individual steps
    #[arg(long, short)]
    pub short: bool,
}

#[async_trait::async_trait]
impl Execute for LogCommand {
    async fn execute(&self, ctx: CommandContext) -> Result<()> {
        let store = RunStore::new(ctx.project_root.path.join(".aglet"));
        let runs = store.list_all_runs()?;

        if runs.is_empty() {
            println!("No runs recorded yet. Use `aglet run --task <task>` to create one.");
            return Ok(());
        }

        let now = Utc::now();

        let mut children: HashMap<String, Vec<&RunManifest>> = HashMap::new();
        let mut roots: Vec<&RunManifest> = Vec::new();

        for run in &runs {
            if let Some(ref pid) = run.parent_run_id {
                children.entry(pid.clone()).or_default().push(run);
            } else {
                roots.push(run);
            }
        }

        // Children in execution order (oldest first)
        for kids in children.values_mut() {
            kids.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        }

        // Top-level runs newest first
        roots.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        for root in &roots {
            render_run(root, &store, &children, self.short, now, "")?;
        }

        Ok(())
    }
}

/// Render one run node and recurse into children.
///
/// `lane` is the graph prefix for the current depth, e.g. "" at top level,
/// "| " at depth 1, "| | " at depth 2. Every line printed starts with `lane`.
fn render_run(
    run: &RunManifest,
    store: &RunStore,
    children: &HashMap<String, Vec<&RunManifest>>,
    short: bool,
    now: DateTime<Utc>,
    lane: &str,
) -> Result<()> {
    let rel = compact_age(run.created_at, now);
    let sym = status_symbol(&run.status);
    let sid = short_id(&run.id);

    println!("{}* {}  {}  {}  {}", lane, run.task, sym, rel, sid);

    if !short {
        let steps = store.list_steps_for_run(&run.id)?;
        for step in &steps {
            render_step(step, lane);
        }
    }

    let kids = children.get(&run.id).map(|v| v.as_slice()).unwrap_or(&[]);
    if !kids.is_empty() {
        println!("{}|\\", lane);
        let child_lane = format!("{}| ", lane);
        for kid in kids {
            render_run(kid, store, children, short, now, &child_lane)?;
        }
        println!("{}|/", lane);
    }

    Ok(())
}

fn render_step(step: &StepRecord, lane: &str) {
    let sym = step_symbol(&step.status);
    let summary = truncate(&step.summary, 72);
    println!(
        "{}  {} {}/{} · {}  \"{}\"",
        lane, sym, step.role, step.kind, step.provider, summary
    );
}

fn compact_age(dt: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let secs = (now - dt).num_seconds().max(0);
    if secs < 60 {
        return "just now".into();
    }
    if secs < 3600 {
        return format!("{}m ago", secs / 60);
    }
    if secs < 86400 {
        return format!("{}h ago", secs / 3600);
    }
    if secs < 7 * 86400 {
        return format!("{}d ago", secs / 86400);
    }
    // Older than a week: show the date
    dt.format("%b %d").to_string()
}

fn status_symbol(status: &RunStatus) -> &'static str {
    match status {
        RunStatus::Planned => "·",
        RunStatus::Running => "⋯",
        RunStatus::Completed => "✓",
        RunStatus::Failed => "✗",
        RunStatus::Merged => "⇒",
    }
}

fn step_symbol(status: &StepStatus) -> &'static str {
    match status {
        StepStatus::Planned => "·",
        StepStatus::Running => "⋯",
        StepStatus::Completed => "◦",
        StepStatus::Failed => "✗",
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max - 1).collect::<String>())
    }
}
