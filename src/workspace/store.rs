use crate::error::Result;
use std::fs;
use std::path::PathBuf;

pub struct WorkspaceStore {
    base: PathBuf,
}

impl WorkspaceStore {
    pub fn new(base: PathBuf) -> Self {
        Self { base }
    }

    pub fn objects_dir(&self) -> PathBuf {
        self.base.join("objects")
    }

    pub fn refs_dir(&self) -> PathBuf {
        self.base.join("refs")
    }

    pub fn init_layout(&self) -> Result<()> {
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

    pub fn write_ref(&self, name: &str, value: &str) -> Result<()> {
        self.init_layout()?;
        fs::write(self.refs_dir().join(name), format!("{}\n", value))?;
        Ok(())
    }

    /// Read a named ref and return its value, or None if the ref doesn't exist.
    pub fn read_ref(&self, name: &str) -> Result<Option<String>> {
        let path = self.refs_dir().join(name);
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(fs::read_to_string(path)?.trim().to_string()))
    }

    /// Resolve a `file:objects/...` ref to its contents, or None if missing.
    pub fn read_object(&self, obj_ref: &str) -> Result<Option<String>> {
        let rel = obj_ref.strip_prefix("file:").unwrap_or(obj_ref);
        let path = self.base.join(rel);
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(fs::read_to_string(path)?))
    }
}

