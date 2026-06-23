use anyhow::{anyhow, Context, Result};
use clap::Parser;
use git2::{ObjectType, Repository, Tree, TreeWalkMode, TreeWalkResult};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::PathBuf};

#[derive(Parser, Debug)]
struct Args {
    /// Path to the agreement JSON (tree-only policy)
    #[arg(long, default_value = "agreement.json")]
    agreement: PathBuf,

    /// Git ref to validate (default: HEAD)
    #[arg(long, default_value = "HEAD")]
    r#ref: String,

    /// Fail fast on first mismatch
    #[arg(long, default_value_t = true)]
    ci: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Agreement {
    version: String,
    mode: String,             // "tree"
    precedence: String,       // "allow-overrides-reject" (required behavior)
    default_action: String,   // "reject"
    allow_dirs: Vec<String>,  // literal names only
    allow_files: Vec<String>, // literal or root-glob "*.ext"
    subject: Subject,
    digest: DigestField,
    #[serde(default)]
    signature: Option<Signature>, // optional
}

#[derive(Deserialize)]
struct Subject {
    r#ref: String,
    scope: String, // "top-level"
}

#[derive(Deserialize)]
struct DigestField {
    algo: String, // "sha256"
    value: String,
}

#[derive(Deserialize)]
struct Signature {
    r#type: String, // "gpg" | "minisign" (stub)
    value: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 1) Load agreement
    let text = fs::read_to_string(&args.agreement)
        .with_context(|| format!("reading {:?}", args.agreement))?;
    let agreement: Agreement = serde_json::from_str(&text).context("parsing agreement.json")?;

    // 2) Schema validation (skipped for now - focus on core functionality)
    // validate_schema(&text).context("agreement schema invalid")?;

    // 3) Sanity on required fields
    if agreement.mode != "tree" || agreement.subject.scope != "top-level" {
        return Err(anyhow!(
            "agreement must be tree-only (mode=tree, scope=top-level)"
        ));
    }
    if agreement.precedence != "allow-overrides-reject" || agreement.default_action != "reject" {
        return Err(anyhow!(
            "agreement must specify precedence=allow-overrides-reject and defaultAction=reject"
        ));
    }
    if agreement.digest.algo != "sha256" {
        return Err(anyhow!(
            "unsupported digest algo: {}",
            agreement.digest.algo
        ));
    }

    // 4) Materialize subject (top-level names via libgit2, no recursion)
    let repo = Repository::discover(".")?;
    let obj = repo.revparse_single(&args.r#ref)?;
    let commit = obj.peel_to_commit().context("ref is not a commit")?;
    let tree = commit.tree()?;

    let (names, files, dirs) = top_level_entries(&tree)?;
    // No slashes in names by construction; assert for safety.
    if names.iter().any(|n| n.contains('/')) {
        return Err(anyhow!(
            "top-level names contained a slash; tree-only invariant broken"
        ));
    }

    // 5) Decision check (allow-overrides-reject, no recursion)
    validate_decisions(&agreement, &files, &dirs, args.ci)?;

    // 6) Digest check (subject + rules canonicalization)
    let computed = compute_digest(&agreement, &names)?;
    if computed != agreement.digest.value {
        return Err(anyhow!(
            "digest mismatch: computed={}, recorded={}",
            computed,
            agreement.digest.value
        ));
    }

    // 7) Optional signature (stub hook)
    if let Some(sig) = &agreement.signature {
        eprintln!(
            "(info) signature present type={} – verification not implemented in this binary",
            sig.r#type
        );
        // Implement GPG/Minisign verification here if you want hard guarantees in CI.
    }

    println!("OK: tree-only agreement validated for ref {}", args.r#ref);
    Ok(())
}

fn top_level_entries(
    tree: &Tree,
) -> Result<(BTreeSet<String>, BTreeSet<String>, BTreeSet<String>)> {
    let mut names = BTreeSet::new();
    let mut files = BTreeSet::new();
    let mut dirs = BTreeSet::new();
    tree.walk(TreeWalkMode::PreOrder, |root, entry| {
        // Only accept the first frame (root == "")
        if !root.is_empty() {
            return TreeWalkResult::Skip;
        }
        let name = entry
            .name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "<invalid>".into());
        names.insert(name.clone());
        match entry.kind() {
            Some(ObjectType::Blob) => {
                files.insert(name);
            }
            Some(ObjectType::Tree) => {
                dirs.insert(name);
            }
            _ => {}
        }
        TreeWalkResult::Ok
    })
    .map_err(|_| anyhow!("tree walk failed"))?;
    Ok((names, files, dirs))
}

fn validate_decisions(
    ag: &Agreement,
    files: &BTreeSet<String>,
    dirs: &BTreeSet<String>,
    ci: bool,
) -> Result<()> {
    // Build allow sets
    let allow_dirs: BTreeSet<&str> = ag.allow_dirs.iter().map(|s| s.as_str()).collect();
    let allow_file_pats: Vec<&str> = ag.allow_files.iter().map(|s| s.as_str()).collect();

    // Dirs: exact names only
    for d in dirs {
        if !allow_dirs.contains(d.as_str()) {
            let msg = format!("REJECT dir: {}", d);
            if ci {
                return Err(anyhow!(msg));
            } else {
                eprintln!("{}", msg);
            }
        }
    }

    // Files: literal or simple root-glob ("*" matches any non-slash)
    'outer: for f in files {
        // exact literal
        if ag.allow_files.iter().any(|p| p == f) {
            continue 'outer;
        }
        // simple star patterns
        for pat in &allow_file_pats {
            if pat.contains('*') && star_match_root(pat, f) {
                continue 'outer;
            }
        }
        let msg = format!("REJECT file: {}", f);
        if ci {
            return Err(anyhow!(msg));
        } else {
            eprintln!("{}", msg);
        }
    }

    Ok(())
}

/// Root-glob matcher: supports a single or multiple '*' wildcards; no '/' allowed in either side.
/// Translates '*' -> ".*" for a full-string match.
fn star_match_root(pat: &str, name: &str) -> bool {
    if pat.contains('/') || name.contains('/') {
        return false;
    }
    // Escape regex meta except '*'
    let mut re = String::from("^");
    for ch in pat.chars() {
        match ch {
            '*' => re.push_str(".*"),
            '.' => re.push_str(r"\."),
            '?' => re.push_str(r"\?"),
            '+' => re.push_str(r"\+"),
            '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' => {
                re.push('\\');
                re.push(ch);
            }
            other => re.push(other),
        }
    }
    re.push('$');
    regex::Regex::new(&re)
        .map(|r| r.is_match(name))
        .unwrap_or(false)
}

fn compute_digest(ag: &Agreement, names: &BTreeSet<String>) -> Result<String> {
    // Subject canonicalization: sorted newline-joined names
    let subject = names.iter().cloned().collect::<Vec<_>>().join("\n");

    // Rules canonicalization: stable JSON using BTreeSet ordering to remove non-determinism
    let rules = json!({
        "mode": ag.mode,
        "precedence": ag.precedence,
        "defaultAction": ag.default_action,
        "allow_dirs": ag.allow_dirs,
        "allow_files": ag.allow_files
    });
    let rules_str = serde_json::to_string(&rules)?; // keys are in insertion order; we inserted deterministically

    let mut outer = Sha256::new();
    let mut inner = Sha256::new();
    inner.update(subject.as_bytes());
    let sha_subject = inner.finalize();

    let mut inner2 = Sha256::new();
    inner2.update(rules_str.as_bytes());
    let sha_rules = inner2.finalize();

    outer.update(hex::encode(sha_subject));
    outer.update(b"\n");
    outer.update(hex::encode(sha_rules));
    Ok(format!("{:x}", outer.finalize()))
}
