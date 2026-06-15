pub mod tsid;

use sha2::{Digest, Sha256};
use std::fmt::Write;

/// SHA-256 of `bytes` as a lowercase, zero-padded hex string (64 chars).
///
/// Centralised so the hash format that anchors the journal chain and Document
/// references stays byte-identical regardless of the digest crate's output
/// type (sha2 0.11 switched its output from `generic-array`, which implemented
/// `LowerHex`, to `hybrid-array`, which does not).
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest.iter() {
        let _ = write!(out, "{:02x}", byte);
    }
    out
}
