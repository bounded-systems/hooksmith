use anyhow::Result;
use std::process::Command;
use std::time::Instant;

/// Hooksmith Performance Test
///
/// This binary tests FSMonitor performance by comparing git status
/// execution times with and without FSMonitor enabled.
fn main() -> Result<()> {
    println!("🔧 Testing FSMonitor performance...");

    // Test without FSMonitor
    println!("\n📊 Without FSMonitor:");
    git_config_fsmonitor(false)?;
    let time_without = measure_git_status()?;
    println!("⏱️  Time: {:.3}s", time_without);

    // Test with FSMonitor
    println!("\n📊 With FSMonitor:");
    git_config_fsmonitor(true)?;
    let time_with = measure_git_status()?;
    println!("⏱️  Time: {:.3}s", time_with);

    // Calculate improvement
    let improvement = ((time_without - time_with) / time_without) * 100.0;
    
    println!("\n📈 Performance Results:");
    println!("   Without FSMonitor: {:.3}s", time_without);
    println!("   With FSMonitor:    {:.3}s", time_with);
    println!("   Improvement:       {:.1}%", improvement);

    if improvement > 0.0 {
        println!("✅ FSMonitor is providing performance benefits!");
    } else {
        println!("⚠️  FSMonitor may not be providing benefits in this repository");
    }

    println!("\n🎉 Performance test complete!");
    Ok(())
}

fn git_config_fsmonitor(enabled: bool) -> Result<()> {
    let value = if enabled { "true" } else { "false" };
    
    let status = Command::new("git")
        .args(&["config", "core.fsmonitor", value])
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to set core.fsmonitor to {}", value));
    }

    Ok(())
}

fn measure_git_status() -> Result<f64> {
    let start = Instant::now();
    
    let output = Command::new("git")
        .args(&["status"])
        .output()?;

    let duration = start.elapsed();
    let seconds = duration.as_secs_f64();

    if !output.status.success() {
        return Err(anyhow::anyhow!("git status failed"));
    }

    Ok(seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_config_fsmonitor() {
        // Test that we can set the config
        let result = git_config_fsmonitor(true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_measure_git_status() {
        // Test that we can measure git status
        let result = measure_git_status();
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }
}
