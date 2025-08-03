use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Structured Logging Tools Demo");
    println!("================================");
    println!();

    // Check if tools are available
    let tools = check_tools_availability();

    if !tools.all_available {
        println!("⚠️  Some tools are not available. Installing...");
        install_tools()?;
    }

    println!("✅ Tools ready!");
    println!();

    // Generate sample structured logging output
    println!("📝 Generating sample structured logging output...");
    let sample_events = generate_sample_events();

    // Save to temporary file
    let temp_file = std::env::temp_dir().join("hooksmith_demo_events.jsonl");
    std::fs::write(&temp_file, sample_events)?;

    println!("📁 Sample events saved to: {}", temp_file.display());
    println!();

    // Demonstrate various tools
    demonstrate_jql(&temp_file)?;
    demonstrate_jless(&temp_file)?;
    demonstrate_fblog(&temp_file)?;

    // Clean up
    std::fs::remove_file(temp_file)?;

    println!("🎉 Demo completed!");
    println!();
    println!("💡 Try these commands with your own structured logging output:");
    println!("   cargo run -p xtask -- structured-auto-push | jql '\"level\"'");
    println!("   cargo run -p xtask -- structured-auto-push | fblog -f 'level == \"error\"'");
    println!("   cargo run -p xtask -- structured-auto-push | jless");

    Ok(())
}

#[derive(Debug)]
struct ToolAvailability {
    jql: bool,
    jless: bool,
    fblog: bool,
    all_available: bool,
}

fn check_tools_availability() -> ToolAvailability {
    let jql = Command::new("jql").arg("--version").output().is_ok();
    let jless = Command::new("jless").arg("--version").output().is_ok();
    let fblog = Command::new("fblog").arg("--version").output().is_ok();

    let all_available = jql && jless && fblog;

    ToolAvailability {
        jql,
        jless,
        fblog,
        all_available,
    }
}

fn install_tools() -> Result<(), Box<dyn std::error::Error>> {
    println!("Installing tools via cargo...");

    let tools = ["jql", "jless", "fblog"];
    for tool in &tools {
        println!("Installing {tool}...");
        let status = Command::new("cargo").args(["install", tool]).status()?;

        if status.success() {
            println!("✅ {tool} installed");
        } else {
            println!("❌ Failed to install {tool}");
        }
    }

    Ok(())
}

fn generate_sample_events() -> String {
    let events = [r#"{"timestamp":"2025-08-03T18:30:00Z","level":"info","tool":"hooksmith","action":"start","message":"Starting structured auto-push workflow","session_id":"demo-session-123"}"#,
        r#"{"timestamp":"2025-08-03T18:30:05Z","level":"info","tool":"cargo","action":"check","message":"Running cargo check","session_id":"demo-session-123"}"#,
        r#"{"timestamp":"2025-08-03T18:30:10Z","level":"warn","tool":"cargo","action":"clippy","message":"variables can be used directly in the `format!` string","code":"clippy::uninlined_format_args","file":"src/main.rs","line":42,"column":9,"session_id":"demo-session-123"}"#,
        r#"{"timestamp":"2025-08-03T18:30:15Z","level":"info","tool":"git","action":"status","message":"Checking git status","session_id":"demo-session-123"}"#,
        r#"{"timestamp":"2025-08-03T18:30:20Z","level":"error","tool":"cargo","action":"test","message":"test failed","code":"E0001","file":"tests/test.rs","line":15,"column":1,"session_id":"demo-session-123"}"#,
        r#"{"timestamp":"2025-08-03T18:30:25Z","level":"info","tool":"hooksmith","action":"completion","message":"Workflow completed","details":{"duration_ms":25000,"success":false},"session_id":"demo-session-123"}"#];

    events.join("\n")
}

fn demonstrate_jql(temp_file: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Demonstrating jql (JSON Query Language)...");
    println!("   jql is a fast, jq-like CLI for querying JSON data");
    println!();

    let queries = vec![
        ("All events", r#"."#),
        ("Error events only", r#""level" == "error""#),
        ("Tool breakdown", r#""tool""#),
        ("Action breakdown", r#""action""#),
        (
            "Events with diagnostic codes",
            r#"select("code") | {"file", "line", "code", "message"}"#,
        ),
        (
            "Performance metrics",
            r#""action" == "completion" | "details""#,
        ),
    ];

    for (description, query) in queries {
        println!("   {description}: jql '{query}'");

        let output = Command::new("jql").arg(query).arg(temp_file).output()?;

        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = result.lines().collect();

            if lines.len() <= 3 {
                println!("      Result: {}", result.trim());
            } else {
                println!("      Result: {} lines (showing first 3)", lines.len());
                for line in lines.iter().take(3) {
                    println!("        {line}");
                }
            }
        } else {
            println!("      Error: {}", String::from_utf8_lossy(&output.stderr));
        }
        println!();
    }

    Ok(())
}

fn demonstrate_jless(temp_file: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("👀 Demonstrating jless (JSON Viewer)...");
    println!("   jless provides interactive, syntax-highlighted JSON browsing");
    println!();
    println!("   Command: jless {}", temp_file.display());
    println!("   (This would open an interactive viewer)");
    println!();
    println!(
        "   Try: jql '\"level\" == \"error\"' {} | jless",
        temp_file.display()
    );
    println!("   (This would show only error events in the viewer)");
    println!();

    Ok(())
}

fn demonstrate_fblog(temp_file: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("📺 Demonstrating fblog (JSON Log Tailer)...");
    println!("   fblog filters JSON lines with Lua expressions");
    println!();

    let filters = vec![
        ("Error events", r#"level == "error""#),
        ("Cargo tool events", r#"tool == "cargo""#),
        ("Events with file paths", r#"file and file:find("main.rs")"#),
        ("Events with diagnostic codes", r#"code"#),
    ];

    for (description, filter) in filters {
        println!(
            "   {}: fblog -f '{}' {}",
            description,
            filter,
            temp_file.display()
        );

        let output = Command::new("fblog")
            .args(["-f", filter, "-d"])
            .arg(temp_file)
            .output()?;

        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = result.lines().collect();

            if lines.len() <= 3 {
                println!("      Result: {} lines", lines.len());
                for line in lines {
                    println!("        {line}");
                }
            } else {
                println!("      Result: {} lines (showing first 3)", lines.len());
                for line in lines.iter().take(3) {
                    println!("        {line}");
                }
            }
        } else {
            println!("      Error: {}", String::from_utf8_lossy(&output.stderr));
        }
        println!();
    }

    Ok(())
}
