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

use std::collections::{BTreeSet, HashSet};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

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

    let path_var = std::env::var_os("PATH");
    let path = resolve(name, path_var.as_deref())
        .ok_or_else(|| anyhow::anyhow!("{}", unrecognized(name)))?;

    exec_plugin(&path, &args[1..])
}

/// Find the executable `gitehr <name>` runs, searching a `PATH`-style list.
/// Dispatch and discovery both use this, so `gitehr plugins` lists exactly
/// what would run: first match in `PATH` order, executables only, and on
/// Windows any `PATHEXT` extension.
fn resolve(name: &str, path_var: Option<&OsStr>) -> Option<PathBuf> {
    // The working directory only anchors relative PATH entries. It may be
    // unreadable (a deleted directory inherited from a parent), and that must
    // not hide plugins installed in absolute PATH directories.
    let cwd = std::env::current_dir().unwrap_or_default();
    which::which_in(format!("{PREFIX}{name}"), path_var, cwd).ok()
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
/// same-named plugin. Derived from the clap command so it cannot drift. The
/// command is built first so clap's generated `help` subcommand, which also
/// wins over a `gitehr-help` plugin, is included.
pub fn builtin_names(cmd: &clap::Command) -> HashSet<String> {
    let mut cmd = cmd.clone();
    cmd.build();
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
    path: PathBuf,
}

/// List the plugins on `$PATH` that [`run`] would dispatch, sorted by name.
fn discover(builtins: &HashSet<String>) -> Vec<Plugin> {
    match std::env::var_os("PATH") {
        Some(path_var) => discover_in(&path_var, builtins),
        None => Vec::new(),
    }
}

/// The testable core of [`discover`], for an explicit `PATH`-style list.
///
/// Scanning only proposes names. Each name is then held to the same rules as
/// [`run`]: it must pass [`is_valid_name`], must not be a built-in, and must
/// [`resolve`] - which also supplies the path shown - so the listing cannot
/// drift from what dispatch actually does.
fn discover_in(path_var: &OsStr, builtins: &HashSet<String>) -> Vec<Plugin> {
    let mut names = BTreeSet::new();
    for dir in std::env::split_paths(path_var) {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            if let Some(name) = candidate_name(&entry.file_name())
                && is_valid_name(&name)
                && !builtins.contains(&name)
            {
                names.insert(name);
            }
        }
    }

    names
        .into_iter()
        .filter_map(|name| {
            let path = resolve(&name, Some(path_var))?;
            Some(Plugin { name, path })
        })
        .collect()
}

/// The name a `gitehr-*` file would be invoked by. On Windows the extension
/// is dropped (`gitehr-foo.exe` runs as `gitehr foo`); elsewhere it is part of
/// the name, so `gitehr-foo.exe` is `foo.exe`, which no valid name matches.
fn candidate_name(file_name: &OsStr) -> Option<String> {
    #[cfg(windows)]
    let file_name = Path::new(file_name).file_stem()?;
    file_name.to_str()?.strip_prefix(PREFIX).map(str::to_string)
}

/// A plugin name must be a simple token, so a crafted subcommand can never be
/// turned into a path lookup (`../`, absolute paths, etc.).
fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
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

        let found: Vec<String> = discover_in(dir.path().as_os_str(), &builtins)
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

    #[test]
    fn discover_excludes_names_that_cannot_be_dispatched() {
        let dir = tempfile::tempdir().unwrap();
        for f in ["gitehr-ok", "gitehr-dot.dot", "gitehr-", "gitehr-sp ace"] {
            let p = dir.path().join(f);
            std::fs::write(&p, "#!/bin/sh\n").unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
            }
        }
        let found: Vec<String> = discover_in(dir.path().as_os_str(), &HashSet::new())
            .into_iter()
            .map(|p| p.name)
            .collect();
        assert_eq!(found, vec!["ok".to_string()]);
    }

    #[cfg(unix)]
    #[test]
    fn discover_keeps_exe_suffix_off_windows() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("gitehr-foo.exe");
        std::fs::write(&p, "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        // `gitehr foo` would not find `gitehr-foo.exe` on Unix, so it must
        // not be listed as `foo`.
        let found = discover_in(dir.path().as_os_str(), &HashSet::new());
        assert!(found.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn discover_lists_the_path_dispatch_would_run() {
        use std::os::unix::fs::PermissionsExt;
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let write = |dir: &Path, name: &str, mode: u32| {
            let p = dir.join(name);
            std::fs::write(&p, "#!/bin/sh\n").unwrap();
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(mode)).unwrap();
            p
        };
        // Both directories hold an executable `tool`: the first in PATH wins.
        let tool = write(first.path(), "gitehr-tool", 0o755);
        write(second.path(), "gitehr-tool", 0o755);
        // `other` is not executable in the first directory, so the second runs.
        write(first.path(), "gitehr-other", 0o644);
        let other = write(second.path(), "gitehr-other", 0o755);

        let path_var = std::env::join_paths([first.path(), second.path()]).unwrap();
        let found: Vec<(String, PathBuf)> = discover_in(&path_var, &HashSet::new())
            .into_iter()
            .map(|p| (p.name, p.path))
            .collect();
        assert_eq!(
            found,
            vec![("other".to_string(), other), ("tool".to_string(), tool)]
        );
        for (name, path) in &found {
            assert_eq!(resolve(name, Some(&path_var)).as_ref(), Some(path));
        }
    }

    #[test]
    fn builtin_names_include_the_generated_help_subcommand() {
        let cmd = clap::Command::new("t").subcommand(clap::Command::new("status"));
        let names = builtin_names(&cmd);
        assert!(names.contains("status"));
        assert!(names.contains("help"));
    }
}
