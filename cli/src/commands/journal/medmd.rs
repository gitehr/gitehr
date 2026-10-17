// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Read-only Medical Markdown extraction (R65, ADR-0004).
//!
//! Structured data is always a pure function of the journal body. This is the
//! single wrapper through which GitEHR reads `CODE/` shorthand, so the
//! upstream `medical-markdown` crate can replace the parser below (it is not
//! yet published on crates.io) without touching callers. Output follows the
//! shape in `spec/medical-markdown.md`.

use serde_json::{Map, Value, json};

/// Parse a `CODE/ text` line into (indent, code, text).
fn parse_code_line(line: &str) -> Option<(usize, &str, &str)> {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    let slash = trimmed.find('/')?;
    let code = &trimmed[..slash];
    let valid = !code.is_empty()
        && code.len() <= 8
        && code.starts_with(|c: char| c.is_ascii_uppercase())
        && code
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
    if !valid {
        return None;
    }
    let rest = &trimmed[slash + 1..];
    // `CODE/` must be followed by whitespace or end of line, so URLs and
    // dates such as `10/12` or `N/A` are not mistaken for codes.
    if !(rest.is_empty() || rest.starts_with(char::is_whitespace)) {
        return None;
    }
    Some((indent, code, rest.trim()))
}

/// Extract structured data from a journal body. Entries with no `CODE/`
/// lines extract to an empty object (plus an empty `_source_map`).
pub fn extract(body: &str) -> Value {
    let mut data = Map::new();
    let mut source_map = Map::new();
    // (top-level code, optional sub-code) currently receiving continuation text
    let mut current: Option<(String, Option<String>)> = None;

    for (i, line) in body.lines().enumerate() {
        let line_no = i + 1;
        if line.trim().is_empty() {
            continue;
        }
        match parse_code_line(line) {
            Some((0, code, text)) => {
                let key = code.to_string();
                data.insert(key.clone(), json!({ "notes": text }));
                source_map.insert(key.clone(), json!(line_no));
                current = Some((key, None));
            }
            Some((_, sub, text)) => {
                if let Some((parent, _)) = current.clone() {
                    if let Some(Value::Object(obj)) = data.get_mut(&parent) {
                        obj.insert(sub.to_string(), json!(text));
                    }
                    source_map.insert(format!("{parent}.{sub}"), json!(line_no));
                    current = Some((parent, Some(sub.to_string())));
                }
            }
            None => {
                // Continuation of the previous code's free text.
                if let Some((parent, sub)) = &current
                    && let Some(Value::Object(obj)) = data.get_mut(parent)
                {
                    let field = sub.as_deref().unwrap_or("notes");
                    let prev = obj.get(field).and_then(Value::as_str).unwrap_or("");
                    let joined = if prev.is_empty() {
                        line.trim().to_string()
                    } else {
                        format!("{prev} {}", line.trim())
                    };
                    obj.insert(field.to_string(), json!(joined));
                }
            }
        }
    }

    data.insert("_source_map".to_string(), Value::Object(source_map));
    Value::Object(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_codes_subcodes_and_source_map() {
        let body = "PC/ chest pain\nOE/ alert\n  RS/ clear\n  CVS/ no murmurs\n";
        let v = extract(body);
        assert_eq!(v["PC"]["notes"], "chest pain");
        assert_eq!(v["OE"]["notes"], "alert");
        assert_eq!(v["OE"]["RS"], "clear");
        assert_eq!(v["OE"]["CVS"], "no murmurs");
        assert_eq!(v["_source_map"]["PC"], 1);
        assert_eq!(v["_source_map"]["OE.CVS"], 4);
    }

    #[test]
    fn prose_and_non_codes_extract_to_empty() {
        let v = extract("Patient seen today, BP 10/12 and N/A\nSee http://x/y");
        assert_eq!(v, json!({ "_source_map": {} }));
    }

    #[test]
    fn continuation_lines_join_previous_code() {
        let v = extract("HPC/ started 2 hours ago\nradiating to the arm\n");
        assert_eq!(
            v["HPC"]["notes"],
            "started 2 hours ago radiating to the arm"
        );
    }
}
