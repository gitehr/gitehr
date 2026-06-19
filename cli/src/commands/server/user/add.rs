use anyhow::Result;

pub fn run(id: &str, name: &str, role: Option<&str>, email: Option<&str>) -> Result<()> {
    crate::commands::contributor::add_contributor(id, name, role, email, None)
}
