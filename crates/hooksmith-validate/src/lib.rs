use anyhow::{Result, anyhow};
use git2::{Repository, ObjectType, Tree, TreeWalkMode, TreeWalkResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use sha2::{Sha256, Digest};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Agreement {
    pub version: String,
    pub mode: String,               // "tree"
    pub precedence: String,         // "allow-overrides-reject"
    pub default_action: String,     // "reject"
    pub allow_dirs: Option<Vec<String>>,
    pub allow_files: Option<Vec<String>>,
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

#[derive(Clone, Debug)]
pub struct SubjectData { 
    pub names: BTreeMap<String, ObjectType> 
} // top-level only

// ---------- Dispatcher (intake)
pub struct Dispatcher;

impl Dispatcher {
    pub fn discover(root: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
        let base = root.join(".hooksmith/agreements");
        eprintln!("Looking for agreements in: {}", base.display());
        if !base.exists() { 
            eprintln!("Directory does not exist");
            return Ok(vec![]); 
        }
        let mut out = vec![];
        for e in walkdir::WalkDir::new(base) {
            let e = e?;
            eprintln!("Found: {} (is_file: {}, ext: {:?})", 
                     e.path().display(), 
                     e.file_type().is_file(),
                     e.path().extension());
            if e.file_type().is_file() && e.path().extension().map(|x| x=="json").unwrap_or(false) {
                out.push(e.into_path());
            }
        }
        out.sort();
        eprintln!("Discovered {} agreement files", out.len());
        Ok(out)
    }

    pub fn load(path: &std::path::Path) -> Result<Agreement> {
        let txt = std::fs::read_to_string(path)?;
        let ag: Agreement = serde_json::from_str(&txt)?;
        
        // Enforce invariants
        if ag.mode != "tree" || ag.subject.scope != "top-level" {
            return Err(anyhow!("agreement not tree/top-level: {}", path.display()));
        }
        if ag.precedence != "allow-overrides-reject" || ag.default_action != "reject" {
            return Err(anyhow!("policy must be allow-overrides-reject + reject default"));
        }
        if ag.digest.algo != "sha256" { 
            return Err(anyhow!("unsupported digest algo")); 
        }
        Ok(ag)
    }
}

// ---------- Researcher
pub struct Researcher { 
    repo: Repository 
}

impl Researcher {
    pub fn new(repo_root: &std::path::Path) -> Result<Self> {
        Ok(Self { repo: Repository::discover(repo_root)? })
    }

    pub fn materialize_top_level(&self, r#ref: &str) -> Result<SubjectData> {
        let commit = self.repo.revparse_single(r#ref)?.peel_to_commit()?;
        let tree = commit.tree()?;
        Ok(SubjectData { names: top_level(&tree)? })
    }
}

fn top_level(tree: &Tree) -> Result<BTreeMap<String, ObjectType>> {
    let mut m = BTreeMap::new();
    tree.walk(TreeWalkMode::PreOrder, |root, e| {
        if !root.is_empty() { 
            return TreeWalkResult::Skip; 
        }
        if let (Some(n), Some(k)) = (e.name(), e.kind()) { 
            m.insert(n.to_string(), k); 
        }
        TreeWalkResult::Ok
    }).map_err(|_| anyhow!("tree walk failed"))?;
    Ok(m)
}

// ---------- Reporter (canonical subject summary for logs + digest)
#[derive(Clone, Debug, Serialize)]
pub struct SubjectReport {
    pub total: usize,
    pub files: usize,
    pub dirs: usize,
    pub names: Vec<String>, // sorted
}

pub struct Reporter;

impl Reporter {
    pub fn summarize(sd: &SubjectData) -> SubjectReport {
        let mut names: Vec<_> = sd.names.keys().cloned().collect();
        names.sort();
        let files = sd.names.values().filter(|k| **k == ObjectType::Blob).count();
        let dirs  = sd.names.values().filter(|k| **k == ObjectType::Tree).count();
        SubjectReport { total: sd.names.len(), files, dirs, names }
    }
}

// ---------- Mandator (contract decision)
pub struct Mandator;

impl Mandator {
    pub fn enforce(ag: &Agreement, sd: &SubjectData) -> Result<()> {
        use regex::Regex;
        let star = Regex::new(r"^\*[^/]*$")?;
        
        let empty_dirs: Vec<String> = vec![];
        let empty_files: Vec<String> = vec![];
        let allow_dirs = ag.allow_dirs.as_ref().unwrap_or(&empty_dirs);
        let allow_files = ag.allow_files.as_ref().unwrap_or(&empty_files);
        
        for (name, kind) in &sd.names {
            let allowed = match kind {
                ObjectType::Tree => allow_dirs.iter().any(|d| d == name),
                ObjectType::Blob => {
                    allow_files.iter().any(|p| p == name)
                    || allow_files.iter().any(|p| star.is_match(p) && star_match_root(p, name))
                }
                _ => false
            };
            if !allowed { 
                return Err(anyhow!("REJECT {}", name)); 
            }
        }
        Ok(())
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

// ---------- Auditor (digest over subject+rules)
pub struct Auditor;

impl Auditor {
    pub fn digest(ag: &Agreement, sr: &SubjectReport) -> String {
        let subject = sr.names.join("\n");
        let rules   = serde_json::json!({
            "mode":"tree",
            "precedence":"allow-overrides-reject",
            "defaultAction":"reject",
            "allow_dirs": ag.allow_dirs.as_ref().unwrap_or(&vec![]),
            "allow_files": ag.allow_files.as_ref().unwrap_or(&vec![])
        }).to_string();
        
        let a = Sha256::digest(subject.as_bytes());
        let b = Sha256::digest(rules.as_bytes());
        let mut h = Sha256::new();
        h.update(format!("{:x}\n{:x}", a, b));
        format!("{:x}", h.finalize())
    }

    pub fn verify(ag: &Agreement, sr: &SubjectReport) -> Result<()> {
        let c = Self::digest(ag, sr);
        if c == ag.digest.value { 
            Ok(()) 
        } else { 
            Err(anyhow!("digest mismatch: computed={}, recorded={}", c, ag.digest.value)) 
        }
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
            
            let sd = match self.researcher.materialize_top_level(refspec) { 
                Ok(s) => s, 
                Err(e) => {
                    results.push(sarif_err(format!("research: {}", e), &p)); 
                    continue; 
                }
            };
            
            let sr = Reporter::summarize(&sd);
            
            if let Err(e) = Mandator::enforce(&ag, &sd) {
                results.push(sarif_err(format!("mandate: {}", e), &p)); 
                continue;
            }
            
            if let Err(e) = Auditor::verify(&ag, &sr) {
                results.push(sarif_err(format!("audit: {}", e), &p)); 
                continue;
            }
            
            results.push(SarifResult {
                level: "note".into(),
                message: SarifMessage { text: "agreement ok".into() },
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
