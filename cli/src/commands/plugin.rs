// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Git-style `$PATH` plugins.
//!
//! `gitehr <name> [args...]` for an unknown `<name>` runs the executable
//! `gitehr-<name>` found on `$PATH`, passing the remaining arguments through.
//! This makes gitehr extensible without recompiling it: install
//! `gitehr-export` on your `PATH` and `gitehr export` works.
//!
//! **Built-in commands always win.** clap resolves every defined subcommand
//! (and its aliases) before an unknown one falls through to
//! [`run`], so a plugin can never shadow or intercept a built-in such as
//! `journal`. Plugin names are also validated, so `gitehr ../../evil` cannot
//! be coerced into a path.

use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;

/// Executable prefix a plugin must carry to be discovered: `gitehr-<name>`.
const PREFIX: &str = "gitehr-";

/// Dispatch an unknown subcommand to a `gitehr-<name>` plugin on `$PATH`.
///
/// `args[0]` is the subcommand name; the rest are forwarded verbatim. On Unix
/// the plugin replaces this process (`exec`), so its stdio, signals, and exit
/// code pass through transparently.
pub fn run(args: Vec<String>) -> Result<()> {
    let name = args.first().map(String::as_str).unwrap_or_default();

    if !is_valid_name(name) {
        anyhow::bail!("{}", unrecognized(name));
    }

    let exe = format!("{PREFIX}{name}");
    let path = which::which(&exe).map_err(|_| anyhow::anyhow!("{}", unrecognized(name)))?;

    exec_plugin(&path, &args[1..])
}

/// `gitehr plugins`: list installed plugins (`gitehr-*` on `$PATH`, excluding
/// any whose name is a built-in command and therefore unreachable as a plugin).
pub fn list(builtins: &HashSet<String>) -> Result<()> {
    let plugins = discover(builtins);
    if plugins.is_empty() {
        println!(
            "No plugins found.\n\nInstall an executable named `gitehr-<command>` on your PATH, \
then run it as `gitehr <command>`."
        );
        return Ok(());
    }
    println!("Installed plugins (run as `gitehr <name>`):\n");
    for p in &plugins {
        println!("  {:<16} {}", p.name, p.path.display());
    }
    Ok(())
}

/// A help section listing discovered plugins, for `gitehr --help`. `None` when
/// there are no plugins, so the help is unchanged on a plain install.
pub fn plugins_help_section(builtins: &HashSet<String>) -> Option<String> {
    let plugins = discover(builtins);
    if plugins.is_empty() {
        return None;
    }
    let mut s = String::from("Plugins (gitehr-* on PATH; run as `gitehr <name>`):\n");
    for p in &plugins {
        s.push_str(&format!("  {}\n", p.name));
    }
    Some(s)
}

/// The names (and aliases) of every built-in subcommand, which always shadow a
/// same-named plugin. Derived from the clap command so it cannot drift.
pub fn builtin_names(cmd: &clap::Command) -> HashSet<String> {
    let mut names = HashSet::new();
    for sub in cmd.get_subcommands() {
        names.insert(sub.get_name().to_string());
        for alias in sub.get_all_aliases() {
            names.insert(alias.to_string());
        }
    }
    names
}

/// A discovered plugin.
struct Plugin {
    name: String,
    path: std::path::PathBuf,
}

/// Scan `$PATH` for `gitehr-<name>` executables. First match per name wins
/// (PATH order); built-in names are excluded since they are unreachable as
/// plugins. Result is sorted by name.
fn discover(builtins: &HashSet<String>) -> Vec<Plugin> {
    match std::env::var_os("PATH") {
        Some(path) => discover_in(std::env::split_paths(&path), builtins),
        None => Vec::new(),
    }
}

/// Discover plugins across an explicit list of directories (the testable core
/// of [`discover`], which supplies `$PATH`).
fn discover_in(
    dirs: impl Iterator<Item = std::path::PathBuf>,
    builtins: &HashSet<String>,
) -> Vec<Plugin> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<Plugin> = Vec::new();

    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            let Some(name) = file_name.strip_prefix(PREFIX) else {
                continue;
            };
            // Tolerate a Windows .exe suffix on the discovered name.
            let name = name.strip_suffix(".exe").unwrap_or(name);
            if name.is_empty() || builtins.contains(name) || seen.contains(name) {
                continue;
            }
            if !is_executable(&entry.path()) {
                continue;
            }
            seen.insert(name.to_string());
            out.push(Plugin {
                name: name.to_string(),
                path: entry.path(),
            });
        }
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// A plugin name must be a simple token, so a crafted subcommand can never be
/// turned into a path lookup (`../`, absolute paths, etc.).
fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn is_executable(p: &Path) -> bool {
    if !p.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(p)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Run the plugin, replacing this process on Unix so signals/exit/TTY pass
/// through. On other platforms, run it and propagate the exit code.
fn exec_plugin(path: &Path, args: &[String]) -> Result<()> {
    let mut cmd = std::process::Command::new(path);
    cmd.args(args);

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // `exec` only returns on failure.
        let err = cmd.exec();
        Err(anyhow::anyhow!("failed to run {}: {err}", path.display()))
    }
    #[cfg(not(unix))]
    {
        let status = cmd.status()?;
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn unrecognized(name: &str) -> String {
    format!(
        "unrecognized subcommand '{name}'\n\n\
'{name}' is not a built-in gitehr command, and no plugin '{PREFIX}{name}' was found on PATH.\n\n\
Run 'gitehr plugins' to list installed plugins, or 'gitehr --help' for built-in commands."
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn valid_names_are_simple_tokens() {
        assert!(is_valid_name("export"));
        assert!(is_valid_name("backup-now"));
        assert!(is_valid_name("foo_bar2"));
        // Anything that could become a path is rejected.
        assert!(!is_valid_name(""));
        assert!(!is_valid_name("../evil"));
        assert!(!is_valid_name("a/b"));
        assert!(!is_valid_name("with space"));
        assert!(!is_valid_name("dot.dot"));
    }

    #[test]
    fn discover_finds_executables_and_excludes_builtins() {
        let dir = tempfile::tempdir().unwrap();
        for f in [
            "gitehr-export",
            "gitehr-journal",
            "gitehr-notexec",
            "unrelated",
        ] {
            let p = dir.path().join(f);
            std::fs::write(&p, "#!/bin/sh\n").unwrap();
            if f != "gitehr-notexec" {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
                }
            }
        }
        // `journal` is a built-in, so the gitehr-journal decoy must be excluded.
        let builtins: HashSet<String> = ["journal".to_string()].into_iter().collect();

        let found: Vec<String> = discover_in([dir.path().to_path_buf()].into_iter(), &builtins)
            .into_iter()
            .map(|p| p.name)
            .collect();

        assert!(found.contains(&"export".to_string()));
        assert!(
            !found.contains(&"journal".to_string()),
            "built-in must be excluded"
        );
        #[cfg(unix)]
        assert!(
            !found.contains(&"notexec".to_string()),
            "non-executable must be excluded"
        );
        assert!(!found.iter().any(|n| n == "unrelated"));
    }
}
