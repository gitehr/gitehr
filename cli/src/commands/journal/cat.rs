use anyhow::Result;
use std::{fs, path::PathBuf};

use super::{is_journal_entry_file, parse_journal_file};

/// Print the full body of every journal entry, in chronological order by default.
///
/// This is the "play back the record" view: one entry after another with a
/// clear header line for each, intended for reading the journal end to end.
pub fn run(reverse: bool) -> Result<()> {
    let journal_dir = PathBuf::from("journal");
    if !journal_dir.exists() {
        println!("No journal entries found.");
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(&journal_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(|name| is_journal_entry_file(name))
                .unwrap_or(false)
        })
        .collect();

    if entries.is_empty() {
        println!("No journal entries found.");
        return Ok(());
    }

    entries.sort();
    if reverse {
        entries.reverse();
    }

    let total = entries.len();

    for (idx, path) in entries.iter().enumerate() {
        let entry_num = idx + 1;

        match parse_journal_file(path) {
            Ok(parsed) => {
                let author = parsed.metadata.author.as_deref().unwrap_or("(unknown)");
                let ts = parsed.metadata.timestamp.format("%Y-%m-%d %H:%M:%S UTC");
                println!("--- Entry {} | {} | author: {} ---", entry_num, ts, author);
                println!("{}", parsed.filename);
                println!();
                println!("{}", parsed.content);
                println!();
            }
            Err(e) => {
                let filename = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                eprintln!("--- Entry {} | {} (error: {}) ---", entry_num, filename, e);
                eprintln!();
            }
        }
    }

    println!("({} entries)", total);

    Ok(())
}
