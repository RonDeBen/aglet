use crate::error::{CliError, Result};
use crate::utils::fs::short_id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunStatus {
    Planned,
    Running,
    Completed,
    Failed,
    Merged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StepStatus {
    Planned,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunManifest {
    pub id: String,
    pub task: String,
    pub created_at: DateTime<Utc>,
    pub status: RunStatus,
    pub parent_run_id: Option<String>,
    pub team: Option<String>,
    pub worktree: Option<String>,
    pub root_step_id: String,
    pub head_step_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRecord {
    pub id: String,
    pub run_id: String,
    pub parent_step_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub status: StepStatus,
    pub provider: String,
    pub role: String,
    pub kind: String,
    pub labels: Vec<String>,
    pub summary: String,
    pub task_fragment: String,
    pub policy_refs: Vec<String>,
    pub input_ref: Option<String>,
    pub output_ref: Option<String>,
    pub diff_ref: Option<String>,
    pub tokens_used: Option<u64>,
    /// Git SHA of the worktree HEAD before this step ran
    #[serde(default)]
    pub checkpoint_before: Option<String>,
    /// Git SHA of the worktree HEAD after this step committed its changes
    #[serde(default)]
    pub checkpoint_after: Option<String>,
}

pub struct RunStore {
    base: PathBuf,
}

impl RunStore {
    pub fn new(base: PathBuf) -> Self {
        Self { base }
    }

    pub fn runs_dir(&self) -> PathBuf {
        self.base.join("runs")
    }

    pub fn steps_dir(&self) -> PathBuf {
        self.base.join("steps")
    }

    pub fn objects_dir(&self) -> PathBuf {
        self.base.join("objects")
    }

    pub fn init_layout(&self) -> Result<()> {
        fs::create_dir_all(self.runs_dir())?;
        fs::create_dir_all(self.steps_dir())?;
        fs::create_dir_all(self.objects_dir())?;
        fs::create_dir_all(self.base.join("refs"))?;
        Ok(())
    }

    /// Overwrite only the manifest.toml and summary.md for an existing run
    /// without touching any refs. Used when updating status after the run is
    /// already recorded (e.g. marking it Merged).
    pub fn update_manifest(&self, manifest: &RunManifest) -> Result<()> {
        let run_dir = self.runs_dir().join(&manifest.id);
        fs::write(
            run_dir.join("manifest.toml"),
            toml::to_string_pretty(manifest)?,
        )?;
        fs::write(
            run_dir.join("summary.md"),
            format!(
                "# Run {}\n\nTask: {}\n\nStatus: {:?}\n\nRoot step: {}\nHead step: {}\n",
                manifest.id,
                manifest.task,
                manifest.status,
                manifest.root_step_id,
                manifest.head_step_id
            ),
        )?;
        Ok(())
    }

    pub fn write_manifest(&self, manifest: &RunManifest) -> Result<()> {
        self.init_layout()?;
        let run_dir = self.runs_dir().join(&manifest.id);
        fs::create_dir_all(&run_dir)?;
        fs::write(
            run_dir.join("manifest.toml"),
            toml::to_string_pretty(manifest)?,
        )?;
        fs::write(
            run_dir.join("summary.md"),
            format!(
                "# Run {}\n\nTask: {}\n\nStatus: {:?}\n\nRoot step: {}\nHead step: {}\n",
                manifest.id,
                manifest.task,
                manifest.status,
                manifest.root_step_id,
                manifest.head_step_id
            ),
        )?;
        fs::write(
            self.base.join("refs").join("latest-run"),
            format!("{}\n", manifest.id),
        )?;
        fs::write(
            run_dir.join("head-step"),
            format!("{}\n", manifest.head_step_id),
        )?;
        Ok(())
    }

    pub fn write_step(&self, step: &StepRecord) -> Result<()> {
        self.init_layout()?;
        let path = self.steps_dir().join(format!("{}.toml", step.id));
        fs::write(path, toml::to_string_pretty(step)?)?;
        fs::write(
            self.base.join("refs").join("latest-step"),
            format!("{}\n", step.id),
        )?;
        Ok(())
    }

    pub fn write_object(&self, name: &str, extension: &str, contents: &str) -> Result<String> {
        self.init_layout()?;
        let filename = format!("{}.{}", name, extension);
        let path = self.objects_dir().join(&filename);
        fs::write(&path, contents)?;
        Ok(format!("file:objects/{}", filename))
    }

    pub fn list_all_runs(&self) -> Result<Vec<RunManifest>> {
        let dir = self.runs_dir();
        if !dir.exists() {
            return Ok(vec![]);
        }
        let mut runs = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let manifest_path = entry.path().join("manifest.toml");
            if manifest_path.exists() {
                let raw = fs::read_to_string(&manifest_path)?;
                let manifest: RunManifest = toml::from_str(&raw)?;
                runs.push(manifest);
            }
        }
        runs.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(runs)
    }

    pub fn list_steps_for_run(&self, run_id: &str) -> Result<Vec<StepRecord>> {
        Ok(self
            .list_all_steps()?
            .into_iter()
            .filter(|s| s.run_id == run_id)
            .collect())
    }

    pub fn list_all_steps(&self) -> Result<Vec<StepRecord>> {
        let dir = self.steps_dir();
        if !dir.exists() {
            return Ok(vec![]);
        }
        let mut steps = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                let raw = fs::read_to_string(&path)?;
                let step: StepRecord = toml::from_str(&raw)?;
                steps.push(step);
            }
        }
        steps.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(steps)
    }

    /// Find a run by full ID, short ID (8-char hex), or unique prefix.
    pub fn find_run(&self, query: &str) -> Result<Option<RunManifest>> {
        let runs = self.list_all_runs()?;
        // Exact match
        if let Some(r) = runs.iter().find(|r| r.id == query) {
            return Ok(Some(r.clone()));
        }
        // Short ID or prefix match
        let matches: Vec<_> = runs
            .iter()
            .filter(|r| short_id(&r.id) == query || r.id.starts_with(query))
            .collect();
        match matches.len() {
            0 => Ok(None),
            1 => Ok(Some(matches[0].clone())),
            _ => Err(CliError::WorkspaceError(format!(
                "ambiguous id '{}' matches {} runs",
                query,
                matches.len()
            ))),
        }
    }

    /// Find a step by full ID, short ID (8-char hex), or unique prefix.
    pub fn find_step(&self, query: &str) -> Result<Option<StepRecord>> {
        let steps = self.list_all_steps()?;
        if let Some(s) = steps.iter().find(|s| s.id == query) {
            return Ok(Some(s.clone()));
        }
        let matches: Vec<_> = steps
            .iter()
            .filter(|s| short_id(&s.id) == query || s.id.starts_with(query))
            .collect();
        match matches.len() {
            0 => Ok(None),
            1 => Ok(Some(matches[0].clone())),
            _ => Err(CliError::WorkspaceError(format!(
                "ambiguous id '{}' matches {} steps",
                query,
                matches.len()
            ))),
        }
    }

    /// Read the contents of a stored object by its ref (e.g. "file:objects/foo.md").
    pub fn read_object(&self, obj_ref: &str) -> Result<Option<String>> {
        let rel = obj_ref.strip_prefix("file:").unwrap_or(obj_ref);
        let path = self.base.join(rel);
        if path.exists() {
            Ok(Some(fs::read_to_string(path)?))
        } else {
            Ok(None)
        }
    }
}
