use git_filter::filter::run_process_filter;
use git_filter::error::FilterError;
use tracing::info;

fn main() -> Result<(), FilterError> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting Git filter process");
    
    // Check if we're being run as a process filter
    if std::env::args().any(|arg| arg == "process") {
        info!("Running as Git process filter");
        run_process_filter()?;
    } else {
        // For testing, we can run in a different mode
        info!("Running in test mode");
        println!("Git filter binary - use with 'process' argument for Git integration");
    }
    
    Ok(())
} 
