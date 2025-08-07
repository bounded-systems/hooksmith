use anyhow::Result;

fn main() -> Result<()> {
    println!("✅ Pre-rebase hook (no-op mode) - would validate before rebase");
    Ok(())
}
