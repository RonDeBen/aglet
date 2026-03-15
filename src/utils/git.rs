use crate::error::{CliError, Result};
use std::path::Path;
use std::process::Command;

/// Returns true if `path` is inside a git repository.
pub fn is_git_repo(path: &Path) -> bool {
    Command::new("git")
        .current_dir(path)
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Create a linked worktree at `worktree_path` on a new branch `branch`.
/// If `from_branch` is given, the new branch starts from that branch's HEAD;
/// otherwise it starts from the current HEAD of the main worktree.
pub fn create_worktree(
    repo_root: &Path,
    worktree_path: &Path,
    branch: &str,
    from_branch: Option<&str>,
) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root);
    cmd.args(["worktree", "add"]);
    cmd.arg(worktree_path);
    cmd.args(["-b", branch]);
    if let Some(from) = from_branch {
        cmd.arg(from);
    }
    let out = cmd.output()?;
    if !out.status.success() {
        return Err(CliError::WorkspaceError(format!(
            "git worktree add failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(())
}

/// Return the current HEAD SHA in `path` (a repo root or a worktree).
/// Returns None if the repo has no commits yet.
pub fn current_sha(path: &Path) -> Result<Option<String>> {
    let out = Command::new("git")
        .current_dir(path)
        .args(["rev-parse", "HEAD"])
        .output()?;
    if out.status.success() {
        Ok(Some(String::from_utf8_lossy(&out.stdout).trim().to_string()))
    } else {
        Ok(None)
    }
}

/// Stage all changes and create a commit in `worktree_path`.
/// Returns the new HEAD SHA, or None if there was nothing to commit.
pub fn commit_all(worktree_path: &Path, message: &str) -> Result<Option<String>> {
    // Stage everything
    let add = Command::new("git")
        .current_dir(worktree_path)
        .args(["add", "-A"])
        .output()?;
    if !add.status.success() {
        return Err(CliError::WorkspaceError(format!(
            "git add failed: {}",
            String::from_utf8_lossy(&add.stderr).trim()
        )));
    }

    // Check if there is actually anything staged
    let staged = Command::new("git")
        .current_dir(worktree_path)
        .args(["diff", "--cached", "--quiet"])
        .status()?;
    if staged.success() {
        // exit 0 means no diff — nothing to commit
        return Ok(None);
    }

    // Commit with aglet as the identity so it works even without user git config
    let commit = Command::new("git")
        .current_dir(worktree_path)
        .args([
            "-c", "user.name=aglet",
            "-c", "user.email=aglet@local",
            "commit", "-m", message,
        ])
        .output()?;
    if !commit.status.success() {
        return Err(CliError::WorkspaceError(format!(
            "git commit failed: {}",
            String::from_utf8_lossy(&commit.stderr).trim()
        )));
    }

    current_sha(worktree_path)
}

/// Compute the diff between two SHAs. Returns an empty string if identical.
pub fn diff_shas(repo_root: &Path, before: &str, after: &str) -> Result<String> {
    let out = Command::new("git")
        .current_dir(repo_root)
        .args(["diff", &format!("{}..{}", before, after)])
        .output()?;
    if !out.status.success() {
        return Err(CliError::WorkspaceError(format!(
            "git diff failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Merge `branch` into the current branch of `worktree_path` (no-ff).
/// This is used by an orchestrator run to absorb a completed child run.
pub fn merge_branch(worktree_path: &Path, branch: &str) -> Result<()> {
    let out = Command::new("git")
        .current_dir(worktree_path)
        .args([
            "-c", "user.name=aglet",
            "-c", "user.email=aglet@local",
            "merge", "--no-ff", branch,
            "-m", &format!("aglet: merge {}", branch),
        ])
        .output()?;
    if !out.status.success() {
        return Err(CliError::WorkspaceError(format!(
            "git merge failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(())
}

/// Remove a linked worktree (force, since the branch is preserved).
pub fn remove_worktree(repo_root: &Path, worktree_path: &Path) -> Result<()> {
    let out = Command::new("git")
        .current_dir(repo_root)
        .args([
            "worktree",
            "remove",
            "--force",
            &worktree_path.to_string_lossy(),
        ])
        .output()?;
    if !out.status.success() {
        return Err(CliError::WorkspaceError(format!(
            "git worktree remove failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(())
}
