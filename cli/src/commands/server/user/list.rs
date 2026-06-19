use anyhow::Result;

pub fn run() -> Result<()> {
    crate::commands::contributor::list_contributors()
}
