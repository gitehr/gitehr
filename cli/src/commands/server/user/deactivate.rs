use anyhow::Result;

pub fn run() -> Result<()> {
    crate::commands::contributor::deactivate_contributor()
}
