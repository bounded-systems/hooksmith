use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = ".")] 
    root: std::path::PathBuf,
    #[arg(long, default_value = "HEAD")] 
    r#ref: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let officer = hooksmith_validate::TriageOfficer::new(&args.root)?;
    std::process::exit(officer.run(&args.r#ref)?);
}
