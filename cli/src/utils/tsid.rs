// Reserved infrastructure for sortable identifiers: exercised by this module's
// tests and pending wiring into repo/entry IDs (see queries.md Q2).
#![allow(dead_code)]

/// Timestamp-prefixed UUID (TSID) utilities
///
/// Implements VPR's approach to sortable identifiers.
/// Format: `20260307T123456.789Z-550e8400-e29b-41d4-a716-446655440000`
///
/// Benefits:
/// - Lexicographically sortable by creation time
/// - UUID provides global uniqueness
/// - Human-readable timestamp component
/// - Compatible with existing UUID-based systems
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Generate a timestamp-prefixed UUID
///
/// Format: `YYYYMMDDTHHmmss.fffZ-<uuid>`
/// Example: `20260307T123456.789Z-550e8400-e29b-41d4-a716-446655440000`
pub fn generate() -> String {
    let now = Utc::now();
    let timestamp = now.format("%Y%m%dT%H%M%S%.3fZ");
    let uuid = Uuid::new_v4();
    format!("{}-{}", timestamp, uuid)
}

/// Generate a TSID with a specific timestamp (for testing)
pub fn generate_at(timestamp: DateTime<Utc>) -> String {
    let timestamp_str = timestamp.format("%Y%m%dT%H%M%S%.3fZ");
    let uuid = Uuid::new_v4();
    format!("{}-{}", timestamp_str, uuid)
}

/// Parse a TSID to extract the timestamp component
///
/// Returns None if the TSID format is invalid
pub fn parse_timestamp(tsid: &str) -> Option<DateTime<Utc>> {
    // Format: 20260307T123456.789Z-<uuid>
    let parts: Vec<&str> = tsid.split('-').collect();
    if parts.is_empty() {
        return None;
    }

    let timestamp_str = parts[0];

    // Remove the trailing 'Z' if present
    let timestamp_str = timestamp_str.trim_end_matches('Z');

    // Parse: YYYYMMDDTHHmmss.fff
    use chrono::NaiveDateTime;
    NaiveDateTime::parse_from_str(timestamp_str, "%Y%m%dT%H%M%S%.3f")
        .ok()
        .map(|naive| naive.and_utc())
}

/// Extract the UUID component from a TSID
pub fn extract_uuid(tsid: &str) -> Option<Uuid> {
    // Format: <timestamp>-<uuid>
    let parts: Vec<&str> = tsid.split('-').collect();
    if parts.len() < 2 {
        return None;
    }

    // UUID is everything after the first hyphen
    let uuid_str = parts[1..].join("-");
    Uuid::parse_str(&uuid_str).ok()
}

/// Validate a TSID format
pub fn is_valid(tsid: &str) -> bool {
    parse_timestamp(tsid).is_some() && extract_uuid(tsid).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, TimeZone, Timelike};

    #[test]
    fn test_generate() {
        let tsid = generate();

        // Should have correct format
        assert!(is_valid(&tsid));

        // Should be parseable
        assert!(parse_timestamp(&tsid).is_some());
        assert!(extract_uuid(&tsid).is_some());
    }

    #[test]
    fn test_generate_at() {
        let timestamp = Utc.with_ymd_and_hms(2026, 3, 7, 12, 34, 56).unwrap();
        let tsid = generate_at(timestamp);

        assert!(tsid.starts_with("20260307T123456.000Z-"));
        assert!(is_valid(&tsid));
    }

    #[test]
    fn test_parse_timestamp() {
        let tsid = "20260307T123456.789Z-550e8400-e29b-41d4-a716-446655440000";
        let timestamp = parse_timestamp(tsid).unwrap();

        assert_eq!(timestamp.year(), 2026);
        assert_eq!(timestamp.month(), 3);
        assert_eq!(timestamp.day(), 7);
        assert_eq!(timestamp.hour(), 12);
        assert_eq!(timestamp.minute(), 34);
        assert_eq!(timestamp.second(), 56);
    }

    #[test]
    fn test_extract_uuid() {
        let tsid = "20260307T123456.789Z-550e8400-e29b-41d4-a716-446655440000";
        let uuid = extract_uuid(tsid).unwrap();

        assert_eq!(uuid.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_is_valid() {
        assert!(is_valid(
            "20260307T123456.789Z-550e8400-e29b-41d4-a716-446655440000"
        ));
        assert!(!is_valid("invalid"));
        assert!(!is_valid("20260307T123456.789Z"));
        assert!(!is_valid("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_sortable() {
        let ts1 = Utc.with_ymd_and_hms(2026, 3, 7, 10, 0, 0).unwrap();
        let ts2 = Utc.with_ymd_and_hms(2026, 3, 7, 11, 0, 0).unwrap();
        let ts3 = Utc.with_ymd_and_hms(2026, 3, 7, 12, 0, 0).unwrap();

        let tsid1 = generate_at(ts1);
        let tsid2 = generate_at(ts2);
        let tsid3 = generate_at(ts3);

        // TSIDs should be lexicographically sortable
        assert!(tsid1 < tsid2);
        assert!(tsid2 < tsid3);
        assert!(tsid1 < tsid3);
    }

    #[test]
    fn test_parse_invalid_tsid() {
        assert!(parse_timestamp("invalid").is_none());
        assert!(extract_uuid("invalid").is_none());
    }
}
