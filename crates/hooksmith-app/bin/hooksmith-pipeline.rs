use clap::{Parser, Subcommand};
use git2::Repository;
use hooksmith_core::{
    git_pipeline::{ConcernSelector, GitPipeline, ToolFingerprint},
    object_names_validator::{load_contract, ObjectNamesValidator},
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hooksmith-pipeline")]
#[command(about = "Git pipeline for object names validation using four-actor pattern")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Resolve scope to get target commit and root tree
    Scope {
        /// Git reference to resolve
        #[arg(value_name = "REF")]
        ref_name: String,
    },

    /// Select objects based on concern selector
    Select {
        /// Git reference to analyze
        #[arg(value_name = "REF")]
        ref_name: String,

        /// Concern selector type
        #[arg(long, default_value = "root-names")]
        selector: String,
    },

    /// Run the complete validation pipeline
    Validate {
        /// Git reference to validate
        #[arg(value_name = "REF")]
        ref_name: String,

        /// Contract file path
        #[arg(long, default_value = "contracts/object-names@v1.json")]
        contract: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Scope { ref_name } => {
            let pipeline = GitPipeline::new(".")?;
            let scope = pipeline.resolve_scope(ref_name)?;

            println!("Scope resolved:");
            println!("  Commit OID: {}", scope.commit_oid);
            println!("  Root Tree OID: {}", scope.root_tree_oid);
            println!("  Selector Hash: {}", scope.selector_hash);
        }

        Commands::Select { ref_name, selector } => {
            let pipeline = GitPipeline::new(".")?;
            let scope = pipeline.resolve_scope(ref_name)?;

            let concern_selector = match selector.as_str() {
                "root-names" => ConcernSelector::RootNamesOnly,
                "pattern" => ConcernSelector::PathPattern("src/**/*.rs".to_string()),
                "depth" => ConcernSelector::TreeDepth(2),
                _ => ConcernSelector::Custom(selector.clone()),
            };

            let objects = pipeline.select_objects(&scope, &concern_selector)?;

            println!("Selected {} objects:", objects.len());
            for object in objects {
                println!(
                    "  {} ({:?}) - {}",
                    object.oid,
                    object.kind,
                    object
                        .logical_path
                        .as_ref()
                        .map(|p| p.to_string_lossy())
                        .unwrap_or_default()
                );
            }
        }

        Commands::Validate { ref_name, contract } => {
            let repo = Repository::open(".")?;

            // Load contract
            let contract_json = std::fs::read_to_string(contract)?;
            let contract = load_contract(&contract_json)?;

            // Create validator
            let validator = ObjectNamesValidator::new(contract);

            // Resolve the reference to get commit OID
            let pipeline = GitPipeline::new(".")?;
            let scope = pipeline.resolve_scope(ref_name)?;

            println!("🔍 Validating object-names contract for {}", ref_name);
            println!("Commit: {}", scope.commit_oid);
            println!("Root Tree: {}", scope.root_tree_oid);
            println!();

            // Validate the root tree
            let verdicts = validator.validate_root_tree(&repo, &scope.commit_oid)?;

            let mut all_passed = true;

            for verdict in verdicts {
                if verdict.pass {
                    println!("✅ {}: {}", verdict.contract_name, verdict.summary_code);
                } else {
                    println!("❌ {}: {}", verdict.contract_name, verdict.summary_code);
                    all_passed = false;

                    if let Some(diff_oid) = verdict.diff_oid {
                        println!("   Diff: {}", diff_oid);
                    }
                }
            }

            println!();
            if all_passed {
                println!("🎉 All validations passed!");
            } else {
                println!("💥 Some validations failed!");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
