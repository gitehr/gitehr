pub mod init;
pub mod journal;

// Re-export commands for easier access
pub use init::initialise;
pub use journal::{create_journal_entry, get_latest_journal_entry};
