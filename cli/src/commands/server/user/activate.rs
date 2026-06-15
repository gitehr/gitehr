use anyhow::Result;

pub fn run(id: &str) -> Result<()> {
    crate::commands::contributor::activate_contributor(id)
}
