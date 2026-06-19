use anyhow::Result;

pub fn run() -> Result<()> {
    crate::commands::contributor::create_user_interactive()
}
