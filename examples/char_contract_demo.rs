use git_filter::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🔧 Character Contract Demo - Dev Contract System\n");

    // Example 1: Character classification
    demo_character_classification()?;

    // Example 2: File validation with contracts
    demo_file_validation()?;

    // Example 3: Detailed character analysis
    demo_character_analysis()?;

    // Example 4: Line ending normalization
    demo_line_ending_normalization()?;

    // Example 5: Binary heuristic detection
    demo_binary_heuristic()?;

    Ok(())
}

fn demo_character_classification() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Example 1: Character Classification");

    let test_bytes = vec![
        b'A',  // ASCII printable
        b'\n', // Safe control (LF)
        b'\t', // Safe control (TAB)
        0x00,  // Forbidden (NUL)
        0x80,  // UTF-8 continuation
        0xC0,  // UTF-8 lead
        0xFF,  // Forbidden (invalid UTF-8)
    ];

    for &byte in &test_bytes {
        let contract = CharacterContract::from_byte(byte);
        println!(
            "  0x{:02X}: {} -> {}",
            byte,
            contract.description(),
            if contract.is_allowed() {
                "✅ Allowed"
            } else {
                "❌ Forbidden"
            }
        );
    }

    println!();
    Ok(())
}

fn demo_file_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Example 2: File Validation with Contracts");

    let validator = CharValidator::default();

    // Valid file
    let valid_content = b"Hello, World!\nThis is valid text with UTF-8: caf\xE9\n";
    let (result, _processed) = validator.validate_file(valid_content);

    println!("  Valid file: {}", result.summary());
    println!("    UTF-8 valid: {}", result.utf8_valid);
    println!(
        "    Line endings normalized: {}",
        result.line_endings_normalized
    );
    println!("    Forbidden characters: {}", result.forbidden_count);

    // Invalid file with forbidden character
    let invalid_content = b"Hello\x00World\nThis has a NUL byte";
    let (result, _) = validator.validate_file(invalid_content);

    println!("  Invalid file: {}", result.summary());
    if let Some(forbidden) = &result.first_forbidden {
        println!("    First forbidden: {}", forbidden.description());
    }

    println!();
    Ok(())
}

fn demo_character_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Example 3: Detailed Character Analysis");

    let validator = CharValidator::default();
    let content = b"Hello\x00World\nCaf\xE9\x80\xC0";

    let contracts = validator.analyze_file(content);

    println!("  Character-by-character analysis:");
    for (i, contract) in contracts.iter().enumerate() {
        let status = if contract.is_allowed() { "✅" } else { "❌" };
        println!(
            "    [{}] {} {} (0x{:02X})",
            i,
            status,
            contract.description(),
            contract.byte
        );
    }

    println!();
    Ok(())
}

fn demo_line_ending_normalization() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Example 4: Line Ending Normalization");

    let validator = CharValidator::new(true, false, 30.0); // Only normalize, no binary heuristic

    let mixed_content = b"Line 1\r\nLine 2\nLine 3\rLine 4\r\nLine 5";
    let (result, processed) = validator.validate_file(mixed_content);

    println!("  Original: {:?}", String::from_utf8_lossy(mixed_content));
    println!("  Normalized: {:?}", String::from_utf8_lossy(&processed));
    println!(
        "  Line endings normalized: {}",
        result.line_endings_normalized
    );

    // Count line endings
    let original_lf = mixed_content.iter().filter(|&&b| b == b'\n').count();
    let original_cr = mixed_content.iter().filter(|&&b| b == b'\r').count();
    let processed_lf = processed.iter().filter(|&&b| b == b'\n').count();
    let processed_cr = processed.iter().filter(|&&b| b == b'\r').count();

    println!("  Original: {original_lf} LF, {original_cr} CR");
    println!("  Processed: {processed_lf} LF, {processed_cr} CR");

    println!();
    Ok(())
}

fn demo_binary_heuristic() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Example 5: Binary Heuristic Detection");

    let validator = CharValidator::new(false, true, 30.0); // Only binary heuristic, no normalization

    // Text file (low forbidden percentage)
    let text_content = b"Hello, World!\nThis is a text file.\n";
    let (result, _) = validator.validate_file(text_content);
    println!(
        "  Text file: {} ({}% forbidden)",
        if result.is_binary_heuristic() {
            "❌ Binary"
        } else {
            "✅ Text"
        },
        result.forbidden_percentage
    );

    // Binary-like file (high forbidden percentage)
    let binary_content = b"Hello\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F";
    let (result, _) = validator.validate_file(binary_content);
    println!(
        "  Binary-like file: {} ({}% forbidden)",
        if result.is_binary_heuristic() {
            "❌ Binary"
        } else {
            "✅ Text"
        },
        result.forbidden_percentage
    );

    // Test different thresholds
    println!("  Testing different thresholds:");
    for threshold in [10.0, 20.0, 30.0, 50.0] {
        let validator = CharValidator::new(false, true, threshold);
        let (result, _) = validator.validate_file(binary_content);
        println!(
            "    {}% threshold: {} ({}% forbidden)",
            threshold,
            if result.is_binary_heuristic() {
                "Binary"
            } else {
                "Text"
            },
            result.forbidden_percentage
        );
    }

    println!();
    Ok(())
}
