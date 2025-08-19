use regex::Regex;

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
    println!("Pattern: '{}' -> Regex: '{}'", pat, rx);
    Regex::new(&rx).unwrap().is_match(s)
}

fn star_match_basename(pat: &str, path: &str) -> bool {
    let base = std::path::Path::new(path).file_name().and_then(|s| s.to_str()).unwrap_or("");
    println!("Path: '{}' -> Basename: '{}'", path, base);
    wildcard(pat, base)
}

fn main() {
    let test_cases = vec![
        ("*.md", "ARCHITECTURE_CLEARANCE_REPORT.md"),
        ("*.md", "README.md"),
        ("*.json", "checksums.json"),
        ("*.txt", "test.txt"),
    ];

    for (pattern, path) in test_cases {
        let result = star_match_basename(pattern, path);
        println!("'{}' matches '{}': {}", pattern, path, result);
        println!("---");
    }
}
