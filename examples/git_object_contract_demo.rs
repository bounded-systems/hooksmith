use git_filter::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🔧 Git Object Contract Demo - Discriminated Union System\n");

    // Example 1: Basic blob contract
    demo_basic_blob_contract()?;

    // Example 2: Blob with invalid UTF-8
    demo_blob_with_invalid_utf8()?;

    // Example 3: Line contracts
    demo_line_contracts()?;

    // Example 4: Chunk contracts (diff hunks)
    demo_chunk_contracts()?;

    // Example 5: Complete Git object validation
    demo_complete_git_object_validation()?;

    // Example 6: Diff modeling
    demo_diff_modeling()?;

    Ok(())
}

fn demo_basic_blob_contract() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Example 1: Basic Blob Contract");

    let validator = GitObjectValidator::default();
    let content = b"Hello, World!\nThis is a test file.\nLine 3 with content.\n";
    let blob = validator.validate_blob("abc123def456", content);

    println!("  {}", blob.summary());
    println!("    ID: {}", blob.id);
    println!("    Size: {} bytes", blob.size);
    println!("    Encoding: {}", blob.encoding);
    println!("    Lines: {}", blob.lines.len());
    println!("    Valid: {}", blob.is_valid());

    // Show individual lines
    for (i, line) in blob.lines.iter().enumerate() {
        println!("    Line {}: {:?}", i + 1, line);
    }

    println!();
    Ok(())
}

fn demo_blob_with_invalid_utf8() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚫 Example 2: Blob with Invalid UTF-8");

    let validator = GitObjectValidator::default();
    let content = b"Hello\x80World\nInvalid UTF-8 sequence";
    let blob = validator.validate_blob("def456ghi789", content);

    println!("  {}", blob.summary());
    println!("    Encoding: {}", blob.encoding);
    println!("    Valid: {}", blob.is_valid());
    println!("    Errors: {:?}", blob.errors);

    println!();
    Ok(())
}

fn demo_line_contracts() -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Example 3: Line Contracts");

    let validator = GitObjectValidator::default();
    let content = b"Line 1: Normal text\nLine 2: Has\x00NUL byte\nLine 3: CRLF\r\nLine 4: Valid again\n";
    let blob = validator.validate_blob("ghi789jkl012", content);
    let lines = validator.validate_blob_lines(&blob);

    println!("  Blob: {}", blob.summary());
    println!("  Line contracts:");

    for line in &lines {
        println!("    {}", line.summary());
        if !line.errors.is_empty() {
            println!("      Errors: {:?}", line.errors);
        }
    }

    // Count by action type
    let accepted = lines.iter().filter(|l| l.is_valid()).count();
    let rejected = lines.iter().filter(|l| l.is_rejected()).count();
    let needs_fixing = lines.iter().filter(|l| l.needs_fixing()).count();

    println!("  Summary: {} accepted, {} rejected, {} need fixing", accepted, rejected, needs_fixing);

    println!();
    Ok(())
}

fn demo_chunk_contracts() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔀 Example 4: Chunk Contracts (Diff Hunks)");

    let validator = GitObjectValidator::default();

    // Create a diff chunk representing changes
    let diff_lines = vec![
        (DiffLineType::Context, "Line 1: Unchanged".to_string()),
        (DiffLineType::Context, "Line 2: Also unchanged".to_string()),
        (DiffLineType::Remove, "Line 3: This line was removed".to_string()),
        (DiffLineType::Add, "Line 3: This line was added".to_string()),
        (DiffLineType::Add, "Line 4: Another added line".to_string()),
        (DiffLineType::Context, "Line 5: Back to unchanged".to_string()),
    ];

    let chunk = validator.create_chunk_contract(
        "@@ -1,3 +1,4 @@",
        1,  // old_start
        3,  // old_lines
        1,  // new_start
        4,  // new_lines
        diff_lines,
    );

    println!("  {}", chunk.summary());
    println!("    Header: {}", chunk.header);
    println!("    Old: {} lines starting at {}", chunk.old_lines, chunk.old_start);
    println!("    New: {} lines starting at {}", chunk.new_lines, chunk.new_start);
    println!("    Total lines in chunk: {}", chunk.lines.len());

    // Show individual diff lines
    for (i, line) in chunk.lines.iter().enumerate() {
        let type_symbol = match line.line_type {
            DiffLineType::Context => " ",
            DiffLineType::Add => "+",
            DiffLineType::Remove => "-",
        };
        println!("    {} {}: {:?} {}", type_symbol, i + 1, line.content, if line.valid { "✅" } else { "❌" });
    }

    println!();
    Ok(())
}

fn demo_complete_git_object_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Example 5: Complete Git Object Validation");

    let validator = GitObjectValidator::new(true, true); // Enable both line and chunk validation
    let content = b"Line 1: Valid content\nLine 2: Has\x01control char\nLine 3: CRLF\r\nLine 4: Valid again\n";
    
    // Validate as a complete Git object
    let git_object = validator.validate_git_object("mno345pqr678", content);
    
    match git_object {
        GitObjectContract::Blob(blob) => {
            println!("  Git Object Type: Blob");
            println!("  {}", blob.summary());
            
            // Validate lines
            let lines = validator.validate_blob_lines(&blob);
            let summary = validator.summarize_validation(&blob, &lines);
            println!("  {}", summary);
            
            // Show line details
            for line in &lines {
                let status = match line.action {
                    GitLineAction::Accept => "✅",
                    GitLineAction::Reject => "❌",
                    GitLineAction::Fix => "🔧",
                };
                println!("    {} {}", status, line.summary());
            }
        }
    }

    println!();
    Ok(())
}

fn demo_diff_modeling() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Example 6: Diff Modeling");

    let validator = GitObjectValidator::default();

    // Simulate a diff between two blobs
    let old_content = b"Line 1: Original\nLine 2: Original\nLine 3: Original\n";
    let new_content = b"Line 1: Original\nLine 2: Modified\nLine 3: Original\nLine 4: New line\n";

    let old_blob = validator.validate_blob("old123", old_content);
    let new_blob = validator.validate_blob("new456", new_content);

    println!("  Old Blob: {}", old_blob.summary());
    println!("  New Blob: {}", new_blob.summary());

    // Create a chunk representing the diff
    let diff_lines = vec![
        (DiffLineType::Context, "Line 1: Original".to_string()),
        (DiffLineType::Remove, "Line 2: Original".to_string()),
        (DiffLineType::Add, "Line 2: Modified".to_string()),
        (DiffLineType::Context, "Line 3: Original".to_string()),
        (DiffLineType::Add, "Line 4: New line".to_string()),
    ];

    let chunk = validator.create_chunk_contract(
        "@@ -1,3 +1,4 @@",
        1,  // old_start
        3,  // old_lines
        1,  // new_start
        4,  // new_lines
        diff_lines,
    );

    println!("  Diff Chunk: {}", chunk.summary());

    // Show the diff structure
    println!("  Diff Structure:");
    println!("    Pair of Blob Contracts:");
    println!("      - Old: {} ({} lines)", old_blob.id, old_blob.lines.len());
    println!("      - New: {} ({} lines)", new_blob.id, new_blob.lines.len());
    println!("    Array of Chunk Contracts:");
    println!("      - Chunk: {} ({} lines)", chunk.header, chunk.lines.len());

    // Validate the diff
    let old_lines = validator.validate_blob_lines(&old_blob);
    let new_lines = validator.validate_blob_lines(&new_blob);

    let old_valid = old_lines.iter().all(|l| l.is_valid());
    let new_valid = new_lines.iter().all(|l| l.is_valid());
    let chunk_valid = chunk.is_valid();

    println!("  Diff Validation:");
    println!("    Old blob lines: {}", if old_valid { "✅ Valid" } else { "❌ Invalid" });
    println!("    New blob lines: {}", if new_valid { "✅ Valid" } else { "❌ Invalid" });
    println!("    Chunk: {}", if chunk_valid { "✅ Valid" } else { "❌ Invalid" });

    println!();
    Ok(())
} 
