//! Contract Workflow Demo
//!
//! This example demonstrates how to use the unified contract-driven
//! bootstrap & validation workflow for Hooksmith projects.

use std::process::Command;

fn main() {
    println!("🔨 Hooksmith Contract Workflow Demo");
    println!("=====================================");

    // Example 1: Full project build
    println!("\n📋 Example 1: Full Project Build");
    println!("Command: cargo xtask contract build --commit");
    println!("This command will:");
    println!("  • Bootstrap the project if needed");
    println!("  • Regenerate all codegen files");
    println!("  • Validate generated files");
    println!("  • Build all components");
    println!("  • Run all tests");
    println!("  • Check Git hooks installation");
    println!("  • Validate Git attributes");
    println!("  • Commit generated files");

    // Example 2: CI/CD build (no tests)
    println!("\n📋 Example 2: CI/CD Build (No Tests)");
    println!("Command: cargo xtask contract build --no-test");
    println!("This command will:");
    println!("  • Skip running tests (for faster CI builds)");
    println!("  • Still validate and build everything else");

    // Example 3: Force regeneration
    println!("\n📋 Example 3: Force Regeneration");
    println!("Command: cargo xtask contract build --force");
    println!("This command will:");
    println!("  • Force regeneration of all files");
    println!("  • Overwrite existing generated files");

    // Example 4: Health check
    println!("\n📋 Example 4: Project Health Check");
    println!("Command: cargo xtask contract check --strict");
    println!("This command will:");
    println!("  • Validate generated files are up-to-date");
    println!("  • Check project builds successfully");
    println!("  • Verify tests can be compiled");
    println!("  • Check Git hooks installation");
    println!("  • Validate Git attributes");
    println!("  • Run linter checks");
    println!("  • Exit with error if any check fails");

    // Example 5: Pre-push validation
    println!("\n📋 Example 5: Pre-Push Validation");
    println!("Command: cargo xtask contract check --strict --custom-message \"Please run 'cargo xtask contract build' to fix issues\"");
    println!("This command will:");
    println!("  • Run all health checks");
    println!("  • Show custom error message if validation fails");
    println!("  • Perfect for Git pre-push hooks");

    // Example 6: Staged files only
    println!("\n📋 Example 6: Staged Files Only");
    println!("Command: cargo xtask contract check --staged-only");
    println!("This command will:");
    println!("  • Only check staged files");
    println!("  • Faster validation for large repositories");

    // Integration with Git hooks
    println!("\n🪝 Git Hook Integration");
    println!("Add to lefthook.yml:");
    println!("```yaml");
    println!("pre-push:");
    println!("  parallel: true");
    println!("  commands:");
    println!("    contract-check:");
    println!("      run: cargo xtask contract check --strict");
    println!("      description: \"Run contract-driven validation\"");
    println!("```");

    // Error handling examples
    println!("\n❌ Error Handling Examples");
    println!("When validation fails, you'll see:");
    println!("  • Clear error messages for each failed step");
    println!("  • Actionable suggestions for fixing issues");
    println!("  • Summary of what passed and what failed");

    // Benefits summary
    println!("\n🎉 Benefits of Contract Workflow");
    println!("  ✅ Single command for project setup");
    println!("  ✅ Clear, actionable error messages");
    println!("  ✅ Consistent environment across developers");
    println!("  ✅ Automated validation prevents drift");
    println!("  ✅ Integration with Git hooks");
    println!("  ✅ Perfect for CI/CD pipelines");

    println!("\n🚀 Ready to try it?");
    println!("Run: cargo xtask contract build --commit");
} 
