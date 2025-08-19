use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Configuring Git to use the Git Proxy");
    println!("========================================");

    // Check if proxy is running
    println!("1️⃣ Checking if proxy is running...");
    let health_check = Command::new("curl")
        .args(["-s", "http://127.0.0.1:8080/health"])
        .output()?;
    
    if health_check.status.success() {
        println!("✅ Git proxy is running on port 8080");
    } else {
        println!("❌ Git proxy is not running. Please start it first:");
        println!("   cargo run -p git-proxy --bin server -- --enable-http --http-port 8080");
        std::process::exit(1);
    }

    println!("\n2️⃣ Current Git configuration:");
    let current_config = Command::new("git")
        .args(["config", "--global", "--list"])
        .output()?;
    
    if current_config.status.success() {
        let config_output = String::from_utf8_lossy(&current_config.stdout);
        let url_lines: Vec<&str> = config_output
            .lines()
            .filter(|line| line.contains("url") || line.contains("http"))
            .collect();
        
        if url_lines.is_empty() {
            println!("   No URL configurations found");
        } else {
            for line in url_lines {
                println!("   {}", line);
            }
        }
    }

    println!("\n3️⃣ Configuring Git to use the proxy...");

    // Configure Git to use our proxy for GitHub
    println!("   Setting up proxy for GitHub...");
    Command::new("git")
        .args(["config", "--global", "url.http://127.0.0.1:8080/.insteadOf", "https://github.com/"])
        .output()?;

    // Configure Git to use our proxy for GitLab
    println!("   Setting up proxy for GitLab...");
    Command::new("git")
        .args(["config", "--global", "url.http://127.0.0.1:8080/.insteadOf", "https://gitlab.com/"])
        .output()?;

    // Configure Git to use our proxy for any HTTPS Git server
    println!("   Setting up proxy for HTTPS Git servers...");
    Command::new("git")
        .args(["config", "--global", "url.http://127.0.0.1:8080/.insteadOf", "https://"])
        .output()?;

    println!("\n4️⃣ Updated Git configuration:");
    let updated_config = Command::new("git")
        .args(["config", "--global", "--list"])
        .output()?;
    
    if updated_config.status.success() {
        let config_output = String::from_utf8_lossy(&updated_config.stdout);
        let url_lines: Vec<&str> = config_output
            .lines()
            .filter(|line| line.contains("url") || line.contains("http"))
            .collect();
        
        if url_lines.is_empty() {
            println!("   No URL configurations found");
        } else {
            for line in url_lines {
                println!("   {}", line);
            }
        }
    }

    println!("\n5️⃣ Testing the configuration...");

    // Create a test repository
    let test_repo = "/tmp/git-proxy-test";
    Command::new("rm").args(["-rf", test_repo]).output()?;
    Command::new("mkdir").args(["-p", test_repo]).output()?;
    
    // Change to test directory
    std::env::set_current_dir(test_repo)?;

    // Initialize a test repository
    Command::new("git").args(["init"]).output()?;
    std::fs::write("test.txt", "Hello from Git Proxy!")?;
    Command::new("git").args(["add", "test.txt"]).output()?;
    Command::new("git").args(["commit", "-m", "Test commit for proxy"]).output()?;

    println!("✅ Test repository created and committed");

    println!("\n6️⃣ Testing push to proxy...");
    println!("   This will fail (expected) since the proxy doesn't have upstream configured");
    println!("   But it shows the proxy is intercepting the request:");

    let push_output = Command::new("git")
        .args(["push", "http://127.0.0.1:8080/test-repo.git", "main"])
        .output();
    
    match push_output {
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let lines: Vec<&str> = stderr.lines().take(5).collect();
            for line in lines {
                println!("   {}", line);
            }
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }

    println!("\n7️⃣ Testing clone through proxy...");
    println!("   This will also fail (expected) but shows proxy interception:");

    let clone_output = Command::new("git")
        .args(["clone", "http://127.0.0.1:8080/test-repo.git", "/tmp/clone-test"])
        .output();
    
    match clone_output {
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let lines: Vec<&str> = stderr.lines().take(5).collect();
            for line in lines {
                println!("   {}", line);
            }
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }

    // Test with a real GitHub URL (this will go through the proxy)
    println!("\n8️⃣ Testing with GitHub URL (goes through proxy)...");
    let github_test = Command::new("git")
        .args(["ls-remote", "https://github.com/octocat/Hello-World.git"])
        .output();
    
    match github_test {
        Ok(output) => {
            if output.status.success() {
                println!("✅ GitHub URL test succeeded (went through proxy)");
                let stdout = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = stdout.lines().take(3).collect();
                for line in lines {
                    println!("   {}", line);
                }
                println!("   ... (truncated)");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("⚠️  GitHub URL test failed (expected):");
                println!("   {}", stderr);
                println!("   This is expected since the proxy doesn't have upstream configured");
            }
        }
        Err(e) => {
            println!("❌ GitHub URL test failed: {}", e);
        }
    }

    println!("\n🎉 Git Proxy Configuration Complete!");
    println!("===================================");
    println!();
    println!("✅ Git is now configured to use the proxy for:");
    println!("   - GitHub repositories");
    println!("   - GitLab repositories"); 
    println!("   - Any HTTPS Git server");
    println!();
    println!("📋 To test with a real repository:");
    println!("   git clone https://github.com/username/repo.git");
    println!("   (This will go through your proxy at http://127.0.0.1:8080/)");
    println!();
    println!("📋 To disable the proxy:");
    println!("   git config --global --unset url.http://127.0.0.1:8080/.insteadOf");
    println!();
    println!("📋 To see current proxy configuration:");
    println!("   git config --global --list | grep url");
    println!();
    println!("📋 To run this script again:");
    println!("   cargo run --bin configure-git-proxy");

    Ok(())
}
