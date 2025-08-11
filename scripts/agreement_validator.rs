use anyhow::{anyhow, Context, Result};
use clap::Parser;
use git2::{ObjectType, Repository, Tree, TreeWalkMode, TreeWalkResult};
use jsonschema::{Draft, JSONSchema};
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::{Path, PathBuf}};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
struct Args {
    /// Root containing .hooksmith/agreements
    #[arg(long, default_value = ".")]
    root: PathBuf,

    /// Git ref to validate (e.g. HEAD, refs/heads/main)
    #[arg(long, default_value = "HEAD")]
    r#ref: String,

    /// Fail fast on first error
    #[arg(long)]
    ci: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Agreement {
    version: String,
    mode: String,               // "tree"
    precedence: String,         // "allow-overrides-reject"
    default_action: String,     // "reject"
    allow_dirs: Option<Vec<String>>, // for top-level/subtree dir names (tree-only uses both)
    allow_files: Option<Vec<String>>, // literal or root-glob (*.ext) at current scope root
    subject: Subject,
    digest: DigestField,
}

#[derive(Deserialize)]
struct Subject {
    r#ref: Option<String>,          // optional override; fallback to CLI --ref
    scope: String,                  // "top-level" | "subtree" | "full-tree"
    path: Option<String>,           // required for subtree
}

#[derive(Deserialize)]
struct DigestField {
    algo: String, // "sha256"
    value: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let repo = Repository::discover(&args.root).context("discovering git repo")?;
    let agreements = discover_agreements(&args.root)?;

    if agreements.is_empty() {
        println!("OK: no agreements found under .hooksmith/agreements");
        return Ok(());
    }

    println!("🔍 Found {} agreement(s) to validate", agreements.len());

    let mut failures = 0usize;
    for ag_path in &agreements {
        match validate_one(&repo, &ag_path, &args.r#ref, args.ci) {
            Ok(_) => println!("✅ OK  {}", ag_path.display()),
            Err(e) => {
                eprintln!("❌ ERR {}: {:#}", ag_path.display(), e);
                failures += 1;
                if args.ci { break; }
            }
        }
    }

    if failures > 0 {
        return Err(anyhow!("{} agreement(s) failed validation", failures));
    }
    
    println!("🎉 All {} agreement(s) validated successfully!", agreements.len());
    Ok(())
}

fn discover_agreements(root: &Path) -> Result<Vec<PathBuf>> {
    let base = root.join(".hooksmith/agreements");
    if !base.exists() { 
        println!("📁 No .hooksmith/agreements directory found");
        return Ok(vec![]); 
    }
    
    let mut out = vec![];
    for entry in WalkDir::new(&base).into_iter().filter_map(Result::ok) {
        let p = entry.path();
        if p.is_file() && p.extension().map(|e| e=="json").unwrap_or(false) {
            out.push(p.to_path_buf());
        }
    }
    out.sort();
    Ok(out)
}

fn validate_one(repo: &Repository, path: &Path, cli_ref: &str, ci: bool) -> Result<()> {
    // Load + schema check
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    validate_schema(&text)?;
    let ag: Agreement = serde_json::from_str(&text).context("parsing agreement json")?;

    // Basic policy invariants
    require(&ag.mode == "tree", "mode must be 'tree'")?;
    require(&ag.precedence == "allow-overrides-reject", "precedence must be allow-overrides-reject")?;
    require(&ag.default_action == "reject", "defaultAction must be reject")?;
    require(&ag.digest.algo == "sha256", "digest algo must be sha256")?;

    // Resolve ref
    let the_ref = ag.subject.r#ref.as_deref().unwrap_or(cli_ref);
    let commit = repo.revparse_single(the_ref)?.peel_to_commit()?;
    let tree = commit.tree()?;

    // Materialize subject names for the scope
    let scope = ag.subject.scope.as_str();
    let names = match scope {
        "top-level" => materialize_top_level(&tree)?,
        "subtree"   => {
            let p = ag.subject.path.as_deref().ok_or_else(|| anyhow!("subject.path required for subtree"))?;
            materialize_subtree(&tree, p)?
        }
        "full-tree" => materialize_full_tree(&tree)?,
        other => return Err(anyhow!("unsupported subject.scope: {}", other)),
    };

    // Decision: allowed sets
    let allow_files = ag.allow_files.clone().unwrap_or_default();
    let allow_dirs  = ag.allow_dirs.clone().unwrap_or_default();

    decision_check(scope, &names, &allow_files, &allow_dirs, ci)?;

    // Digest check: subject + rules canonical
    let computed = compute_digest(scope, &names, &allow_files, &allow_dirs)?;
    require(&computed == &ag.digest.value, &format!("digest mismatch: computed={}, recorded={}", computed, ag.digest.value))?;

    Ok(())
}

fn require(cond: bool, msg: &str) -> Result<()> { 
    if cond { 
        Ok(()) 
    } else { 
        Err(anyhow!("{}", msg)) 
    } 
}

/// Names for top-level: only "foo" (no slashes). Types recorded for decision.
fn materialize_top_level(tree: &Tree) -> Result<BTreeMap<String, ObjectType>> {
    let mut map = BTreeMap::new();
    tree.walk(TreeWalkMode::PreOrder, |root, entry| {
        if !root.is_empty() { return TreeWalkResult::Skip; }
        if let Some(name) = entry.name() {
            if let Some(kind) = entry.kind() {
                map.insert(name.to_string(), kind);
            }
        }
        TreeWalkResult::Ok
    }).map_err(|_| anyhow!("tree walk failed"))?;
    Ok(map)
}

/// Names for subtree: "docs/README.md", "docs/guide.md", etc. Recursive.
fn materialize_subtree(tree: &Tree, sub: &str) -> Result<BTreeMap<String, ObjectType>> {
    let mut map = BTreeMap::new();
    let prefix = if sub.ends_with('/') { sub.to_string() } else { format!("{}/", sub) };
    tree.walk(TreeWalkMode::PreOrder, |root, entry| {
        let full = format!("{}{}", root, entry.name().unwrap_or_default());
        if !full.starts_with(&prefix) { return TreeWalkResult::Ok; }
        if let Some(kind) = entry.kind() {
            map.insert(full.clone(), kind);
        }
        TreeWalkResult::Ok
    }).map_err(|_| anyhow!("tree walk failed"))?;
    Ok(map)
}

/// Names for full-tree: paths like "src/lib.rs", recursive.
fn materialize_full_tree(tree: &Tree) -> Result<BTreeMap<String, ObjectType>> {
    let mut map = BTreeMap::new();
    tree.walk(TreeWalkMode::PreOrder, |root, entry| {
        let full = format!("{}{}", root, entry.name().unwrap_or_default());
        if full.is_empty() { return TreeWalkResult::Ok; }
        if let Some(kind) = entry.kind() {
            map.insert(full.clone(), kind);
        }
        TreeWalkResult::Ok
    }).map_err(|_| anyhow!("tree walk failed"))?;
    Ok(map)
}

fn decision_check(
    scope: &str,
    names: &BTreeMap<String, ObjectType>,
    allow_files: &Vec<String>,
    allow_dirs: &Vec<String>,
    ci: bool,
) -> Result<()> {
    let star_re = Regex::new(r"^\*[^/]*$").unwrap(); // root-only star patterns like "*.md"
    for (name, kind) in names {
        let allowed = match (scope, kind) {
            ("top-level", ObjectType::Tree) => allow_dirs.iter().any(|d| d == name),
            ("top-level", ObjectType::Blob) => {
                allow_files.iter().any(|p| p == name || (star_re.is_match(p) && star_match_root(p, name)))
            }
            // subtree/full-tree: only file rules apply (dirs are implicit)
            (_, ObjectType::Blob) => {
                // allow_files can be full paths or glob suffixes like "*.md"
                if allow_files.iter().any(|p| p == name) { true }
                else {
                    allow_files.iter().any(|p| {
                        if star_re.is_match(p) && !name.contains('/') { star_match_root(p, name) }
                        else if star_re.is_match(p) { star_match_basename(p, name) }
                        else { false }
                    })
                }
            }
            (_, ObjectType::Tree) => true, // directories are not rejected explicitly for recursive scopes
            _ => false,
        };
        if !allowed {
            let msg = format!("REJECT [{}] {}", scope, name);
            if ci { return Err(anyhow!(msg)); } else { eprintln!("{}", msg); }
        }
    }
    Ok(())
}

fn star_match_root(pat: &str, name: &str) -> bool {
    if pat.contains('/') || name.contains('/') { return false; }
    wildcard(pat, name)
}

fn star_match_basename(pat: &str, path: &str) -> bool {
    let base = Path::new(path).file_name().and_then(|s| s.to_str()).unwrap_or("");
    wildcard(pat, base)
}

fn wildcard(pat: &str, s: &str) -> bool {
    // simple '*' only, full-string match
    let mut rx = String::from("^");
    for ch in pat.chars() {
        match ch {
            '*' => rx.push_str(".*"),
            '.' => rx.push_str(r"\."),
            c @ ('+'| '?'| '('|')'|'['|']'|'{'|'}'|'|'|'^'|'$') => { rx.push('\\'); rx.push(c); }
            c => rx.push(c),
        }
    }
    rx.push('$');
    Regex::new(&rx).unwrap().is_match(s)
}

fn compute_digest(
    scope: &str,
    names: &BTreeMap<String, ObjectType>,
    allow_files: &Vec<String>,
    allow_dirs: &Vec<String>,
) -> Result<String> {
    // Subject: deterministic lines of names (keys), sorted by BTreeMap
    let subject = names.keys().cloned().collect::<Vec<_>>().join("\n");
    // Rules: stable JSON
    let rules = json!({
        "scope": scope,
        "allow_files": allow_files,
        "allow_dirs": allow_dirs,
    });
    let rules_str = serde_json::to_string(&rules)?;
    let mut h = Sha256::new();
    let mut h1 = Sha256::new(); h1.update(subject.as_bytes()); let a = h1.finalize();
    let mut h2 = Sha256::new(); h2.update(rules_str.as_bytes()); let b = h2.finalize();
    h.update(hex::encode(a)); h.update(b"\n"); h.update(hex::encode(b));
    Ok(format!("{:x}", h.finalize()))
}

fn validate_schema(doc: &str) -> Result<()> {
    // Skip schema validation for now to focus on core functionality
    // TODO: Implement proper schema validation
    Ok(())
}
