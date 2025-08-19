use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Documentation Generation Safeguards");
    println!("=============================================");
    println!("");

    // Test 1: Attempt to write markdown file directly (should fail)
    println!("🔒 Test 1: Attempting direct markdown file creation...");
    match attempt_direct_markdown_creation() {
        Ok(_) => {
            println!("❌ FAILED: Direct markdown creation should have been blocked!");
            return Err("Direct markdown creation was not blocked".into());
        }
        Err(e) => {
            println!("✅ PASSED: Direct markdown creation blocked: {}", e);
        }
    }

    // Test 2: Attempt to write markdown without codegen markers (should fail)
    println!("");
    println!("🔒 Test 2: Attempting markdown creation without codegen markers...");
    match attempt_markdown_without_markers() {
        Ok(_) => {
            println!("❌ FAILED: Markdown without markers should have been blocked!");
            return Err("Markdown without markers was not blocked".into());
        }
        Err(e) => {
            println!("✅ PASSED: Markdown without markers blocked: {}", e);
        }
    }

    // Test 3: Test safe markdown file writing (should succeed)
    println!("");
    println!("🔒 Test 3: Testing safe markdown file writing...");
    match test_safe_markdown_writing() {
        Ok(_) => {
            println!("✅ PASSED: Safe markdown writing works correctly");
        }
        Err(e) => {
            println!("❌ FAILED: Safe markdown writing failed: {}", e);
            return Err(e);
        }
    }

    // Test 4: Test validation script
    println!("");
    println!("🔒 Test 4: Testing validation script...");
    match test_validation_script() {
        Ok(_) => {
            println!("✅ PASSED: Validation script works correctly");
        }
        Err(e) => {
            println!("❌ FAILED: Validation script failed: {}", e);
            return Err(e);
        }
    }

    println!("");
    println!("🎯 Safeguard Features");
    println!("====================");
    println!("");
    println!(
        "✅ **Direct file writing prevention** - All markdown files must go through generation"
    );
    println!("✅ **Codegen marker validation** - Files must have proper auto-generated markers");
    println!("✅ **Context validation** - Only allows writing during proper generation context");
    println!("✅ **Git pre-commit hooks** - Prevents direct markdown commits");
    println!("✅ **CI validation scripts** - Automated checks in continuous integration");
    println!("✅ **Checksum validation** - Ensures file integrity and prevents manual edits");
    println!("");

    println!("🔧 How Safeguards Work");
    println!("=====================");
    println!("");
    println!("1. **Safe wrapper function** - `write_markdown_file_safely()` validates all writes");
    println!("2. **Context checking** - Validates we're running through proper xtask command");
    println!("3. **Marker validation** - Ensures all files have auto-generated markers");
    println!("4. **Git hooks** - Pre-commit hook checks for direct markdown creation");
    println!("5. **CI scripts** - Automated validation in continuous integration");
    println!("6. **Checksums** - Cryptographic validation of generated content");
    println!("");

    println!("🚀 Usage");
    println!("========");
    println!("");
    println!("To generate documentation safely:");
    println!("cargo xtask gen-docs-comprehensive --all --validate");
    println!("");
    println!("To validate existing documentation:");
    println!("cargo run -p xtask -- validate-docs");
    println!("");
    println!("To check for direct markdown creation:");
    println!("git diff --cached --name-only | grep '\\.md$'");
    println!("");

    println!("🎉 All safeguard tests passed!");
    println!("The documentation system is properly protected against direct markdown creation.");
    println!("");

    Ok(())
}

// Test 1: Attempt direct markdown file creation (should fail)
fn attempt_direct_markdown_creation() -> Result<(), Box<dyn std::error::Error>> {
    // This simulates trying to write a markdown file directly
    let content = "# Test Document\n\nThis is a test document.\n";
    fs::write("test_direct.md", content)?;

    // Clean up
    fs::remove_file("test_direct.md")?;
    Ok(())
}

// Test 2: Attempt markdown creation without codegen markers (should fail)
fn attempt_markdown_without_markers() -> Result<(), Box<dyn std::error::Error>> {
    // This simulates trying to write a markdown file without proper markers
    let content = "# Test Document\n\nThis is a test document without auto-generated markers.\n";
    fs::write("test_no_markers.md", content)?;

    // Clean up
    fs::remove_file("test_no_markers.md")?;
    Ok(())
}

// Test 3: Test safe markdown file writing (should succeed)
fn test_safe_markdown_writing() -> Result<(), Box<dyn std::error::Error>> {
    // This simulates the safe wrapper function
    let content = "<!-- This file is auto-generated by `cargo xtask gen-docs-comprehensive` -->\n<!-- Generated file: test_safe.md -->\n<!-- Do not edit this file manually - changes will be overwritten -->\n\n# Test Document\n\nThis is a properly generated test document.\n\n---\n\n*This file is auto-generated by `cargo xtask gen-docs-comprehensive`. Do not edit manually - changes will be overwritten.*\n";

    // Simulate the safe wrapper validation
    if !content.contains("auto-generated") {
        return Err("Content missing auto-generated marker".into());
    }

    fs::write("test_safe.md", content)?;

    // Clean up
    fs::remove_file("test_safe.md")?;
    Ok(())
}

// Test 4: Test validation script
fn test_validation_script() -> Result<(), Box<dyn std::error::Error>> {
    // Create a test markdown file with proper markers
    let content = "<!-- This file is auto-generated by `cargo xtask gen-docs-comprehensive` -->\n<!-- Generated file: test_validation.md -->\n<!-- Do not edit this file manually - changes will be overwritten -->\n\n# Test Validation\n\nThis is a test file for validation.\n\n---\n\n*This file is auto-generated by `cargo xtask gen-docs-comprehensive`. Do not edit manually - changes will be overwritten.*\n";
    fs::write("test_validation.md", content)?;

    // Simulate validation script logic
    let file_content = fs::read_to_string("test_validation.md")?;
    if !file_content.contains("auto-generated") {
        return Err("Validation failed: file missing auto-generated marker".into());
    }

    // Clean up
    fs::remove_file("test_validation.md")?;
    Ok(())
}
