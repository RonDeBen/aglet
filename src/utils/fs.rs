use crate::error::{CliError, Result};
use chrono::{DateTime, Utc};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ProjectRoot {
    pub path: PathBuf,
}

impl ProjectRoot {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn discover() -> Result<Self> {
        let mut current = std::env::current_dir()?;
        loop {
            if current.join(".aglet").exists() || current.join(".git").exists() {
                return Ok(Self::new(current));
            }

            if !current.pop() {
                return Err(CliError::WorkspaceError(
                    "could not discover project root from current directory".into(),
                ));
            }
        }
    }
}

pub fn sortable_timestamp(now: DateTime<Utc>) -> String {
    now.format("%Y%m%dT%H%M%SZ").to_string()
}

/// Deterministic 8-char hex short ID from any string (FNV-1a 64-bit).
/// Stable across runs — same input always produces the same output.
pub fn short_id(full_id: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in full_id.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)[..8].to_string()
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
