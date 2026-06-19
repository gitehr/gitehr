use anyhow::Result;

pub fn run(id: &str) -> Result<()> {
    crate::commands::contributor::enable_contributor(id)
}
