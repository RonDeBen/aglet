use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Budget {
    pub tokens_used: Option<u64>,
    pub time_ms: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Commit {
    pub id: String,
    pub parents: Vec<String>,
    pub agent_id: String,
    pub agent_version: String,
    pub team_id: String,
    pub timestamp: DateTime<Utc>,
    pub worktree: String,
    pub scope_path: String,
    pub prompt_snippet: String,
    pub output_ref: String,
    pub output_summary: Option<String>,
    pub evidence_refs: Vec<String>,
    pub tool_calls: Vec<String>,
    pub decision_tags: Vec<String>,
    pub iteration: u32,
    pub budget: Budget,
    pub signature: String,
    pub human_ready: bool,
    pub proposed_branch: Option<String>,
}

pub struct ProvStoreJson {
    pub base: PathBuf,
}

impl ProvStoreJson {
    pub fn new(base: PathBuf) -> Self {
        ProvStoreJson { base }
    }

    pub fn commits_dir(&self) -> PathBuf {
        self.base.join("commits")
    }

    pub fn write_commit(&self, commit: &Commit) -> Result<String> {
        fs::create_dir_all(self.commits_dir())?;
        let p = self.commits_dir().join(format!("{}.json", commit.id));
        let mut f = fs::File::create(&p)?;
        let s = serde_json::to_string_pretty(commit)?;
        f.write_all(s.as_bytes())?;
        Ok(commit.id.clone())
    }

    pub fn read_commit(&self, id: &str) -> Result<Commit> {
        let p = self.commits_dir().join(format!("{}.json", id));
        let s = fs::read_to_string(&p)?;
        let c: Commit = serde_json::from_str(&s)?;
        Ok(c)
    }

    pub fn list_commits(&self) -> Result<Vec<Commit>> {
        if !self.commits_dir().exists() {
            return Ok(Vec::new());
        }

        let mut out = Vec::new();
        for entry in fs::read_dir(self.commits_dir())? {
            let e = entry?;
            if e.file_type()?.is_file() {
                let s = fs::read_to_string(e.path())?;
                if let Ok(c) = serde_json::from_str::<Commit>(&s) {
                    out.push(c);
                }
            }
        }
        out.sort_by_key(|c| c.timestamp);
        Ok(out)
    }

    pub fn write_map_initial(&self, summary: &str) -> Result<String> {
        fs::create_dir_all(self.base.join("artifacts"))?;
        let artifact_id = format!("map-{}", uuid::Uuid::new_v4());
        let artifact_path = self
            .base
            .join("artifacts")
            .join(format!("{artifact_id}.md"));
        fs::write(&artifact_path, summary)?;

        let commit = Commit {
            id: uuid::Uuid::new_v4().to_string(),
            parents: vec![],
            agent_id: "mapper-1".into(),
            agent_version: env!("CARGO_PKG_VERSION").into(),
            team_id: "map-team".into(),
            timestamp: Utc::now(),
            worktree: "main".into(),
            scope_path: ".".into(),
            prompt_snippet: "map-codebase".into(),
            output_ref: format!("artifact:{artifact_id}"),
            output_summary: Some(summary.into()),
            evidence_refs: vec![],
            tool_calls: vec![],
            decision_tags: vec!["map:initial".into()],
            iteration: 1,
            budget: Default::default(),
            signature: "agent/mapper-1@init".into(),
            human_ready: false,
            proposed_branch: None,
        };
        self.write_commit(&commit)?;
        Ok(artifact_id)
    }
}
