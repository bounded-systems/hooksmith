use clap::{Parser, Subcommand};
use git2::{ObjectType, Repository, Tree};
use serde_json::json;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hooksmith-scope")]
#[command(about = "Git scope resolver for Hooksmith pipeline")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Resolve a Git reference to get commit OID
    Resolve {
        /// Git reference to resolve
        #[arg(value_name = "REF")]
        ref_name: String,
    },

    /// List objects based on concern selector
    Ls {
        /// Commit OID to analyze
        #[arg(long, value_name = "COMMIT_OID")]
        commit: String,

        /// Concern selector type
        #[arg(long, default_value = "root-names")]
        selector: String,
    },
}

/// Object descriptor for NDJSON output
#[derive(Debug, serde::Serialize)]
struct ObjectDescriptor {
    oid: String,
    kind: String,
    logical_path: Option<String>,
    parent_tree_oid: Option<String>,
    size: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Resolve { ref_name } => {
            let repo = Repository::open(".")?;
            let commit = repo.find_reference(ref_name)?.peel_to_commit()?;

            // Output just the commit OID
            println!("{}", commit.id());
        }

        Commands::Ls { commit, selector } => {
            let repo = Repository::open(".")?;
            let commit_oid = git2::Oid::from_str(commit)?;
            let commit = repo.find_commit(commit_oid)?;
            let tree = commit.tree()?;

            let objects = match selector.as_str() {
                "root-names" => select_root_names(&repo, &tree)?,
                "pattern" => select_by_pattern(&repo, &tree, "src/**/*.rs")?,
                "depth" => select_by_depth(&repo, &tree, 2)?,
                _ => return Err(format!("Unknown selector: {}", selector).into()),
            };

            // Output NDJSON stream to stdout
            let stdout = io::stdout();
            let mut handle = stdout.lock();

            for object in objects {
                let json = serde_json::to_string(&object)?;
                writeln!(handle, "{}", json)?;
            }
        }
    }

    Ok(())
}

fn select_root_names(
    repo: &Repository,
    tree: &Tree,
) -> Result<Vec<ObjectDescriptor>, Box<dyn std::error::Error>> {
    let mut objects = Vec::new();

    for entry in tree.iter() {
        let object = ObjectDescriptor {
            oid: entry.id().to_string(),
            kind: format!("{:?}", entry.kind()),
            logical_path: Some(entry.name().unwrap_or("").to_string()),
            parent_tree_oid: Some(tree.id().to_string()),
            size: entry.id().as_bytes().len(),
        };
        objects.push(object);
    }

    Ok(objects)
}

fn select_by_pattern(
    repo: &Repository,
    tree: &Tree,
    pattern: &str,
) -> Result<Vec<ObjectDescriptor>, Box<dyn std::error::Error>> {
    // TODO: Implement glob pattern matching
    // For now, just return root names
    select_root_names(repo, tree)
}

fn select_by_depth(
    repo: &Repository,
    tree: &Tree,
    depth: usize,
) -> Result<Vec<ObjectDescriptor>, Box<dyn std::error::Error>> {
    // TODO: Implement depth-based tree walking
    // For now, just return root names
    select_root_names(repo, tree)
}
