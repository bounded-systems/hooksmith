use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::io::{self, Write};
use std::process::Command;

/// Actor 1: Researcher - analyzes tree objects
struct Researcher {
    tool_fingerprint: String,
}

impl Researcher {
    fn new() -> Self {
        Self {
            tool_fingerprint: "researcher-v1.0.0".to_string(),
        }
    }

    fn analyze_tree(&self, tree_sha: &str) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(["ls-tree", "-z", "--name-only", tree_sha])
            .output()
            .context("Failed to get root tree entries")?;

        if !output.status.success() {
            anyhow::bail!(
                "git ls-tree failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let entries: Vec<String> = String::from_utf8(output.stdout)?
            .split('\0')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        Ok(entries)
    }
}

/// Actor 2: Reporter - normalizes analysis to NDJSON
struct Reporter {
    domain: String,
    version: String,
}

impl Reporter {
    fn new() -> Self {
        Self {
            domain: "hooksmith.dev".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    fn create_report(&self, entries: &[String]) -> Result<Vec<Value>> {
        let mut report = Vec::new();

        // Sort entries for stable output
        let mut sorted_entries = entries.to_vec();
        sorted_entries.sort();

        for entry in sorted_entries {
            report.push(json!({
                "type": "entry",
                "name": entry
            }));
        }

        Ok(report)
    }
}

/// Actor 3: Mandator - creates expectations from contract
struct Mandator {
    contract: Value,
}

impl Mandator {
    fn new(contract_path: &str) -> Result<Self> {
        let contract_content =
            std::fs::read_to_string(contract_path).context("Failed to read contract file")?;

        let contract: Value =
            serde_json::from_str(&contract_content).context("Failed to parse contract JSON")?;

        Ok(Self { contract })
    }

    fn create_mandate(&self) -> Result<Vec<Value>> {
        let spec = &self.contract["spec"]["git"]["tree"]["objects"]["names"];
        let mut mandate = Vec::new();

        // Required entries
        if let Some(required) = spec["required"].as_array() {
            for req in required {
                if let Some(name) = req.as_str() {
                    mandate.push(json!({
                        "type": "required",
                        "name": name
                    }));
                }
            }
        }

        // Allowed entries
        if let Some(allowed) = spec["allowed"].as_array() {
            for allow in allowed {
                if let Some(name) = allow.as_str() {
                    mandate.push(json!({
                        "type": "allowed",
                        "name": name
                    }));
                }
            }
        }

        // Rejected entries
        if let Some(rejected) = spec["rejected"].as_array() {
            for reject in rejected {
                if let Some(name) = reject.as_str() {
                    mandate.push(json!({
                        "type": "rejected",
                        "name": name
                    }));
                }
            }
        }

        // Ignored entries
        if let Some(ignored) = spec["ignored"].as_array() {
            for ignore in ignored {
                if let Some(name) = ignore.as_str() {
                    mandate.push(json!({
                        "type": "ignored",
                        "name": name
                    }));
                }
            }
        }

        Ok(mandate)
    }
}

/// Actor 4: Auditor - compares report against mandate
struct Auditor {
    contract_name: String,
    version: String,
}

impl Auditor {
    fn new(contract_name: String, version: String) -> Self {
        Self {
            contract_name,
            version,
        }
    }

    fn audit(&self, report: &[Value], mandate: &[Value]) -> Result<Vec<Value>> {
        let mut diff = Vec::new();

        // Extract sets from mandate
        let mut required = HashSet::new();
        let mut allowed = HashSet::new();
        let mut rejected = HashSet::new();
        let mut ignored = HashSet::new();

        for item in mandate {
            if let (Some(item_type), Some(name)) = (item["type"].as_str(), item["name"].as_str()) {
                match item_type {
                    "required" => {
                        required.insert(name.to_string());
                    }
                    "allowed" => {
                        allowed.insert(name.to_string());
                    }
                    "rejected" => {
                        rejected.insert(name.to_string());
                    }
                    "ignored" => {
                        ignored.insert(name.to_string());
                    }
                    _ => {}
                }
            }
        }

        // Extract actual entries from report
        let mut actual = HashSet::new();
        for item in report {
            if let Some(name) = item["name"].as_str() {
                actual.insert(name.to_string());
            }
        }

        // Check missing required
        for req in &required {
            if !actual.contains(req) {
                diff.push(json!({
                    "kind": "missing_required",
                    "name": req
                }));
            }
        }

        // Check unexpected entries (not ignored, not in allowed)
        for entry in &actual {
            if !ignored.contains(entry) && !allowed.contains(entry) {
                diff.push(json!({
                    "kind": "unexpected_entry",
                    "name": entry
                }));
            }
        }

        // Check rejected entries
        for entry in &actual {
            if !ignored.contains(entry) && rejected.contains(entry) {
                diff.push(json!({
                    "kind": "rejected_entry",
                    "name": entry
                }));
            }
        }

        Ok(diff)
    }
}

/// Triage Officer - converts diff to SARIF
struct TriageOfficer {
    contract_name: String,
    version: String,
}

impl TriageOfficer {
    fn new(contract_name: String, version: String) -> Self {
        Self {
            contract_name,
            version,
        }
    }

    fn create_sarif(&self, diff: &[Value], tree_sha: &str) -> Result<Value> {
        let mut results = Vec::new();

        for violation in diff {
            if let (Some(kind), Some(name)) =
                (violation["kind"].as_str(), violation["name"].as_str())
            {
                let message = match kind {
                    "missing_required" => format!("Missing required entry: {}", name),
                    "unexpected_entry" => format!("Unexpected entry at root: {}", name),
                    "rejected_entry" => format!("Rejected entry at root: {}", name),
                    _ => format!("Unknown violation: {}", name),
                };

                results.push(json!({
                    "ruleId": "root-policy-violation",
                    "level": "error",
                    "message": {
                        "text": message
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": name
                            }
                        }
                    }]
                }));
            }
        }

        let sarif = json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "hooksmith-root-contract",
                        "version": self.version,
                        "informationUri": "https://hooksmith.dev"
                    }
                },
                "invocations": [{
                    "executionSuccessful": results.is_empty(),
                    "commandLine": format!("git ls-tree -z --name-only {}", tree_sha)
                }],
                "results": results
            }]
        });

        Ok(sarif)
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <contract-path> [tree-sha]", args[0]);
        std::process::exit(1);
    }

    let contract_path = &args[1];
    let default_tree = "HEAD^{tree}".to_string();
    let tree_sha = args.get(2).unwrap_or(&default_tree);

    // Step 1: Scope resolver - resolve to root tree
    let resolved_tree = if tree_sha == "HEAD^{tree}" {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD^{tree}"])
            .output()
            .context("Failed to resolve tree SHA")?;

        if !output.status.success() {
            anyhow::bail!(
                "git rev-parse failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        String::from_utf8(output.stdout)?.trim().to_string()
    } else {
        tree_sha.clone()
    };

    // Step 2: Enumerator - list names at root only
    let researcher = Researcher::new();
    let entries = researcher.analyze_tree(&resolved_tree)?;

    // Step 3: Researcher → Reporter - normalize to NDJSON
    let reporter = Reporter::new();
    let report = reporter.create_report(&entries)?;

    // Step 4: Mandator - contract → expectation
    let mandator = Mandator::new(contract_path)?;
    let mandate = mandator.create_mandate()?;

    // Step 5: Auditor - compare and emit diff
    let auditor = Auditor::new("object-names".to_string(), "1.0.0".to_string());
    let diff = auditor.audit(&report, &mandate)?;

    // Step 6: Triage Officer - convert to SARIF
    let triage = TriageOfficer::new("object-names".to_string(), "1.0.0".to_string());
    let sarif = triage.create_sarif(&diff, &resolved_tree)?;

    // Output results
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    // Output report.ndjson
    for item in &report {
        writeln!(handle, "{}", serde_json::to_string(item)?)?;
    }

    // Output mandate.ndjson
    for item in &mandate {
        writeln!(handle, "{}", serde_json::to_string(item)?)?;
    }

    // Output diff.ndjson
    for item in &diff {
        writeln!(handle, "{}", serde_json::to_string(item)?)?;
    }

    // Output SARIF to stderr for CI
    eprintln!("SARIF Output:");
    eprintln!("{}", serde_json::to_string_pretty(&sarif)?);

    // Exit with error if there are violations
    if !diff.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}
