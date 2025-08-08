use std::process::Command;

fn main() {
    println!("🔍 Verifying GitHub stub binaries...");

    // List of all GitHub event stub binaries we have
    let github_stubs = vec![
        "github-push",
        "github-pull-request",
        "github-issues",
        "github-release",
        "github-create",
        "github-delete",
        "github-branch-protection-rule",
        "github-check-run",
    ];

    let mut all_working = true;

    // Check GitHub stub binaries
    println!("\n📋 Checking GitHub stub binaries:");
    for stub in &github_stubs {
        let output = Command::new("cargo").args(&["run", "--bin", stub]).output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("✅ {} - exists and runs successfully", stub);
                } else {
                    println!(
                        "⚠️  {} - exists but failed with exit code {}",
                        stub, output.status
                    );
                    all_working = false;
                }
            }
            Err(_) => {
                println!("❌ {} - missing or failed to run", stub);
                all_working = false;
            }
        }
    }

    // Summary
    println!("\n📊 Summary:");
    if all_working {
        println!("🎉 All GitHub stub binaries are working correctly!");
        println!("✅ Ready for GitHub Actions integration");
    } else {
        println!("⚠️  Some binaries failed. Check the output above for details.");
        std::process::exit(1);
    }

    println!("\n📋 Available GitHub stub binaries:");
    for stub in &github_stubs {
        println!("  - {}", stub);
    }

    println!("\n🚀 To enable validation, edit .github/workflows/hooksmith.yml and set:");
    println!("   ENABLE_HOOKSMITH_VALIDATION: true");

    println!("\n✅ Verification complete!");
}
