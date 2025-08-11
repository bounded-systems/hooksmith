use anyhow::{Result, anyhow};
use git2::{Repository, ObjectType, TreeWalkMode, TreeWalkResult};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::cell::Cell;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Agreement {
    pub version: String,
    pub mode: String,               // "tree"
    pub policy: String,             // "allow-only"
    pub allow_dirs: Vec<String>,
    pub allow_files: Vec<String>,
    pub subject: Subject,
    pub digest: DigestField,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Subject { 
    pub scope: String 
} // "top-level" (for now)

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DigestField { 
    pub algo: String, 
    pub value: String 
}



// ---------- Dispatcher (intake)
pub struct Dispatcher;

impl Dispatcher {
    pub fn discover(root: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
        let base = root.join(".hooksmith/agreements");
        if !base.exists() { 
            return Ok(vec![]); 
        }
        let mut out = vec![];
        for e in walkdir::WalkDir::new(base) {
            let e = e?;
            if e.file_type().is_file() && e.path().extension().map(|x| x=="json").unwrap_or(false) {
                out.push(e.into_path());
            }
        }
        out.sort();
        Ok(out)
    }

    pub fn load(path: &std::path::Path) -> Result<Agreement> {
        let txt = std::fs::read_to_string(path)?;
        let ag: Agreement = serde_json::from_str(&txt)?;
        
        // Enforce invariants
        if ag.mode != "tree" || ag.subject.scope != "top-level" {
            return Err(anyhow!("agreement not tree/top-level: {}", path.display()));
        }
        if ag.policy != "allow-only" {
            return Err(anyhow!("policy must be allow-only"));
        }
        if ag.digest.algo != "sha256" { 
            return Err(anyhow!("unsupported digest algo")); 
        }
        Ok(ag)
    }
}

// ---------- Researcher (single pass, streaming)
pub struct Researcher { 
    repo: Repository 
}

impl Researcher {
    pub fn new(repo_root: &std::path::Path) -> Result<Self> {
        Ok(Self { repo: Repository::discover(repo_root)? })
    }

    pub fn validate_agreement(&self, ag: &Agreement, r#ref: &str) -> Result<ValidationResult> {
        let commit = self.repo.revparse_single(r#ref)?.peel_to_commit()?;
        let tree = commit.tree()?;
        
        let mut auditor = Auditor::new();
        let mut files = 0usize;
        let mut dirs = 0usize;
        let fail_reason = Cell::new(None::<String>);
        
        let result = tree.walk(TreeWalkMode::PreOrder, |root, e| {
            if !root.is_empty() { 
                return TreeWalkResult::Skip; 
            }
            
            let (Some(name), Some(kind)) = (e.name(), e.kind()) else { 
                return TreeWalkResult::Ok; 
            };

            // Mandator: fail fast
            if !Mandator::check_entry(&ag.allow_dirs, &ag.allow_files, name, kind) {
                fail_reason.set(Some(format!("REJECT {}", name)));
                return TreeWalkResult::Abort;
            }

            // Reporter (counters only)
            match kind { 
                ObjectType::Tree => dirs += 1, 
                ObjectType::Blob => files += 1, 
                _ => {} 
            }

            // Auditor (streaming)
            auditor.ingest_name(name);

            TreeWalkResult::Ok
        });
        
        if let Err(_) = result {
            return Err(anyhow!("tree walk failed"));
        }
        
        // Check for failure reason
        if let Some(reason) = fail_reason.take() {
            return Err(anyhow!(reason));
        }
        
        // Finalize digest
        let computed = auditor.finalize(&ag.allow_dirs, &ag.allow_files);
        let digest_ok = computed == ag.digest.value;
        
        Ok(ValidationResult {
            files,
            dirs,
            digest_ok,
            computed_digest: computed,
        })
    }
}

#[derive(Debug)]
pub struct ValidationResult {
    pub files: usize,
    pub dirs: usize,
    pub digest_ok: bool,
    pub computed_digest: String,
}



// ---------- Mandator (streaming, no overrides, no recursion)
pub struct Mandator;

impl Mandator {
    pub fn check_entry(allow_dirs: &[String], allow_files: &[String], name: &str, kind: ObjectType) -> bool {
        match kind {
            ObjectType::Tree => allow_dirs.iter().any(|d| d == name),
            ObjectType::Blob => {
                if allow_files.iter().any(|p| p == name) { 
                    return true; 
                }
                // root '*.ext' only
                allow_files.iter().any(|p| {
                    p.as_bytes().first() == Some(&b'*') && 
                    !p.contains('/') && 
                    star_match_root(p, name)
                })
            }
            _ => false
        }
    }
}

fn star_match_root(pat: &str, name: &str) -> bool {
    if pat.contains('/') || name.contains('/') { 
        return false; 
    }
    // simple '*' matcher (no regex crate required in prod if you prefer)
    let mut pi = 0usize; 
    let p = pat.as_bytes(); 
    let s = name.as_bytes();
    let (mut si, mut star_idx, mut match_idx) = (0usize, None, 0usize);
    
    while si < s.len() {
        if pi < p.len() && (p[pi] == b'?' || p[pi] == s[si]) { 
            pi+=1; 
            si+=1; 
        }
        else if pi < p.len() && p[pi] == b'*' { 
            star_idx = Some(pi); 
            match_idx = si; 
            pi+=1; 
        }
        else if let Some(st) = star_idx { 
            pi = st+1; 
            match_idx += 1; 
            si = match_idx; 
        }
        else { 
            return false; 
        }
    }
    while pi < p.len() && p[pi] == b'*' { 
        pi+=1; 
    }
    pi == p.len()
}

// ---------- Auditor (order-independent digest; streaming)
fn add256(acc: &mut [u8;32], h: [u8;32]) {
    let mut carry = 0u16;
    for i in (0..32).rev() {
        let sum = acc[i] as u16 + h[i] as u16 + carry;
        acc[i] = (sum & 0xff) as u8;
        carry = sum >> 8;
    }
}

pub struct Auditor { 
    acc: [u8;32] 
}

impl Auditor {
    pub fn new() -> Self { 
        Self { acc: [0u8;32] } 
    }
    
    pub fn ingest_name(&mut self, name: &str) {
        let h = Sha256::digest(name.as_bytes());
        add256(&mut self.acc, h.into());
    }
    
    pub fn finalize(self, allow_dirs: &[String], allow_files: &[String]) -> String {
        let rules = serde_json::json!({
            "mode":"tree",
            "policy":"allow-only",
            "allow_dirs": allow_dirs,
            "allow_files": allow_files
        }).to_string();
        let h_rules = Sha256::digest(rules.as_bytes());
        let mut outer = Sha256::new();
        outer.update(hex::encode(self.acc));
        outer.update(b"\n");
        outer.update(format!("{:x}", h_rules));
        format!("{:x}", outer.finalize())
    }
}

// ---------- Triage Officer (orchestration + SARIF)
#[derive(Serialize)]
struct Sarif {
    version: String,
    #[serde(rename="runs")] 
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
}

#[derive(Serialize)]
struct SarifTool { 
    driver: SarifDriver 
}

#[derive(Serialize)]
struct SarifDriver { 
    name: String, 
    information_uri: String 
}

#[derive(Serialize)]
struct SarifResult { 
    level: String, 
    message: SarifMessage, 
    locations: Vec<SarifLocation> 
}

#[derive(Serialize)]
struct SarifMessage { 
    text: String 
}

#[derive(Serialize)]
struct SarifLocation { 
    physical_location: SarifPhysical 
}

#[derive(Serialize)]
struct SarifPhysical { 
    artifact_location: SarifArtifact 
}

#[derive(Serialize)]
struct SarifArtifact { 
    uri: String 
}

pub struct TriageOfficer<'a> { 
    researcher: Researcher, 
    root: &'a std::path::Path 
}

impl<'a> TriageOfficer<'a> {
    pub fn new(root: &'a std::path::Path) -> Result<Self> {
        Ok(Self { 
            researcher: Researcher::new(root)?, 
            root 
        })
    }

    pub fn run(&self, refspec: &str) -> Result<i32> {
        let paths = Dispatcher::discover(self.root)?;
        let mut results = vec![];
        
        for p in paths {
            let ag = match Dispatcher::load(&p) { 
                Ok(a) => a, 
                Err(e) => {
                    results.push(sarif_err(format!("load: {}", e), &p)); 
                    continue; 
                }
            };
            
            let validation = match self.researcher.validate_agreement(&ag, refspec) { 
                Ok(v) => v, 
                Err(e) => {
                    results.push(sarif_err(format!("validation: {}", e), &p)); 
                    continue; 
                }
            };
            
            if !validation.digest_ok {
                results.push(sarif_err(
                    format!("digest mismatch: computed={}, recorded={}", 
                           validation.computed_digest, ag.digest.value), 
                    &p
                )); 
                continue;
            }
            
            results.push(SarifResult {
                level: "note".into(),
                message: SarifMessage { 
                    text: format!("agreement ok ({} files, {} dirs)", validation.files, validation.dirs).into() 
                },
                locations: vec![loc(&p)],
            });
        }
        
        let sarif = Sarif {
            version: "2.1.0".into(),
            runs: vec![SarifRun {
                tool: SarifTool { 
                    driver: SarifDriver {
                        name: "hooksmith-agreements".into(), 
                        information_uri: "https://internal/hooksmith".into()
                    }
                },
                results
            }]
        };
        
        println!("{}", serde_json::to_string_pretty(&sarif)?);
        
        // nonzero exit on any error-level results
        let fail = sarif.runs[0].results.iter().any(|r| r.level != "note");
        Ok(if fail { 1 } else { 0 })
    }
}

fn loc(p: &std::path::Path) -> SarifLocation {
    SarifLocation { 
        physical_location: SarifPhysical { 
            artifact_location: SarifArtifact {
                uri: p.display().to_string()
            } 
        } 
    }
}

fn sarif_err(msg: String, p: &std::path::Path) -> SarifResult {
    SarifResult { 
        level: "error".into(), 
        message: SarifMessage { text: msg }, 
        locations: vec![loc(p)] 
    }
}
