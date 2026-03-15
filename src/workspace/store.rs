use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceDocument {
    pub id: String,
    pub scope_path: String,
    pub kind: String,
    pub summary: String,
    pub artifact_ref: String,
    pub updated_at: DateTime<Utc>,
}

pub struct WorkspaceStore {
    base: PathBuf,
}

impl WorkspaceStore {
    pub fn new(base: PathBuf) -> Self {
        Self { base }
    }

    pub fn workspace_dir(&self) -> PathBuf {
        self.base.join("workspace")
    }

    pub fn objects_dir(&self) -> PathBuf {
        self.base.join("objects")
    }

    pub fn refs_dir(&self) -> PathBuf {
        self.base.join("refs")
    }

    pub fn init_layout(&self) -> Result<()> {
        fs::create_dir_all(self.workspace_dir())?;
        fs::create_dir_all(self.objects_dir())?;
        fs::create_dir_all(self.refs_dir())?;
        Ok(())
    }

    pub fn write_object(&self, name: &str, extension: &str, contents: &str) -> Result<String> {
        self.init_layout()?;
        let filename = format!("{}.{}", name, extension);
        let path = self.objects_dir().join(&filename);
        fs::write(&path, contents)?;
        Ok(format!("file:objects/{}", filename))
    }

    pub fn write_document(&self, document: &WorkspaceDocument) -> Result<()> {
        self.init_layout()?;
        let path = self.workspace_dir().join(format!("{}.toml", document.id));
        fs::write(path, toml::to_string_pretty(document)?)?;
        Ok(())
    }

    pub fn write_ref(&self, name: &str, value: &str) -> Result<()> {
        self.init_layout()?;
        fs::write(self.refs_dir().join(name), format!("{}\n", value))?;
        Ok(())
    }
}

pub fn workspace_doc_id(kind: &str, scope_path: &str, now: DateTime<Utc>) -> String {
    let scope = slugify(scope_path);
    let kind = slugify(kind);
    format!("{}-{}-{}", now.format("%Y%m%dT%H%M%SZ"), kind, scope)
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

    if slug.is_empty() { "root".into() } else { slug }
}
