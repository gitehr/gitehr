use crate::error::{FsError, Result};
use std::fs;
use std::path::{Path, PathBuf};

// Proposed default root directory for the GitEHR repository subjec to change
const DEFAULT_ROOT: &str = "gitehr_fs";

#[derive(Debug, Clone)]
pub struct FsLayout {
    root: PathBuf, // Maybe string instead of PathBuf?
}

impl FsLayout {
    pub fn new() -> Self {
        Self::from_root(DEFAULT_ROOT)
    }

    pub fn from_root<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn gitehr_dir(&self) -> PathBuf {
        self.root.join(".gitehr")
    }

    pub fn config_path(&self) -> PathBuf {
        self.gitehr_dir().join("config.toml")
    }

    pub fn journal_dir(&self) -> PathBuf {
        self.root.join("journal")
    }

    pub fn journal_patients_dir(&self) -> PathBuf {
        self.journal_dir().join("patients")
    }

    pub fn patient_journal_dir(&self, patient_id: &str) -> PathBuf {
        self.journal_patients_dir().join(patient_id)
    }

    pub fn imaging_dir(&self) -> PathBuf {
        self.root.join("imaging")
    }

    pub fn state_dir(&self) -> PathBuf {
        self.root.join("state")
    }

    pub fn ensure_structure(&self) -> Result<()> {
        if !self.root.exists() {
            fs::create_dir_all(&self.root).map_err(|source| FsError::CreateDir {
                path: self.root.clone(),
                source,
            })?;
        }

        for dir in [
            self.gitehr_dir(),
            self.journal_dir(),
            self.journal_patients_dir(),
            self.imaging_dir(),
            self.state_dir(),
        ] {
            if !dir.exists() {
                fs::create_dir_all(&dir).map_err(|source| FsError::CreateDir {
                    path: dir.clone(),
                    source,
                })?;
            }
        }

        Ok(())
    }
}
