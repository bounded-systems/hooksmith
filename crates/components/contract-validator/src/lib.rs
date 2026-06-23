//! Contract-Validator WASM Component
//!
//! Pure validation logic — no git I/O, no git2. The host reads tree entries
//! from the git object database and passes them as plain data. This component
//! can therefore compile to `wasm32-wasip2` (or `wasm32-unknown-unknown`) and
//! run anywhere: CLI (wasmtime), Node.js (jco-transpiled), browser, MCP.
//!
//! Build:
//!   cargo build --target wasm32-wasip2 -p contract-validator --release
//!
//! Transpile to JS (requires @bytecodealliance/jco):
//!   jco transpile target/wasm32-wasip2/release/contract_validator.wasm \
//!       --wit crates/components/contract-validator/wit/ \
//!       -o js/dist/contract-validator/

#[cfg(feature = "host")]
wit_bindgen::generate!({
    path: "../wit/contract-validator.wit",
    world: "contract-validator-world",
});

// ── validate-names ────────────────────────────────────────────────────────────

/// Validate tree entry names against required/allowed/rejected rules.
///
/// Rules are applied in order:
///   1. Any name matching a `rejected` pattern → error
///   2. Any `required` name absent from the entry list → error
///   3. Any name that is not `required` and not `allowed` → warn
///
/// Patterns support a single trailing `*` wildcard (e.g. "docs/*").
pub fn validate_names_impl(entries: &[(String, String)], rules: &NamingRules) -> ValidationResult {
    let names: Vec<&str> = entries.iter().map(|(n, _)| n.as_str()).collect();
    let mut findings: Vec<Finding> = Vec::new();

    // 1. Rejected patterns
    for name in &names {
        for pat in &rules.rejected {
            if glob_match(pat, name) {
                findings.push(Finding {
                    level: "error".into(),
                    rule: "rejected-name".into(),
                    message: format!("\"{}\" matches rejected pattern \"{}\"", name, pat),
                    path: Some((*name).to_string()),
                });
            }
        }
    }

    // 2. Required names
    for pat in &rules.required {
        let present = names.iter().any(|n| glob_match(pat, n));
        if !present {
            findings.push(Finding {
                level: "error".into(),
                rule: "missing-required".into(),
                message: format!("required entry \"{}\" not found in tree", pat),
                path: None,
            });
        }
    }

    // 3. Names not covered by required or allowed
    if !rules.allowed.is_empty() {
        for name in &names {
            let covered = rules.required.iter().any(|p| glob_match(p, name))
                || rules.allowed.iter().any(|p| glob_match(p, name));
            if !covered {
                findings.push(Finding {
                    level: "warn".into(),
                    rule: "uncatalogued-name".into(),
                    message: format!(
                        "\"{}\" is not listed in the contract's allowed entries",
                        name
                    ),
                    path: Some((*name).to_string()),
                });
            }
        }
    }

    let errors = findings.iter().filter(|f| f.level == "error").count() as u8;
    let warns = findings.iter().filter(|f| f.level == "warn").count() as u8;
    let score = 10u8.saturating_sub(errors * 2).saturating_sub(warns);

    ValidationResult {
        passed: errors == 0,
        score,
        findings,
    }
}

/// Simple glob: only `*` at the end (e.g. "docs/*") or exact match.
fn glob_match(pattern: &str, name: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix("/*") {
        // "dir/*" → matches anything inside `dir/`
        name.starts_with(&format!("{}/", prefix))
    } else if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        name.starts_with(prefix)
    } else {
        pattern == name
    }
}

// ── check-copy-drift ─────────────────────────────────────────────────────────
//
// Mirrors string-audit's registryDrift (prose.mjs) but runs in WASM.
// Three classes of drift:
//   1. `--flag` references not in the vocab's flag list
//   2. `ENUM=value` references where `value` is not in the allowed set
//   3. `` `bin verb` `` backtick tokens where `bin`/`verb` are unknown

/// Check prose copy for registry drift.
pub fn check_copy_drift_impl(value: &str, copy_type: &str, vocab: &VocabSpec) -> Vec<DriftFinding> {
    // Only check copy types that are documentation surfaces.
    if !matches!(copy_type, "headline" | "body" | "subhead" | "title") {
        return vec![];
    }

    let mut out: Vec<DriftFinding> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    let flag_set: std::collections::HashSet<&str> =
        vocab.flags.iter().map(|s| s.as_str()).collect();
    let bin_set: std::collections::HashSet<&str> = vocab.bins.iter().map(|s| s.as_str()).collect();
    let verb_set: std::collections::HashSet<&str> =
        vocab.verb_ids.iter().map(|s| s.as_str()).collect();

    // 1. --flag references
    let mut i = 0;
    let bytes = value.as_bytes();
    while i + 1 < bytes.len() {
        if bytes[i] == b'-' && bytes[i + 1] == b'-' {
            // Make sure it's not mid-word (preceded by alphanumeric).
            let preceded_by_word =
                i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'-');
            if !preceded_by_word {
                // Collect the flag name.
                let start = i + 2;
                let end = bytes[start..]
                    .iter()
                    .position(|&b| !b.is_ascii_alphanumeric() && b != b'-')
                    .map(|p| start + p)
                    .unwrap_or(bytes.len());
                if end > start {
                    let flag = value[start..end].to_lowercase();
                    let key = format!("f:{}", flag);
                    if !flag_set.contains(flag.as_str()) && !seen.contains(&key) {
                        seen.insert(key);
                        out.push(DriftFinding {
                            level: "error".into(),
                            message: format!(
                                "registry-drift: --{} is not a flag of any verb (renamed/removed/typo?)",
                                flag
                            ),
                        });
                    }
                }
            }
            i += 2;
        } else {
            i += 1;
        }
    }

    // 2. ENUM=value references
    for entry in &vocab.enums {
        let flag_upper = entry.flag.to_uppercase();
        let prefix = format!("{}=", flag_upper);
        let mut search_start = 0;
        while let Some(pos) = value[search_start..].find(&prefix) {
            let abs = search_start + pos;
            let val_start = abs + prefix.len();
            let val_end = value[val_start..]
                .bytes()
                .position(|b| !b.is_ascii_alphanumeric() && b != b'-')
                .map(|p| val_start + p)
                .unwrap_or(value.len());
            let val = value[val_start..val_end].to_lowercase();
            if !val.is_empty() {
                let allowed: std::collections::HashSet<&str> =
                    entry.values.iter().map(|s| s.as_str()).collect();
                let key = format!("e:{}:{}", entry.flag, val);
                if !allowed.contains(val.as_str()) && !seen.contains(&key) {
                    seen.insert(key);
                    out.push(DriftFinding {
                        level: "error".into(),
                        message: format!(
                            "registry-drift: {}={} — not a valid value ({})",
                            entry.flag,
                            val,
                            entry.values.join("|")
                        ),
                    });
                }
            }
            search_start = val_start + 1;
            if search_start >= value.len() {
                break;
            }
        }
    }

    // 3. `bin verb` backtick tokens
    let mut search_start = 0;
    while let Some(open) = value[search_start..].find('`') {
        let abs_open = search_start + open + 1;
        if let Some(close_rel) = value[abs_open..].find('`') {
            let token = &value[abs_open..abs_open + close_rel];
            let parts: Vec<&str> = token.splitn(3, ' ').collect();
            if parts.len() == 2 {
                let bin = parts[0];
                let verb = parts[1];
                if !bin_set.contains(bin) {
                    out.push(DriftFinding {
                        level: "error".into(),
                        message: format!("registry-drift: `{}` — unknown bin", bin),
                    });
                } else if !verb_set.contains(verb) {
                    out.push(DriftFinding {
                        level: "error".into(),
                        message: format!("registry-drift: `{}` — verb not in registry", verb),
                    });
                }
            }
            search_start = abs_open + close_rel + 1;
        } else {
            break;
        }
    }

    out.truncate(5);
    out
}

// ── WIT glue (host feature only) ─────────────────────────────────────────────
//
// When compiled to WASM without the `host` feature, wit-bindgen is absent and
// we expose the functions via `#[no_mangle] extern "C"` (generated by the
// WIT component toolchain). With `host` enabled (for tests), we wire to the
// trait impl below.

#[cfg(feature = "host")]
struct ContractValidatorImpl;

#[cfg(feature = "host")]
impl exports::hooksmith::contract_validator::contract_validator::Guest for ContractValidatorImpl {
    fn validate_names(
        entries: Vec<exports::hooksmith::contract_validator::contract_validator::TreeEntry>,
        rules: exports::hooksmith::contract_validator::contract_validator::NamingRules,
    ) -> exports::hooksmith::contract_validator::contract_validator::ValidationResult {
        use exports::hooksmith::contract_validator::contract_validator as wit;
        let entries_plain: Vec<(String, String)> =
            entries.into_iter().map(|e| (e.name, e.kind)).collect();
        let rules_plain = NamingRules {
            required: rules.required,
            allowed: rules.allowed,
            rejected: rules.rejected,
        };
        let result = validate_names_impl(&entries_plain, &rules_plain);
        wit::ValidationResult {
            passed: result.passed,
            score: result.score,
            findings: result
                .findings
                .into_iter()
                .map(|f| wit::Finding {
                    level: f.level,
                    rule: f.rule,
                    message: f.message,
                    path: f.path,
                })
                .collect(),
        }
    }

    fn check_copy_drift(
        value: String,
        copy_type: String,
        vocab: exports::hooksmith::contract_validator::contract_validator::VocabSpec,
    ) -> Vec<exports::hooksmith::contract_validator::contract_validator::DriftFinding> {
        use exports::hooksmith::contract_validator::contract_validator as wit;
        let vocab_plain = VocabSpec {
            flags: vocab.flags,
            enums: vocab
                .enums
                .into_iter()
                .map(|e| EnumEntry {
                    flag: e.flag,
                    values: e.values,
                })
                .collect(),
            bins: vocab.bins,
            verb_ids: vocab.verb_ids,
        };
        check_copy_drift_impl(&value, &copy_type, &vocab_plain)
            .into_iter()
            .map(|f| wit::DriftFinding {
                level: f.level,
                message: f.message,
            })
            .collect()
    }
}

#[cfg(feature = "host")]
export!(ContractValidatorImpl);

// ── Plain Rust types (used by both the impl and the test helpers) ─────────────

pub struct NamingRules {
    pub required: Vec<String>,
    pub allowed: Vec<String>,
    pub rejected: Vec<String>,
}

pub struct Finding {
    pub level: String,
    pub rule: String,
    pub message: String,
    pub path: Option<String>,
}

pub struct ValidationResult {
    pub passed: bool,
    pub score: u8,
    pub findings: Vec<Finding>,
}

pub struct VocabSpec {
    pub flags: Vec<String>,
    pub enums: Vec<EnumEntry>,
    pub bins: Vec<String>,
    pub verb_ids: Vec<String>,
}

pub struct EnumEntry {
    pub flag: String,
    pub values: Vec<String>,
}

pub struct DriftFinding {
    pub level: String,
    pub message: String,
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(required: &[&str], allowed: &[&str], rejected: &[&str]) -> NamingRules {
        NamingRules {
            required: required.iter().map(|s| s.to_string()).collect(),
            allowed: allowed.iter().map(|s| s.to_string()).collect(),
            rejected: rejected.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn entries(names: &[(&str, &str)]) -> Vec<(String, String)> {
        names
            .iter()
            .map(|(n, k)| (n.to_string(), k.to_string()))
            .collect()
    }

    fn vocab(
        flags: &[&str],
        enums: &[(&str, &[&str])],
        bins: &[&str],
        verbs: &[&str],
    ) -> VocabSpec {
        VocabSpec {
            flags: flags.iter().map(|s| s.to_string()).collect(),
            enums: enums
                .iter()
                .map(|(f, vals)| EnumEntry {
                    flag: f.to_string(),
                    values: vals.iter().map(|s| s.to_string()).collect(),
                })
                .collect(),
            bins: bins.iter().map(|s| s.to_string()).collect(),
            verb_ids: verbs.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn required_present_passes() {
        let r = validate_names_impl(
            &entries(&[("Cargo.toml", "blob"), ("src", "tree")]),
            &rules(&["Cargo.toml", "src"], &[], &[]),
        );
        assert!(r.passed);
        assert_eq!(r.score, 10);
        assert!(r.findings.is_empty());
    }

    #[test]
    fn missing_required_is_error() {
        let r = validate_names_impl(
            &entries(&[("src", "tree")]),
            &rules(&["Cargo.toml", "src"], &[], &[]),
        );
        assert!(!r.passed);
        assert!(r.findings.iter().any(|f| f.rule == "missing-required"));
    }

    #[test]
    fn rejected_name_is_error() {
        let r = validate_names_impl(
            &entries(&[("target", "tree"), ("src", "tree")]),
            &rules(&["src"], &[], &["target"]),
        );
        assert!(!r.passed);
        assert!(r.findings.iter().any(|f| f.rule == "rejected-name"));
    }

    #[test]
    fn uncatalogued_name_is_warn() {
        let r = validate_names_impl(
            &entries(&[("src", "tree"), ("mystery", "blob")]),
            &rules(&["src"], &["src"], &[]),
        );
        assert!(r.passed); // warns don't block
        assert!(r
            .findings
            .iter()
            .any(|f| f.rule == "uncatalogued-name" && f.level == "warn"));
    }

    #[test]
    fn glob_wildcard_matches_prefix() {
        assert!(glob_match("docs/*", "docs/api"));
        assert!(!glob_match("docs/*", "notdocs/api"));
        assert!(glob_match("*.md", "README.md"));
        assert!(!glob_match("*.md", "README.toml"));
    }

    #[test]
    fn drift_clean_copy_is_empty() {
        let v = vocab(
            &["catalog", "store"],
            &[("store", &["fs", "cas"])],
            &["hooksmith"],
            &["validate"],
        );
        let r = check_copy_drift_impl("Run the validator.", "body", &v);
        assert!(r.is_empty());
    }

    #[test]
    fn drift_unknown_flag_is_error() {
        let v = vocab(&["catalog"], &[], &[], &[]);
        let r = check_copy_drift_impl("Pass --backend to configure.", "body", &v);
        assert!(r.iter().any(|f| f.message.contains("--backend")));
    }

    #[test]
    fn drift_invalid_enum_value_is_error() {
        let v = vocab(&["store"], &[("store", &["fs", "cas"])], &[], &[]);
        let r = check_copy_drift_impl("Use STORE=redis for caching.", "body", &v);
        assert!(r.iter().any(|f| f.message.contains("redis")));
    }

    #[test]
    fn drift_valid_enum_value_is_clean() {
        let v = vocab(&["store"], &[("store", &["fs", "cas"])], &[], &[]);
        let r = check_copy_drift_impl("Use STORE=cas for caching.", "body", &v);
        assert!(r.is_empty());
    }

    #[test]
    fn drift_unknown_bin_in_backtick_is_error() {
        let v = vocab(&[], &[], &["hooksmith"], &["validate"]);
        let r = check_copy_drift_impl("Run `unknown-bin validate` to check.", "body", &v);
        assert!(r.iter().any(|f| f.message.contains("unknown-bin")));
    }

    #[test]
    fn drift_unknown_verb_in_backtick_is_error() {
        let v = vocab(&[], &[], &["hooksmith"], &["validate"]);
        let r = check_copy_drift_impl("Run `hooksmith frobnicate` to check.", "body", &v);
        assert!(r.iter().any(|f| f.message.contains("frobnicate")));
    }

    #[test]
    fn drift_skips_non_doc_copy_types() {
        let v = vocab(&["store"], &[], &[], &[]);
        let r = check_copy_drift_impl("Pass --unknown flag.", "cta", &v);
        assert!(r.is_empty(), "cta type should be skipped");
    }
}
