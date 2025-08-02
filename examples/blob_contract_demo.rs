use git_filter::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🔧 Blob Contract Demo - Git Object Validation\n");

    // Example 1: Basic blob validation
    demo_basic_blob_validation()?;

    // Example 2: Blob contract with forbidden bytes
    demo_blob_with_forbidden_bytes()?;

    // Example 3: Line ending normalization
    demo_line_ending_normalization()?;

    // Example 4: UTF-8 validation
    demo_utf8_validation()?;

    // Example 5: Byte audit records
    demo_byte_audit_records()?;

    // Example 6: Binary heuristic detection
    demo_binary_heuristic()?;

    Ok(())
}

fn demo_basic_blob_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Example 1: Basic Blob Validation");

    let validator = BlobValidator::default();
    let content = b"Hello, World!\nThis is a valid text blob.\n";
    let oid = "abc123def456";

    let (contract, _processed, _) = validator.validate_blob(oid, content);

    println!("  {}", contract.summary());
    println!("    Size: {} bytes", contract.size);
    println!("    UTF-8 valid: {}", contract.valid_utf8);
    println!("    EOL normalized: {}", contract.normalized_eol);
    println!("    Has forbidden bytes: {}", contract.has_forbidden_byte);
    println!("    Action: {:?}", contract.action);

    println!();
    Ok(())
}

fn demo_blob_with_forbidden_bytes() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚫 Example 2: Blob with Forbidden Bytes");

    let validator = BlobValidator::default();
    let content = b"Hello\x00World\nThis has a NUL byte\x01and control chars";
    let oid = "def456ghi789";

    let (contract, _processed, _) = validator.validate_blob(oid, content);

    println!("  {}", contract.summary());
    println!("    Forbidden bytes: {}", contract.has_forbidden_byte);

    if let Some(positions) = &contract.forbidden_byte_positions {
        println!("    Forbidden byte positions: {:?}", positions);
    }

    println!("    Action: {:?}", contract.action);

    println!();
    Ok(())
}

fn demo_line_ending_normalization() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Example 3: Line Ending Normalization");

    let validator = BlobValidator::new(true, false, 30.0, false);
    let content = b"Line 1\r\nLine 2\nLine 3\rLine 4\r\nLine 5";
    let oid = "ghi789jkl012";

    let (contract, processed, _) = validator.validate_blob(oid, content);

    println!("  {}", contract.summary());
    println!("    Original: {:?}", String::from_utf8_lossy(content));
    println!("    Processed: {:?}", String::from_utf8_lossy(&processed));
    println!("    EOL normalized: {}", contract.normalized_eol);

    // Count line endings
    let original_lf = content.iter().filter(|&&b| b == b'\n').count();
    let original_cr = content.iter().filter(|&&b| b == b'\r').count();
    let processed_lf = processed.iter().filter(|&&b| b == b'\n').count();
    let processed_cr = processed.iter().filter(|&&b| b == b'\r').count();

    println!("    Original: {} LF, {} CR", original_lf, original_cr);
    println!("    Processed: {} LF, {} CR", processed_lf, processed_cr);

    println!();
    Ok(())
}

fn demo_utf8_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔤 Example 4: UTF-8 Validation");

    let validator = BlobValidator::default();

    // Valid UTF-8
    let valid_content = b"Hello, World!\nCaf\xE9 with UTF-8\n";
    let oid = "jkl012mno345";

    let (contract, _, _) = validator.validate_blob(oid, valid_content);
    println!("  Valid UTF-8: {}", contract.summary());

    // Invalid UTF-8
    let invalid_content = b"Hello\x80World\nInvalid UTF-8 sequence";
    let oid = "mno345pqr678";

    let (contract, _, _) = validator.validate_blob(oid, invalid_content);
    println!("  Invalid UTF-8: {}", contract.summary());

    println!();
    Ok(())
}

fn demo_byte_audit_records() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Example 5: Byte Audit Records");

    let validator = BlobValidator::new(true, false, 30.0, true); // Enable audit records
    let content = b"Hello\x00World\nCaf\xE9\x80\xC0";
    let oid = "pqr678stu901";

    let (contract, _, audit_records) = validator.validate_blob(oid, content);

    println!("  {}", contract.summary());
    println!("    Generated {} audit records", audit_records.len());

    // Show forbidden bytes
    let forbidden_audits: Vec<_> = audit_records.iter()
        .filter(|audit| !audit.allowed)
        .collect();

    if !forbidden_audits.is_empty() {
        println!("    Forbidden bytes:");
        for audit in &forbidden_audits {
            println!("      [{}] {} (0x{:02X})", audit.offset, audit.description(), audit.byte);
        }
    }

    // Show byte classification summary
    let mut class_counts = std::collections::HashMap::new();
    for audit in &audit_records {
        *class_counts.entry(&audit.class).or_insert(0) += 1;
    }

    println!("    Byte classification:");
    for (class, count) in class_counts {
        println!("      {:?}: {} bytes", class, count);
    }

    println!();
    Ok(())
}

fn demo_binary_heuristic() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Example 6: Binary Heuristic Detection");

    let validator = BlobValidator::new(false, true, 30.0, false);

    // Text file (low forbidden percentage)
    let text_content = b"Hello, World!\nThis is a text file.\n";
    let oid = "stu901vwx234";

    let (contract, _, _) = validator.validate_blob(oid, text_content);
    println!("  Text file: {}", contract.summary());

    // Binary-like file (high forbidden percentage)
    let binary_content = b"Hello\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F";
    let oid = "vwx234yza567";

    let (contract, _, _) = validator.validate_blob(oid, binary_content);
    println!("  Binary-like file: {}", contract.summary());

    // Test different thresholds
    println!("  Testing different thresholds:");
    for threshold in [10.0, 20.0, 30.0, 50.0] {
        let validator = BlobValidator::new(false, true, threshold, false);
        let (contract, _, _) = validator.validate_blob(oid, binary_content);
        
        let forbidden_percentage = if contract.size > 0 {
            (contract.forbidden_byte_positions.as_ref().map(|v| v.len()).unwrap_or(0) as f64 / contract.size as f64) * 100.0
        } else {
            0.0
        };
        
        println!("    {}% threshold: {} ({}% forbidden)", 
            threshold,
            if contract.is_accepted() { "Accept" } else { "Reject" },
            forbidden_percentage);
    }

    println!();
    Ok(())
} 
