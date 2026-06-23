use anyhow::Result;
use serde_json::Value;
use validate_object_names_contract::ScopeRefManager;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --bin scope_ref_manager <command> [args...]");
        println!();
        println!("Commands:");
        println!("  set <scope> <commit>                    - Set scope ref to commit");
        println!("  get <scope>                             - Get scope ref commit SHA");
        println!("  list                                    - List all scope refs");
        println!("  delete <scope>                          - Delete scope ref");
        println!("  metadata <scope>                        - Get scope metadata");
        println!("  set-metadata <scope> <json>             - Set scope metadata");
        println!("  needs-validation <scope> <commit>       - Check if scope needs validation");
        println!("  mark-validated <scope> <commit> <contracts> - Mark scope as validated");
        println!("  init <base-commit>                      - Initialize project scopes");
        println!("  export                                  - Export scope refs as JSON");
        println!("  import <json-file>                      - Import scope refs from JSON");
        std::process::exit(1);
    }

    let command = &args[1];
    let repo_path = ".";

    let ref_manager = ScopeRefManager::new(repo_path)?;

    match command.as_str() {
        "set" => {
            if args.len() < 4 {
                eprintln!("Error: set command requires scope and commit");
                std::process::exit(1);
            }
            ref_manager.set_scope_ref(&args[2], &args[3])?;
        }
        "get" => {
            if args.len() < 3 {
                eprintln!("Error: get command requires scope");
                std::process::exit(1);
            }
            match ref_manager.get_scope_ref(&args[2])? {
                Some(sha) => println!("{}", sha),
                None => println!("Scope ref not found: {}", &args[2]),
            }
        }
        "list" => {
            let scope_refs = ref_manager.list_scope_refs()?;
            println!("{}", serde_json::to_string_pretty(&scope_refs)?);
        }
        "delete" => {
            if args.len() < 3 {
                eprintln!("Error: delete command requires scope");
                std::process::exit(1);
            }
            ref_manager.delete_scope_ref(&args[2])?;
        }
        "metadata" => {
            if args.len() < 3 {
                eprintln!("Error: metadata command requires scope");
                std::process::exit(1);
            }
            let metadata = ref_manager.get_scope_metadata(&args[2])?;
            println!("{}", serde_json::to_string_pretty(&metadata)?);
        }
        "set-metadata" => {
            if args.len() < 4 {
                eprintln!("Error: set-metadata command requires scope and JSON");
                std::process::exit(1);
            }
            let metadata: Value = serde_json::from_str(&args[3])?;
            ref_manager.set_scope_metadata(&args[2], &metadata)?;
        }
        "needs-validation" => {
            if args.len() < 4 {
                eprintln!("Error: needs-validation command requires scope and commit");
                std::process::exit(1);
            }
            let needs = ref_manager.scope_needs_validation(&args[2], &args[3])?;
            println!("{}", needs);
        }
        "mark-validated" => {
            if args.len() < 5 {
                eprintln!("Error: mark-validated command requires scope, commit, and contracts");
                std::process::exit(1);
            }
            let contracts: Vec<String> = serde_json::from_str(&args[4])?;
            ref_manager.update_scope_after_validation(&args[2], &args[3], &contracts, None)?;
        }
        "init" => {
            if args.len() < 3 {
                eprintln!("Error: init command requires base commit");
                std::process::exit(1);
            }
            ref_manager.initialize_project_scopes(&args[2])?;
        }
        "export" => {
            let export = ref_manager.export_scope_refs()?;
            println!("{}", serde_json::to_string_pretty(&export)?);
        }
        "import" => {
            if args.len() < 3 {
                eprintln!("Error: import command requires JSON file");
                std::process::exit(1);
            }
            let import_data: Value = serde_json::from_str(&std::fs::read_to_string(&args[2])?)?;
            ref_manager.import_scope_refs(&import_data)?;
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(1);
        }
    }

    Ok(())
}
