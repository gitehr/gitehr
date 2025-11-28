use crate::error::{RepoError, Result};
use git2::Repository;

/// Ensure we are operating inside a Git repository, initializing one if needed.
pub fn ensure_repo() -> Result<Repository> {
    match Repository::discover(".") {
        Ok(repo) => Ok(repo),
        Err(_) => Repository::init(".").map_err(|err| RepoError::Init(err).into()),
    }
}

/// Locate the repository without creating it.
pub fn open_repo() -> Result<Repository> {
    Repository::discover(".").map_err(|err| RepoError::Discover(err).into())
}
