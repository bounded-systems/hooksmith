use xtask::dashboard::{Dashboard, DashboardConfig, AutoPushConfig};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Starting Hooksmith TUI Dashboard Demo...");
    println!("This demo shows the modernized dashboard using ratatui");

    // Create dashboard configuration
    let config = DashboardConfig {
        show_dashboard: true,
        log_to_jsonl: false,
        jsonl_path: None,
        auto_push_config: AutoPushConfig {
            enabled: false, // Disable auto-push for demo
            commit_message: None,
            skip_validation: false,
        },
        file_watch_mode: false,
        heartbeat_interval: 5, // Faster heartbeat for demo
    };

    // Create and start the dashboard
    let mut dashboard = Dashboard::new(config)?;

    println!("📊 Dashboard initialized. You should see a modern TUI interface.");
    println!("💡 The dashboard will show:");
    println!("   - System status and uptime");
    println!("   - Error tracking and deduplication");
    println!("   - Event statistics");
    println!("   - Auto-push status");
    println!("");
    println!("🎮 Controls:");
    println!("   - Ctrl+C to exit");
    println!("   - Tab keys to switch between views (when implemented)");
    println!("");
    println!("⏳ Starting dashboard in 3 seconds...");

    sleep(Duration::from_secs(3)).await;

    // Start the dashboard
    dashboard.start().await?;

    Ok(())
} 
