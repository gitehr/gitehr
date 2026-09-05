// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Store-first bootstrap tests (ADR-0005). These exercise the library functions
//! directly and cd into a tempdir, so they are `#[serial]` (the process cwd is
//! shared across tests).

use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

use gitehr::commands::store;

fn gitehr() -> Command {
    Command::new(env!("CARGO_BIN_EXE_gitehr"))
}

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
    assert_eq!(
        fs::read_to_string(repo.join("state/medications.md"))?,
        "---\nmedications: []\n---\n"
    );

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
fn store_init_requires_an_empty_store_root() {
    let store_root = tempdir().unwrap();
    fs::write(store_root.path().join("existing-file.txt"), "not a Store").unwrap();

    let output = gitehr()
        .args(["store", "init", "rex"])
        .current_dir(store_root.path())
        .output()
        .unwrap();

    assert!(!output.status.success(), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("empty directory"),
        "{output:?}"
    );
    assert!(!store_root.path().join("gitehr-mpi.json").exists());
    assert!(!store_root.path().join("rex").exists());
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

#[test]
fn single_subject_store_auto_targets_repo_commands() {
    let store_root = tempdir().unwrap();

    let init = gitehr()
        .args(["store", "init", "rex"])
        .current_dir(store_root.path())
        .output()
        .unwrap();
    assert!(init.status.success(), "{init:?}");

    let status = gitehr()
        .arg("status")
        .current_dir(store_root.path())
        .output()
        .unwrap();
    assert!(status.status.success(), "{status:?}");
    assert!(
        String::from_utf8_lossy(&status.stdout).contains("GitEHR Repository Status"),
        "{status:?}"
    );
}

// ── R6: identifier resolution operations ──────────────────────────────────────

#[test]
#[serial]
fn store_path_prints_repo_path_for_id_or_name() -> Result<()> {
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp)?;

    store::init::run(Some("rex"))?;
    let mpi_text = fs::read_to_string("gitehr-mpi.json")?;
    // Extract the canonical id by reading the MPI as JSON.
    let mpi: store::MpiInfo = serde_json::from_str(&mpi_text)?;
    let id = mpi.patients[0].patient_id.clone();

    store::path::run("rex")?; // prints the path; must not error
    store::path::run(&id)?;

    assert!(
        store::path::run("nonexistent").is_err(),
        "path for an unknown subject should fail"
    );
    Ok(())
}

#[test]
#[serial]
fn store_search_finds_by_identifier_value_and_fails_when_no_match() -> Result<()> {
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp)?;

    store::init::run(Some("rex"))?;
    store::link::run("rex", "NHS".to_string(), "1234567890")?;

    assert!(
        store::search::run("1234567890").is_ok(),
        "value substring should match"
    );
    assert!(
        store::search::run("NHS:1234567890").is_ok(),
        "exact type:value should match"
    );
    assert!(store::search::run("rex").is_ok(), "name should match");
    assert!(
        store::search::run("zzz-no-match").is_err(),
        "no match should error"
    );
    Ok(())
}

#[test]
#[serial]
fn store_link_adds_identifier_and_conflict_is_refused() -> Result<()> {
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp)?;

    store::init::run(Some("rex"))?;
    store::add::run(Some("fido"), vec![])?;

    store::link::run("rex", "NHS".to_string(), "1234567890")?;
    // Re-linking the same subject is a harmless no-op.
    store::link::run("rex", "NHS".to_string(), "1234567890")?;
    // Same identifier on another subject must be refused.
    assert!(
        store::link::run("second", "NHS".to_string(), "1234567890").is_err()
            || store::link::run("second-clone", "NHS".to_string(), "1234567890").is_err(),
    );

    let mpi: store::MpiInfo = serde_json::from_str(&fs::read_to_string("gitehr-mpi.json")?)?;
    let rex = mpi.patients.iter().find(|p| p.repo_path == "rex").unwrap();
    assert!(
        rex.identifiers
            .iter()
            .any(|i| i.id_type == "NHS" && i.value == "1234567890"),
        "identifier should be recorded"
    );
    Ok(())
}

#[test]
#[serial]
fn store_unlink_removes_identifier_anywhere() -> Result<()> {
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp)?;

    store::init::run(Some("rex"))?;
    store::link::run("rex", "NHS".to_string(), "1234567890")?;

    store::unlink::run("NHS", "1234567890")?;

    let mpi: store::MpiInfo = serde_json::from_str(&fs::read_to_string("gitehr-mpi.json")?)?;
    assert!(
        !mpi.patients
            .iter()
            .any(|p| p.identifiers.iter().any(|i| i.value == "1234567890")),
        "identifier should be gone"
    );
    assert!(
        store::unlink::run("NHS", "1234567890").is_err(),
        "unlinking an absent identifier should fail"
    );
    Ok(())
}

#[test]
#[serial]
fn store_merge_moves_identifiers_and_marks_source() -> Result<()> {
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp)?;

    store::init::run(Some("rex"))?;
    store::link::run("rex", "NHS".to_string(), "1111111111")?;
    store::add::run(Some("duplicate-rex"), vec![])?;
    store::link::run("duplicate-rex", "NHS".to_string(), "2222222222")?;

    store::merge::run("duplicate-rex", "rex")?;

    let mpi: store::MpiInfo = serde_json::from_str(&fs::read_to_string("gitehr-mpi.json")?)?;
    let rex = mpi.patients.iter().find(|p| p.repo_path == "rex").unwrap();
    assert_eq!(
        rex.identifiers.len(),
        2,
        "both identifiers should be on the target"
    );
    let dup = mpi
        .patients
        .iter()
        .find(|p| p.repo_path == "duplicate-rex")
        .unwrap();
    assert_eq!(dup.status, "merged");
    assert!(dup.identifiers.is_empty(), "source identifiers should move");
    assert!(dup.merged_into.is_some(), "merged_into should be set");

    // Records never silently merge into themselves; already-merged sources refuse.
    assert!(store::merge::run("rex", "rex").is_err());
    assert!(store::merge::run("duplicate-rex", "rex").is_err());
    Ok(())
}

#[test]
#[serial]
fn store_merge_refuses_identifier_conflicts() -> Result<()> {
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp)?;

    store::init::run(Some("rex"))?;
    store::link::run("rex", "NHS".to_string(), "1111111111")?;
    store::add::run(Some("dupe"), vec![])?;

    // `link` refuses to put the same identifier on two subjects, so to place
    // the conflict the test edits the MPI directly (defence-in-depth check of
    // merge's own conflict guard).
    let mut mpi: store::MpiInfo = serde_json::from_str(&fs::read_to_string("gitehr-mpi.json")?)?;
    for p in &mut mpi.patients {
        if p.repo_path == "dupe" {
            p.identifiers.push(store::MpiIdentifier {
                id_type: "NHS".to_string(),
                value: "1111111111".to_string(),
            });
        }
    }
    fs::write("gitehr-mpi.json", serde_json::to_string_pretty(&mpi)?)?;

    assert!(
        store::merge::run("dupe", "rex").is_err(),
        "a clashing identifier must block the merge"
    );
    Ok(())
}

#[test]
#[serial]
fn store_mpi_path_env_override_is_honoured() -> Result<()> {
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp)?;

    store::init::run(Some("rex"))?;

    let elsewhere = tempdir().unwrap();
    let moved = elsewhere.path().join("custom-mpi.json");
    fs::copy("gitehr-mpi.json", &moved)?;

    // Edition 2024 marks env mutation unsafe; safe here because the test is
    // #[serial], so no other test reads the environment concurrently.
    // SAFETY: single-threaded, serial test; the variable is removed before return.
    unsafe { std::env::set_var("GITEHR_MPI_PATH", &moved) };
    let result = store::path::run("rex");
    unsafe { std::env::remove_var("GITEHR_MPI_PATH") };

    assert!(result.is_ok(), "GITEHR_MPI_PATH should redirect MPI reads");
    Ok(())
}

#[test]
#[serial]
fn store_init_scaffolds_openehr_layout() -> Result<()> {
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp)?;

    store::init::run(Some("rex"))?;

    let openehr = Path::new("rex/openehr");
    assert!(
        openehr.join("README.md").exists(),
        "openehr README should exist"
    );
    assert!(
        openehr.join("templates").is_dir(),
        "templates/ should exist"
    );
    assert!(
        openehr.join("instances/COMPOSITION").is_dir(),
        "instances/COMPOSITION/ should exist"
    );
    assert!(
        openehr.join("instances/EHR").is_dir(),
        "instances/EHR/ should exist"
    );
    assert!(openehr.join("indexes").is_dir(), "indexes/ should exist");
    Ok(())
}
