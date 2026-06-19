use anyhow::Result;

pub fn run(id: &str) -> Result<()> {
    crate::commands::contributor::disable_contributor(id)
}
