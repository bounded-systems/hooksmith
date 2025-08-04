use git_filter::actions::GitOperation;
use git_filter::prelude::*;
use std::io::{Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Read from stdin
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input)?;

    // Create blob contract filter
    let filter = BlobContractFilter::new(
        true,  // normalize_line_endings
        true,  // apply_binary_heuristic
        30.0,  // binary_threshold
        false, // generate_audit (set to true for detailed logging)
    );

    // Create a default file state
    let file_state = FileState::default();
    let operation = GitOperation::Add;

    // Process the content
    match filter.process(&input, &file_state, &operation) {
        Ok(processed) => {
            // Write processed content to stdout
            std::io::stdout().write_all(&processed)?;
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
