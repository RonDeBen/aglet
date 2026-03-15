use crate::commands::run::store::{RunStatus, RunStore};
use crate::commands::CommandContext;
use crate::error::{CliError, Result};
use crate::execute::Execute;
use crate::utils::git;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct MergeCommand {
    /// Short ID, full ID, or unique prefix of the run to merge
    pub id: String,
}

#[async_trait::async_trait]
impl Execute for MergeCommand {
    async fn execute(&self, ctx: CommandContext) -> Result<()> {
        let repo_root = ctx.project_root.path.clone();
        let store = RunStore::new(repo_root.join(".aglet"));

        let mut manifest = store
            .find_run(&self.id)?
            .ok_or_else(|| CliError::WorkspaceError(format!("no run found matching '{}'", self.id)))?;

        match manifest.status {
            RunStatus::Merged => {
                return Err(CliError::WorkspaceError(format!(
                    "run '{}' has already been merged",
                    self.id
                )))
            }
            RunStatus::Running => {
                return Err(CliError::WorkspaceError(format!(
                    "run '{}' is still running — wait for it to complete before merging",
                    self.id
                )))
            }
            RunStatus::Planned => {
                return Err(CliError::WorkspaceError(format!(
                    "run '{}' has not started yet",
                    self.id
                )))
            }
            RunStatus::Completed | RunStatus::Failed => {}
        }

        let branch = format!("aglet/{}", manifest.id);

        git::merge_branch(&repo_root, &branch)?;
        log::debug!("merged branch {} into current HEAD", branch);

        if let Some(ref wt) = manifest.worktree {
            let wt_path = PathBuf::from(wt);
            git::remove_worktree(&repo_root, &wt_path)?;
            log::debug!("removed worktree {}", wt_path.display());
        }

        manifest.status = RunStatus::Merged;
        manifest.worktree = None;
        store.update_manifest(&manifest)?;

        println!("merged  {}  {}", manifest.id, manifest.task);
        Ok(())
    }
}
