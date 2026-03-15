use crate::commands::run::store::{RunManifest, RunStatus, RunStore, StepRecord, StepStatus};
use crate::commands::CommandContext;
use crate::error::{CliError, Result};
use crate::execute::Execute;
use crate::utils::fs::short_id;
use crate::utils::git;
use chrono::{DateTime, Utc};
use clap::Args;
use std::path::Path;

#[derive(Args)]
pub struct ShowCommand {
    /// Short ID, full ID, or unique prefix of a run or step
    pub id: String,
}

#[async_trait::async_trait]
impl Execute for ShowCommand {
    async fn execute(&self, ctx: CommandContext) -> Result<()> {
        let repo_root = ctx.project_root.path.clone();
        let store = RunStore::new(repo_root.join(".aglet"));
        let now = Utc::now();

        if let Some(run) = store.find_run(&self.id)? {
            show_run(&run, &store, &repo_root, now)?;
            return Ok(());
        }

        if let Some(step) = store.find_step(&self.id)? {
            show_step(&step, &store, &repo_root, now)?;
            return Ok(());
        }

        Err(CliError::WorkspaceError(format!(
            "no run or step found matching '{}'",
            self.id
        )))
    }
}

fn show_run(
    run: &RunManifest,
    store: &RunStore,
    repo_root: &Path,
    now: DateTime<Utc>,
) -> Result<()> {
    let sid = short_id(&run.id);
    let age = compact_age(run.created_at, now);
    let sym = status_symbol(&run.status);

    println!("run  {}  {}", sid, run.task);
    println!("  status   {} {}", sym, fmt_status_run(&run.status));
    println!(
        "  created  {}  ({})",
        run.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
        age
    );
    if let Some(ref p) = run.parent_run_id {
        println!("  parent   {}", short_id(p));
    }
    if let Some(ref w) = run.worktree {
        println!("  branch   aglet/{}", run.id);
        println!("  worktree {}", w);
    }
    if let Some(ref t) = run.team {
        println!("  team     {}", t);
    }

    let steps = store.list_steps_for_run(&run.id)?;
    if steps.is_empty() {
        println!("\n  (no steps recorded)");
        return Ok(());
    }

    println!("\nsteps");
    for step in &steps {
        show_step_inline(step, store, repo_root, now)?;
    }

    Ok(())
}

fn show_step_inline(
    step: &StepRecord,
    store: &RunStore,
    repo_root: &Path,
    now: DateTime<Utc>,
) -> Result<()> {
    let sid = short_id(&step.id);
    let age = compact_age(step.created_at, now);
    let sym = step_symbol(&step.status);

    println!(
        "  {} {}  {}/{}  {}  {}",
        sym, sid, step.role, step.kind, step.provider, age
    );
    if !step.summary.is_empty() {
        println!("    \"{}\"", step.summary);
    }

    print_object_section(store, "    input ", step.input_ref.as_deref())?;
    print_object_section(store, "    output", step.output_ref.as_deref())?;
    print_diff_section(
        store,
        repo_root,
        "    diff  ",
        step.diff_ref.as_deref(),
        step.checkpoint_before.as_deref(),
        step.checkpoint_after.as_deref(),
    )?;

    if let Some(t) = step.tokens_used {
        println!("    tokens  {}", t);
    }

    println!();
    Ok(())
}

fn show_step(
    step: &StepRecord,
    store: &RunStore,
    repo_root: &Path,
    now: DateTime<Utc>,
) -> Result<()> {
    let sid = short_id(&step.id);
    let age = compact_age(step.created_at, now);
    let sym = step_symbol(&step.status);

    println!(
        "step  {}  {}/{}  {}  {} {}",
        sid, step.role, step.kind, step.provider, sym, age
    );
    println!("  run  {}  ({})", short_id(&step.run_id), step.run_id);
    if !step.summary.is_empty() {
        println!("  \"{}\"", step.summary);
    }

    print_object_section(store, "input ", step.input_ref.as_deref())?;
    print_object_section(store, "output", step.output_ref.as_deref())?;
    print_diff_section(
        store,
        repo_root,
        "diff  ",
        step.diff_ref.as_deref(),
        step.checkpoint_before.as_deref(),
        step.checkpoint_after.as_deref(),
    )?;

    if let Some(t) = step.tokens_used {
        println!("tokens  {}", t);
    }

    Ok(())
}

fn print_object_section(store: &RunStore, label: &str, obj_ref: Option<&str>) -> Result<()> {
    match obj_ref {
        None => println!("{}  (none)", label),
        Some(r) => match store.read_object(r)? {
            None => println!("{}  (missing: {})", label, r),
            Some(content) => {
                println!("{}  ─────────────────────────────────────────────", label);
                for line in content.lines() {
                    println!("           {}", line);
                }
                println!("           ─────────────────────────────────────────────");
            }
        },
    }
    Ok(())
}

/// Render the diff for a step. Prefers live git diff from checkpoint SHAs;
/// falls back to a stored diff_ref object; falls back to "(none)".
fn print_diff_section(
    store: &RunStore,
    repo_root: &Path,
    label: &str,
    diff_ref: Option<&str>,
    before: Option<&str>,
    after: Option<&str>,
) -> Result<()> {
    // Prefer checkpoint-based live diff
    if let (Some(b), Some(a)) = (before, after) {
        let diff = git::diff_shas(repo_root, b, a)?;
        if diff.is_empty() {
            println!("{}  (no file changes)", label);
        } else {
            println!("{}  ─────────────────────────────────────────────", label);
            for line in diff.lines() {
                println!("           {}", line);
            }
            println!("           ─────────────────────────────────────────────");
        }
        return Ok(());
    }

    // Fall back to stored diff object
    if diff_ref.is_some() {
        return print_object_section(store, label, diff_ref);
    }

    println!("{}  (none)", label);
    Ok(())
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

fn fmt_status_run(status: &RunStatus) -> &'static str {
    match status {
        RunStatus::Planned => "planned",
        RunStatus::Running => "running",
        RunStatus::Completed => "completed",
        RunStatus::Failed => "failed",
        RunStatus::Merged => "merged",
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
