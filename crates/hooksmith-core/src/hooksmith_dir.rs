use std::path::PathBuf;

/// Resolves the Hooksmith directory path.
/// 
/// Priority:
/// 1. HOOKSMITH_DIR environment variable (if set)
/// 2. Default: ".hooksmith" in current working directory
/// 
/// This function is used by all Hooksmith actors to locate configuration,
/// agreements, snapshots, and other Hooksmith-specific files.
pub fn hooksmith_dir() -> PathBuf {
    std::env::var_os("HOOKSMITH_DIR")
        .map(Into::into)
        .unwrap_or_else(|| ".hooksmith".into())
}

/// Resolves the agreements directory path.
/// 
/// Returns: `{hooksmith_dir}/agreements`
pub fn agreements_dir() -> PathBuf {
    hooksmith_dir().join("agreements")
}

/// Resolves the snapshots directory path.
/// 
/// Returns: `{hooksmith_dir}/snapshots`
pub fn snapshots_dir() -> PathBuf {
    hooksmith_dir().join("snapshots")
}

/// Resolves the actors configuration directory path.
/// 
/// Returns: `{hooksmith_dir}/actors`
pub fn actors_dir() -> PathBuf {
    hooksmith_dir().join("actors")
}

/// Resolves the cache directory path.
/// 
/// Returns: `{hooksmith_dir}/cache`
pub fn cache_dir() -> PathBuf {
    hooksmith_dir().join("cache")
}

/// Resolves the logs directory path.
/// 
/// Returns: `{hooksmith_dir}/logs`
pub fn logs_dir() -> PathBuf {
    hooksmith_dir().join("logs")
}

/// Resolves the hooks directory path.
/// 
/// Returns: `{hooksmith_dir}/hooks`
pub fn hooks_dir() -> PathBuf {
    hooksmith_dir().join("hooks")
}

/// Resolves the refs directory path.
/// 
/// Returns: `{hooksmith_dir}/refs`
pub fn refs_dir() -> PathBuf {
    hooksmith_dir().join("refs")
}

/// Ensures all Hooksmith directories exist.
/// 
/// Creates the directory structure if it doesn't exist:
/// ```
/// .hooksmith/
/// ├── agreements/
/// ├── actors/
/// ├── snapshots/
/// ├── cache/
/// ├── logs/
/// ├── hooks/
/// └── refs/
/// ```
pub fn ensure_hooksmith_dirs() -> std::io::Result<()> {
    let dirs = [
        hooksmith_dir(),
        agreements_dir(),
        actors_dir(),
        snapshots_dir(),
        cache_dir(),
        logs_dir(),
        hooks_dir(),
        refs_dir(),
    ];

    for dir in dirs {
        std::fs::create_dir_all(dir)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_hooksmith_dir_default() {
        // Clear any existing HOOKSMITH_DIR
        env::remove_var("HOOKSMITH_DIR");
        
        let dir = hooksmith_dir();
        assert_eq!(dir, PathBuf::from(".hooksmith"));
    }

    #[test]
    fn test_hooksmith_dir_env() {
        env::set_var("HOOKSMITH_DIR", "/custom/hooksmith");
        
        let dir = hooksmith_dir();
        assert_eq!(dir, PathBuf::from("/custom/hooksmith"));
        
        // Clean up
        env::remove_var("HOOKSMITH_DIR");
    }

    #[test]
    fn test_subdirectories() {
        env::remove_var("HOOKSMITH_DIR");
        
        assert_eq!(agreements_dir(), PathBuf::from(".hooksmith/agreements"));
        assert_eq!(snapshots_dir(), PathBuf::from(".hooksmith/snapshots"));
        assert_eq!(actors_dir(), PathBuf::from(".hooksmith/actors"));
        assert_eq!(cache_dir(), PathBuf::from(".hooksmith/cache"));
        assert_eq!(logs_dir(), PathBuf::from(".hooksmith/logs"));
        assert_eq!(hooks_dir(), PathBuf::from(".hooksmith/hooks"));
        assert_eq!(refs_dir(), PathBuf::from(".hooksmith/refs"));
    }
}
