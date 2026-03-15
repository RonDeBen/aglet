use crate::error::Result;
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
}

pub fn sortable_timestamp(now: DateTime<Utc>) -> String {
    now.format("%Y%m%dT%H%M%SZ").to_string()
}

pub fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    while slug.starts_with('-') {
        slug.remove(0);
    }

    if slug.is_empty() { "task".into() } else { slug }
}
