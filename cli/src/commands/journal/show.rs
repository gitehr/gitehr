use anyhow::Result;
use std::{fs, path::PathBuf};

use super::{is_journal_entry_file, parse_journal_file};

pub fn run(limit: usize, offset: usize, reverse: bool, all: bool) -> Result<()> {
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

    let total_count = entries.len();

    let entries_to_show: Vec<_> = if all {
        entries.into_iter().skip(offset).collect()
    } else {
        entries.into_iter().skip(offset).take(limit).collect()
    };

    if entries_to_show.is_empty() {
        println!(
            "No entries to display (offset {} exceeds total {}).",
            offset, total_count
        );
        return Ok(());
    }

    for (idx, path) in entries_to_show.iter().enumerate() {
        let entry_num = offset + idx + 1;

        match parse_journal_file(path) {
            Ok(parsed) => {
                let preview: String = parsed
                    .content
                    .chars()
                    .take(80)
                    .collect::<String>()
                    .replace('\n', " ");
                let preview_suffix = if parsed.content.len() > 80 { "..." } else { "" };

                println!("[{}] {}", entry_num, parsed.filename);
                println!(
                    "    Timestamp: {}",
                    parsed.metadata.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                );
                println!("    Preview: {}{}", preview, preview_suffix);
                println!();
            }
            Err(e) => {
                let filename = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                println!("[{}] {} (error: {})", entry_num, filename, e);
                println!();
            }
        }
    }

    let shown = entries_to_show.len();
    println!("Showing {} of {} entries.", shown, total_count);

    Ok(())
}
