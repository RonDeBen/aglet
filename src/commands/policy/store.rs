use crate::error::{CliError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyMode {
    Always,
    Optional,
}

impl std::fmt::Display for PolicyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Always => write!(f, "always"),
            Self::Optional => write!(f, "optional"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDocument {
    pub name: String,
    pub mode: PolicyMode,
    pub summary: String,
    #[serde(default)]
    pub applies_when: Vec<String>,
    #[serde(default)]
    pub skip_when: Vec<String>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub examples_good: Vec<String>,
    #[serde(default)]
    pub examples_bad: Vec<String>,
    #[serde(default)]
    pub rationale: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LegacyPolicyDocument {
    pub name: String,
    pub mode: PolicyMode,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub checklist: Vec<String>,
}

impl From<LegacyPolicyDocument> for PolicyDocument {
    fn from(value: LegacyPolicyDocument) -> Self {
        Self {
            name: value.name,
            mode: value.mode,
            summary: if value.description.is_empty() {
                "Describe the policy in one sentence.".into()
            } else {
                value.description
            },
            applies_when: Vec::new(),
            skip_when: Vec::new(),
            rules: value.checklist,
            examples_good: Vec::new(),
            examples_bad: Vec::new(),
            rationale: None,
        }
    }
}

pub struct PolicyStore {
    base: PathBuf,
}

impl PolicyStore {
    pub fn new(base: PathBuf) -> Self {
        Self { base }
    }

    pub fn dir(&self) -> PathBuf {
        self.base.join("policies")
    }

    pub fn ensure_dir(&self) -> Result<()> {
        fs::create_dir_all(self.dir())?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<PolicyEntry>> {
        self.ensure_dir()?;
        let mut out = Vec::new();

        for entry in fs::read_dir(self.dir())? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_file()
                && path.extension().and_then(|ext| ext.to_str()) == Some("toml")
            {
                let document = self.read_path(&path)?;
                out.push(PolicyEntry {
                    key: path
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or_default()
                        .to_string(),
                    path,
                    document,
                });
            }
        }

        out.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(out)
    }

    pub fn read(&self, key: &str) -> Result<PolicyEntry> {
        let path = self.path_for(key);
        let document = self.read_path(&path)?;
        Ok(PolicyEntry {
            key: key.to_string(),
            path,
            document,
        })
    }

    pub fn create(&self, key: &str, document: &PolicyDocument) -> Result<PathBuf> {
        self.ensure_dir()?;
        let path = self.path_for(key);
        if path.exists() {
            return Err(CliError::ConfigError(format!(
                "policy '{key}' already exists"
            )));
        }

        fs::write(&path, toml::to_string_pretty(document)?)?;
        Ok(path)
    }

    pub fn ensure_default_policies(&self) -> Result<()> {
        self.ensure_dir()?;

        let logging = self.path_for("logging");
        let logging_doc = PolicyDocument {
            name: "prefer-log-for-stdout".into(),
            mode: PolicyMode::Optional,
            summary: "Operational updates go to logs. Stdout is for command results.".into(),
            applies_when: vec![
                "changing CLI command behavior".into(),
                "adding progress or status output".into(),
            ],
            skip_when: vec![
                "printing the primary result of a command".into(),
                "emitting JSON or text intended for piping".into(),
            ],
            rules: vec![
                "Use log macros for progress, status, and diagnostics.".into(),
                "Do not use println! for operational chatter.".into(),
                "Use stdout only for command results the user may read, pipe, or parse.".into(),
            ],
            examples_good: vec!["log::info!(\"created planner commit\")".into()],
            examples_bad: vec!["println!(\"created planner commit\")".into()],
            rationale: Some(
                "This keeps quiet and verbose behavior consistent and avoids mixing machine-readable output with status noise.".into(),
            ),
        };

        fs::write(logging, toml::to_string_pretty(&logging_doc)?)?;
        Ok(())
    }

    pub fn path_for(&self, key: &str) -> PathBuf {
        self.dir().join(format!("{key}.toml"))
    }

    fn read_path(&self, path: &Path) -> Result<PolicyDocument> {
        let contents = fs::read_to_string(path)?;
        match toml::from_str(&contents) {
            Ok(document) => Ok(document),
            Err(_) => {
                let legacy: LegacyPolicyDocument = toml::from_str(&contents)?;
                Ok(legacy.into())
            }
        }
    }
}

pub struct PolicyEntry {
    pub key: String,
    pub path: PathBuf,
    pub document: PolicyDocument,
}

pub fn policy_key(input: &str) -> String {
    let mut key = String::new();
    let mut last_was_dash = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            key.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            key.push('-');
            last_was_dash = true;
        }
    }

    while key.ends_with('-') {
        key.pop();
    }

    while key.starts_with('-') {
        key.remove(0);
    }

    key
}

pub fn scaffold_policy(name: String, mode: PolicyMode) -> PolicyDocument {
    PolicyDocument {
        name,
        mode,
        summary: "Describe the policy in one sentence.".into(),
        applies_when: vec![
            "Describe when this policy should be considered.".into(),
            "Use concrete triggers rather than broad intent.".into(),
        ],
        skip_when: vec![
            "Describe when this policy does not apply.".into(),
            "Use explicit exclusions to avoid over-applying it.".into(),
        ],
        rules: vec![
            "Write short, testable rules.".into(),
            "Prefer direct instructions over long explanations.".into(),
        ],
        examples_good: vec!["Add one example of compliant behavior.".into()],
        examples_bad: vec!["Add one example of behavior to avoid.".into()],
        rationale: Some("Explain why this policy exists in one sentence.".into()),
    }
}
