// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Read-only Medical Markdown extraction (R65, ADR-0004).
//!
//! Structured data is always a pure function of the journal body. This is the
//! single wrapper through which GitEHR reads `CODE/` shorthand, keeping the
//! independently maintained `medical-markdown` parser behind a stable local
//! boundary.

use serde_json::Value;

/// Extract structured data from a journal body.
pub fn extract(body: &str) -> Value {
    medical_markdown::parse(body).structured()
}
