use git_filter::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🔧 Git Object Contract Demo - Current API System\n");

    // Example 1: Basic blob contract
    demo_basic_blob_contract()?;

    // Example 2: Blob with invalid UTF-8
    demo_blob_with_invalid_utf8()?;

    // Example 3: Line contracts
    demo_line_contracts()?;

    // Example 4: Complete Git object validation
    demo_complete_git_object_validation()?;

    // Example 5: Diff modeling
    demo_diff_modeling()?;

    Ok(())
}

fn demo_basic_blob_contract() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Example 1: Basic Blob Contract");

    let validator = GitObjectValidator::default();
    let content = b"Hello, World!\nThis is a test file.\nLine 3 with content.\n";

    // Create a blob contract first
    let blob_contract = BlobContract::new("abc123def456".to_string(), content.len());
    let git_object = validator.validate_blob(&blob_contract, None);

    println!("  {}", git_object.summary());
    println!("    OID: {}", git_object.oid);
    println!("    Size: {} bytes", git_object.size);
    println!("    Valid: {}", git_object.is_valid());

    if !git_object.errors.is_empty() {
        println!("    Errors: {:?}", git_object.errors);
    }

    println!();
    Ok(())
}

fn demo_blob_with_invalid_utf8() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚫 Example 2: Blob with Invalid UTF-8");

    let validator = GitObjectValidator::default();
    let content = b"Hello\x80World\nInvalid UTF-8 sequence";

    // Create a blob contract first
    let blob_contract = BlobContract::new("def456ghi789".to_string(), content.len());
    let git_object = validator.validate_blob(&blob_contract, None);

    println!("  {}", git_object.summary());
    println!("    Valid: {}", git_object.is_valid());
    println!("    Errors: {:?}", git_object.errors);

    println!();
    Ok(())
}

fn demo_line_contracts() -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Example 3: Line Contracts");

    let validator = GitObjectValidator::default();
    let content =
        b"Line 1: Normal text\nLine 2: Has\x00NUL byte\nLine 3: CRLF\r\nLine 4: Valid again\n";

    // Create a blob contract first
    let blob_contract = BlobContract::new("ghi789jkl012".to_string(), content.len());
    let git_object = validator.validate_blob(&blob_contract, None);

    println!("  Git Object: {}", git_object.summary());
    println!("    Valid: {}", git_object.is_valid());

    if !git_object.errors.is_empty() {
        println!("    Errors: {:?}", git_object.errors);
    }

    println!();
    Ok(())
}

fn demo_complete_git_object_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Example 4: Complete Git Object Validation");

    let tree_validator = TreeValidator::new(true, true, true);
    let validator = GitObjectValidator::new(true, true, true, true, tree_validator);
    let content = b"Line 1: Valid content\nLine 2: Has\x01control char\nLine 3: CRLF\r\nLine 4: Valid again\n";

    // Validate as a complete Git object
    let git_object = validator.validate_object(
        GitObjectType::Blob,
        "mno345pqr678".to_string(),
        content.len(),
        None,
        None,
    );

    println!("  Git Object Type: {:?}", git_object.object_type);
    println!("  {}", git_object.summary());
    println!("    Valid: {}", git_object.is_valid());

    if !git_object.errors.is_empty() {
        println!("    Errors: {:?}", git_object.errors);
    }

    println!();
    Ok(())
}

fn demo_diff_modeling() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Example 5: Diff Modeling");

    let validator = GitObjectValidator::default();

    // Simulate a diff between two blobs
    let old_content = b"Line 1: Original\nLine 2: Original\nLine 3: Original\n";
    let new_content = b"Line 1: Original\nLine 2: Modified\nLine 3: Original\nLine 4: New line\n";

    let old_blob_contract = BlobContract::new("old123".to_string(), old_content.len());
    let new_blob_contract = BlobContract::new("new456".to_string(), new_content.len());

    let old_git_object = validator.validate_blob(&old_blob_contract, None);
    let new_git_object = validator.validate_blob(&new_blob_contract, None);

    println!("  Old Git Object: {}", old_git_object.summary());
    println!("  New Git Object: {}", new_git_object.summary());

    // Show the diff structure
    println!("  Diff Structure:");
    println!("    Pair of Git Object Contracts:");
    println!(
        "      - Old: {} ({} bytes)",
        old_git_object.oid, old_git_object.size
    );
    println!(
        "      - New: {} ({} bytes)",
        new_git_object.oid, new_git_object.size
    );

    // Validate the diff
    let old_valid = old_git_object.is_valid();
    let new_valid = new_git_object.is_valid();

    println!("  Diff Validation:");
    println!(
        "    Old blob: {}",
        if old_valid {
            "✅ Valid"
        } else {
            "❌ Invalid"
        }
    );
    println!(
        "    New blob: {}",
        if new_valid {
            "✅ Valid"
        } else {
            "❌ Invalid"
        }
    );

    println!();
    Ok(())
}
