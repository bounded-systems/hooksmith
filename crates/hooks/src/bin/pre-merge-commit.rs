use anyhow::Result;

fn main() -> Result<()> {
    println!("✅ Pre-merge-commit hook (no-op mode) - would validate before merge commit");
    Ok(())
}
