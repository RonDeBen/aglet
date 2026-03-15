use crate::error::{CliError, Result};
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
