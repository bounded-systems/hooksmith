use git_filter::prelude::*;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("🔧 Git Filter Demo - Attributes as Hooks\n");
    
    // Example 1: Basic safe-ascii validation
    demo_safe_ascii_validation()?;
    
    // Example 2: EOL normalization
    demo_eol_normalization()?;
    
    // Example 3: Custom attribute mapping
    demo_custom_attributes()?;
    
    // Example 4: Action resolution
    demo_action_resolution()?;
    
    Ok(())
}

fn demo_safe_ascii_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Example 1: Safe ASCII Validation");
    
    let mut attributes = HashMap::new();
    attributes.insert("safe-ascii".to_string(), Some("true"));
    
    let file_state = FileState::from_attributes(&attributes);
    let operation = GitOperation::Add;
    
    // Valid content
    let valid_content = b"Hello, World!\nThis is valid ASCII content.\n";
    
    // Invalid content (contains non-ASCII)
    let invalid_content = b"Hello, World!\nThis contains \x80 invalid bytes.\n";
    
    let mut filter = MultiFilter::new();
    filter.add_driver("safe-ascii", Box::new(SafeAsciiFilter::default()));
    
    // Test valid content
    match filter.process_file(valid_content, &file_state, &operation) {
        Ok(_) => println!("✅ Valid content passed"),
        Err(e) => println!("❌ Valid content failed: {}", e),
    }
    
    // Test invalid content
    match filter.process_file(invalid_content, &file_state, &operation) {
        Ok(_) => println!("❌ Invalid content should have failed"),
        Err(e) => println!("✅ Invalid content correctly rejected: {}", e),
    }
    
    println!();
    Ok(())
}

fn demo_eol_normalization() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Example 2: EOL Normalization");
    
    let mut attributes = HashMap::new();
    attributes.insert("text".to_string(), Some("true"));
    attributes.insert("eol".to_string(), Some("lf"));
    
    let file_state = FileState::from_attributes(&attributes);
    let operation = GitOperation::Add;
    
    // Content with mixed line endings
    let mixed_content = b"Line 1\r\nLine 2\nLine 3\rLine 4\n";
    
    let mut filter = MultiFilter::new();
    
    match filter.process_file(mixed_content, &file_state, &operation) {
        Ok(normalized) => {
            println!("✅ EOL normalization successful");
            println!("   Original length: {} bytes", mixed_content.len());
            println!("   Normalized length: {} bytes", normalized.len());
            
            // Count line endings in normalized content
            let lf_count = normalized.iter().filter(|&&b| b == b'\n').count();
            let cr_count = normalized.iter().filter(|&&b| b == b'\r').count();
            println!("   LF count: {}, CR count: {}", lf_count, cr_count);
        }
        Err(e) => println!("❌ EOL normalization failed: {}", e),
    }
    
    println!();
    Ok(())
}

fn demo_custom_attributes() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Example 3: Custom Attribute Mapping");
    
    let mut resolver = ActionResolver::new();
    
    // Add a custom attribute mapping
    resolver.add_attribute_mapping(
        "no-trailing-whitespace",
        vec![HookAction::Custom {
            name: "remove_trailing_whitespace".to_string(),
            parameters: HashMap::new(),
        }],
        true,
    );
    
    let mut attributes = HashMap::new();
    attributes.insert("no-trailing-whitespace".to_string(), Some("true"));
    attributes.insert("safe-ascii".to_string(), Some("true"));
    
    let file_state = FileState::from_attributes(&attributes);
    let operation = GitOperation::Add;
    
    let actions = resolver.resolve_actions(&file_state, &operation);
    
    println!("   Resolved actions:");
    for action in &actions {
        println!("   - {}", action.description());
    }
    
    println!();
    Ok(())
}

fn demo_action_resolution() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎭 Example 4: Action Resolution by Operation");
    
    let mut attributes = HashMap::new();
    attributes.insert("filter".to_string(), Some("safe-ascii"));
    attributes.insert("diff".to_string(), Some("custom-diff"));
    attributes.insert("merge".to_string(), Some("custom-merge"));
    attributes.insert("export-ignore".to_string(), Some("true"));
    
    let file_state = FileState::from_attributes(&attributes);
    let resolver = ActionResolver::new();
    
    let operations = vec![
        GitOperation::Add,
        GitOperation::Checkout,
        GitOperation::Diff,
        GitOperation::Merge,
        GitOperation::Archive,
    ];
    
    for operation in operations {
        let actions = resolver.resolve_actions(&file_state, &operation);
        println!("   {:?}: {} actions", operation, actions.len());
        
        for action in &actions {
            println!("     - {}", action.description());
        }
    }
    
    println!();
    Ok(())
} 