use git_filter::prelude::*;
use git_filter::actions::GitOperation;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🔧 Combined Contract Demo - Blob + Line Level Validation\n");

    // Example 1: Basic combined validation
    demo_basic_combined_validation()?;

    // Example 2: Blob with mixed line issues
    demo_blob_with_mixed_line_issues()?;

    // Example 3: Line-by-line analysis
    demo_line_by_line_analysis()?;

    // Example 4: EOL normalization at line level
    demo_eol_normalization_at_line_level()?;

    // Example 5: UTF-8 validation per line
    demo_utf8_validation_per_line()?;

    // Example 6: Combined filter usage
    demo_combined_filter_usage()?;

    Ok(())
}

fn demo_basic_combined_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Example 1: Basic Combined Validation");

    let _validator = CombinedContractFilter::default();
    let content = b"Hello, World!\nThis is a valid text blob.\nLine 3 with normal content.\n";
    let oid = "abc123def456";

    // Simulate the filter process
    let blob_validator = BlobValidator::default();
    let line_validator = LineValidator::default();

    // Validate at blob level
    let (blob_contract, _processed_content, _) = blob_validator.validate_blob(oid, content);
    println!("  {}", blob_contract.summary());

    // Validate at line level
    let (line_contracts, _) = line_validator.validate_blob_lines(oid, content);
    let line_summary = line_validator.summarize_line_contracts(&line_contracts);
    println!("  {}", line_summary);

    // Show individual line contracts
    for line_contract in &line_contracts {
        println!("    {}", line_contract.summary());
    }

    println!();
    Ok(())
}

fn demo_blob_with_mixed_line_issues() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚫 Example 2: Blob with Mixed Line Issues");

    let content = b"Line 1 - valid\nLine 2 - has\x00NUL byte\nLine 3 - CRLF\r\nLine 4 - valid\n";
    let oid = "def456ghi789";

    let blob_validator = BlobValidator::default();
    let line_validator = LineValidator::default();

    // Validate at blob level
    let (blob_contract, _, _) = blob_validator.validate_blob(oid, content);
    println!("  {}", blob_contract.summary());

    // Validate at line level
    let (line_contracts, _) = line_validator.validate_blob_lines(oid, content);
    let line_summary = line_validator.summarize_line_contracts(&line_contracts);
    println!("  {}", line_summary);

    // Show individual line contracts
    for line_contract in &line_contracts {
        println!("    {}", line_contract.summary());
    }

    // Count issues by type
    let rejected_lines: Vec<_> = line_contracts.iter()
        .filter(|c| c.is_rejected())
        .collect();
    let fixed_lines: Vec<_> = line_contracts.iter()
        .filter(|c| c.needs_fixing())
        .collect();

    println!("  Issues found:");
    println!("    Rejected lines: {}", rejected_lines.len());
    println!("    Lines needing fixing: {}", fixed_lines.len());

    println!();
    Ok(())
}

fn demo_line_by_line_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Example 3: Line-by-Line Analysis");

    let content = b"Line 1: Normal text\nLine 2: Has\x01control char\nLine 3: CRLF\r\nLine 4: UTF-8 caf\xE9\nLine 5: Normal again\n";
    let oid = "ghi789jkl012";

    let line_validator = LineValidator::default();
    let (line_contracts, processed_content) = line_validator.validate_blob_lines(oid, content);

    println!("  Original content:");
    println!("    {:?}", String::from_utf8_lossy(content));
    println!("  Processed content:");
    println!("    {:?}", String::from_utf8_lossy(&processed_content));

    println!("  Line-by-line analysis:");
    for line_contract in &line_contracts {
        let status = match line_contract.action {
            LineAction::Accept => "✅",
            LineAction::Reject => "❌",
            LineAction::Fix => "🔧",
        };
        println!("    {} Line {}: {} (offset: {}, length: {})", 
            status, 
            line_contract.line_number, 
            line_contract.summary(),
            line_contract.byte_offset,
            line_contract.length
        );
    }

    println!();
    Ok(())
}

fn demo_eol_normalization_at_line_level() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Example 4: EOL Normalization at Line Level");

    let content = b"Line 1: LF only\nLine 2: CRLF\r\nLine 3: CR only\rLine 4: Mixed\r\nLine 5: LF again\n";
    let oid = "jkl012mno345";

    let line_validator = LineValidator::new(true, false, false); // Normalize, don't allow mixed
    let (line_contracts, processed_content) = line_validator.validate_blob_lines(oid, content);

    println!("  Original content:");
    println!("    {:?}", String::from_utf8_lossy(content));
    println!("  Processed content:");
    println!("    {:?}", String::from_utf8_lossy(&processed_content));

    println!("  Line EOL analysis:");
    for line_contract in &line_contracts {
        let eol_status = if line_contract.normalized_eol { "✅" } else { "❌" };
        let action_status = match line_contract.action {
            LineAction::Accept => "Accept",
            LineAction::Reject => "Reject",
            LineAction::Fix => "Fix",
        };
        println!("    {} Line {}: EOL {} -> {}", 
            eol_status, 
            line_contract.line_number, 
            if line_contract.normalized_eol { "normalized" } else { "mixed" },
            action_status
        );
    }

    println!();
    Ok(())
}

fn demo_utf8_validation_per_line() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔤 Example 5: UTF-8 Validation Per Line");

    let content = b"Line 1: Valid UTF-8\nLine 2: Invalid\x80UTF-8\nLine 3: Valid again\nLine 4: Another\xFFinvalid\n";
    let oid = "mno345pqr678";

    let line_validator = LineValidator::default();
    let (line_contracts, _) = line_validator.validate_blob_lines(oid, content);

    println!("  UTF-8 validation per line:");
    for line_contract in &line_contracts {
        let utf8_status = if line_contract.valid_utf8 { "✅" } else { "❌" };
        let action_status = match line_contract.action {
            LineAction::Accept => "Accept",
            LineAction::Reject => "Reject",
            LineAction::Fix => "Fix",
        };
        println!("    {} Line {}: UTF-8 {} -> {}", 
            utf8_status, 
            line_contract.line_number, 
            if line_contract.valid_utf8 { "valid" } else { "invalid" },
            action_status
        );
    }

    // Count UTF-8 issues
    let invalid_utf8_lines: Vec<_> = line_contracts.iter()
        .filter(|c| !c.valid_utf8)
        .collect();

    println!("  UTF-8 issues found: {} lines", invalid_utf8_lines.len());

    println!();
    Ok(())
}

fn demo_combined_filter_usage() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Example 6: Combined Filter Usage");

    // Create a combined filter with custom settings
    let filter = CombinedContractFilter::new(
        true,   // normalize_line_endings
        true,   // apply_binary_heuristic
        30.0,   // binary_threshold
        false,  // allow_mixed_eol
        true,   // generate_line_contracts
    );

    let content = b"Line 1: Valid content\nLine 2: Has\x00NUL byte\nLine 3: CRLF\r\nLine 4: Valid again\n";
    let file_state = FileState::default();
    let operation = GitOperation::Add;

    println!("  Processing content with combined filter:");
    println!("    Original: {:?}", String::from_utf8_lossy(content));

    // Simulate the filter process
    match filter.process(content, &file_state, &operation) {
        Ok(processed) => {
            println!("    ✅ Processing successful");
            println!("    Processed: {:?}", String::from_utf8_lossy(&processed));
        }
        Err(e) => {
            println!("    ❌ Processing failed: {}", e);
        }
    }

    println!();
    Ok(())
} 
