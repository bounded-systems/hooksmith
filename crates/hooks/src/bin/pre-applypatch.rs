use anyhow::Result;

fn main() -> Result<()> {
    println!("✅ Pre-applypatch hook (no-op mode) - would validate before applying patch");
    Ok(())
}
