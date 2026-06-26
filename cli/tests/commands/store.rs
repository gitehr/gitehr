// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Store-first bootstrap tests (ADR-0005). These exercise the library functions
//! directly and cd into a tempdir, so they are `#[serial]` (the process cwd is
//! shared across tests).

use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use gitehr::commands::store;

#[test]
#[serial]
fn store_init_bootstraps_store_mpi_and_first_subject() -> Result<()> {
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp)?;

    store::init::run(Some("rex"))?;

    // The Store: an MPI index at the root.
    assert!(
        Path::new("gitehr-mpi.json").exists(),
        "gitehr-mpi.json should exist at the Store root"
    );

    // The first subject: a fully scaffolded repo under its friendly-name dir.
    let repo = Path::new("rex");
    assert!(repo.join(".gitehr").is_dir(), ".gitehr should exist");
    assert!(
        repo.join(".gitehr/GITEHR_VERSION").exists(),
        "GITEHR_VERSION should exist"
    );
    assert!(
        repo.join(".gitehr/ID").exists(),
        "canonical id (.gitehr/ID) should exist"
    );
    assert!(
        repo.join(".gitehr/gitehr").is_file(),
        "bundled binary should exist"
    );
    assert!(repo.join("journal").is_dir(), "journal/ should exist");
    assert!(repo.join("state").is_dir(), "state/ should exist");

    // The MPI records the subject under its friendly name.
    assert!(fs::read_to_string("gitehr-mpi.json")?.contains("\"rex\""));
    Ok(())
}

#[test]
#[serial]
fn store_init_fails_if_already_a_store() -> Result<()> {
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp)?;

    store::init::run(Some("first"))?;
    assert!(
        store::init::run(Some("second")).is_err(),
        "a second `store init` in the same directory should fail"
    );
    Ok(())
}

#[test]
#[serial]
fn store_add_registers_a_second_subject() -> Result<()> {
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp)?;

    store::init::run(Some("first"))?;
    store::add::run(Some("second"), vec![])?;

    assert!(
        Path::new("second/.gitehr").is_dir(),
        "the second subject's repo should exist"
    );
    let mpi = fs::read_to_string("gitehr-mpi.json")?;
    assert!(
        mpi.contains("\"first\"") && mpi.contains("\"second\""),
        "the MPI should list both subjects"
    );
    Ok(())
}

#[test]
#[serial]
fn store_init_without_a_name_uses_an_auto_id_directory() -> Result<()> {
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp)?;

    // Non-interactive (no TTY in tests) -> no prompt -> auto-generated id.
    store::init::run(None)?;

    let subject_repos: Vec<_> = fs::read_dir(".")?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().join(".gitehr").is_dir())
        .collect();
    assert_eq!(
        subject_repos.len(),
        1,
        "exactly one auto-id subject repo should exist"
    );
    Ok(())
}
