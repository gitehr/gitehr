use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, GitehrError>;

#[derive(Debug, Error)]
pub enum GitehrError {
    #[error(transparent)]
    Repo(#[from] RepoError),
    #[error(transparent)]
    Fs(#[from] FsError),
    #[error(transparent)]
    Journal(#[from] JournalError),
    #[error(transparent)]
    Verify(#[from] VerifyError),
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("failed to discover git repository")]
    Discover(#[source] git2::Error),
    #[error("failed to initialize git repository")]
    Init(#[source] git2::Error),
}

#[derive(Debug, Error)]
pub enum FsError {
    #[error("failed to create directory {path}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Error)]
pub enum JournalError {
    #[error("patient_id is required for journal entries")]
    MissingPatientId,
    #[error("author id and name are required")]
    MissingAuthor,
    #[error("failed to create patient directory {path}")]
    CreatePatientDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write journal entry at {path}")]
    WriteEntry {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read journal entry at {path}")]
    ReadEntry {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid journal header")]
    InvalidHeader(#[source] serde_yaml::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("journal directory not found")]
    MissingJournal,
    #[error("invalid entry {entry_file}: {reason}")]
    InvalidEntry { entry_file: String, reason: String },
    #[error("patient mismatch in {entry_file}: expected {expected}, found {found}")]
    PatientMismatch {
        entry_file: String,
        expected: String,
        found: String,
    },
    #[error("correction target '{correction_of}' referenced in {entry_file} does not exist")]
    MissingCorrectionTarget {
        entry_file: String,
        correction_of: String,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_yaml::Error),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config at {path}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write config at {path}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to create directory {path} for config")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
