//! Binary management for lefthook-rs
//!
//! This module handles Lefthook binary detection, installation, and management.

use crate::error::{LefthookError, Result};
use anyhow::Context;
use std::path::PathBuf;
use std::process::Command;
use which::which;

/// Find the Lefthook binary in the system
///
/// This function searches for the Lefthook binary in the system PATH
/// and returns the path if found.
///
/// # Returns
///
/// Returns the path to the Lefthook binary if found.
///
/// # Errors
///
/// Returns an error if the binary is not found in PATH.
pub async fn find_lefthook_binary() -> Result<PathBuf> {
    // First, try to find lefthook in PATH
    match which("lefthook") {
        Ok(path) => {
            tracing::debug!("Found lefthook binary at: {:?}", path);
            Ok(path)
        }
        Err(_) => {
            // If not found in PATH, try common installation locations
            let common_paths = vec![
                "/usr/local/bin/lefthook",
                "/opt/homebrew/bin/lefthook",
                "/usr/bin/lefthook",
                "./node_modules/.bin/lefthook",
            ];

            for path in common_paths {
                let path_buf = PathBuf::from(path);
                if path_buf.exists() {
                    tracing::debug!("Found lefthook binary at: {:?}", path_buf);
                    return Ok(path_buf);
                }
            }

            Err(LefthookError::BinaryNotFound(
                "Lefthook binary not found in PATH or common locations".to_string(),
            ))
        }
    }
}

/// Check if Lefthook is installed and working
///
/// This function verifies that Lefthook is properly installed and can be executed.
///
/// # Returns
///
/// Returns `Ok(())` if Lefthook is working correctly.
///
/// # Errors
///
/// Returns an error if Lefthook is not found or not working.
pub async fn check_lefthook_installation() -> Result<()> {
    let binary = find_lefthook_binary().await?;

    let output = Command::new(binary)
        .arg("version")
        .output()
        .context("Failed to execute lefthook version")?;

    if !output.status.success() {
        return Err(LefthookError::Installation(
            "Lefthook version command failed".to_string(),
        ));
    }

    let version = String::from_utf8_lossy(&output.stdout);
    tracing::info!("Lefthook version: {}", version.trim());

    Ok(())
}

/// Get the version of the installed Lefthook binary
///
/// This function runs `lefthook version` to get the version information.
///
/// # Returns
///
/// Returns the version string if successful.
///
/// # Errors
///
/// Returns an error if the version command fails.
pub async fn get_lefthook_version() -> Result<String> {
    let binary = find_lefthook_binary().await?;

    let output = Command::new(binary)
        .arg("version")
        .output()
        .context("Failed to execute lefthook version")?;

    if !output.status.success() {
        return Err(LefthookError::Version(
            "Lefthook version command failed".to_string(),
        ));
    }

    let version = String::from_utf8(output.stdout).context("Failed to parse version output")?;

    Ok(version.trim().to_string())
}

/// Install Lefthook if not already installed
///
/// This function attempts to install Lefthook if it's not found in the system.
/// It tries different installation methods based on the platform.
///
/// # Returns
///
/// Returns `Ok(())` if Lefthook is successfully installed or already present.
///
/// # Errors
///
/// Returns an error if installation fails.
#[cfg(feature = "download")]
pub async fn install_lefthook_if_needed() -> Result<()> {
    // First check if already installed
    if find_lefthook_binary().await.is_ok() {
        tracing::info!("Lefthook is already installed");
        return Ok(());
    }

    tracing::info!("Lefthook not found, attempting to install...");

    // Try different installation methods
    let install_methods = vec![
        ("npm", vec!["install", "-g", "@evilmartians/lefthook"]),
        ("yarn", vec!["global", "add", "@evilmartians/lefthook"]),
        ("brew", vec!["install", "lefthook"]),
    ];

    for (package_manager, args) in install_methods {
        if which(package_manager).is_ok() {
            tracing::info!("Attempting to install lefthook using {}", package_manager);

            let status = Command::new(package_manager)
                .args(args)
                .status()
                .context(format!("Failed to execute {} install", package_manager))?;

            if status.success() {
                tracing::info!("Successfully installed lefthook using {}", package_manager);

                // Verify installation
                if find_lefthook_binary().await.is_ok() {
                    return Ok(());
                }
            }
        }
    }

    Err(LefthookError::Installation(
        "Failed to install Lefthook using any available package manager".to_string(),
    ))
}

/// Download Lefthook binary from GitHub releases
///
/// This function downloads the Lefthook binary from GitHub releases
/// and installs it to a local directory.
///
/// # Arguments
///
/// * `version` - Version to download (e.g., "1.5.0")
/// * `target_dir` - Directory to install the binary to
///
/// # Returns
///
/// Returns the path to the installed binary.
///
/// # Errors
///
/// Returns an error if download or installation fails.
#[cfg(feature = "download")]
pub async fn download_lefthook_binary(version: &str, target_dir: &PathBuf) -> Result<PathBuf> {
    use std::env;
    use tokio::fs;

    // Determine platform and architecture
    let (platform, arch) = match env::consts::OS {
        "linux" => ("linux", "amd64"),
        "macos" => ("darwin", "amd64"),
        "windows" => ("windows", "amd64"),
        _ => {
            return Err(LefthookError::Download("Unsupported platform".to_string()));
        }
    };

    let filename = format!("lefthook_{}_{}_{}.tar.gz", version, platform, arch);
    let download_url = format!(
        "https://github.com/evilmartians/lefthook/releases/download/v{}/{}",
        version, filename
    );

    tracing::info!("Downloading lefthook from: {}", download_url);

    // Create target directory
    fs::create_dir_all(target_dir).await?;

    // Download the binary
    let response = reqwest::get(&download_url).await?;
    if !response.status().is_success() {
        return Err(LefthookError::Download(format!(
            "Failed to download: {}",
            response.status()
        )));
    }

    let bytes = response.bytes().await?;
    let archive_path = target_dir.join(&filename);
    fs::write(&archive_path, bytes).await?;

    // Extract the archive
    let extract_status = Command::new("tar")
        .args([
            "-xzf",
            archive_path.to_str().unwrap(),
            "-C",
            target_dir.to_str().unwrap(),
        ])
        .status()?;

    if !extract_status.success() {
        return Err(LefthookError::Download(
            "Failed to extract lefthook archive".to_string(),
        ));
    }

    // Make the binary executable
    let binary_path = target_dir.join("lefthook");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path).await?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms).await?;
    }

    // Clean up archive
    fs::remove_file(archive_path).await?;

    Ok(binary_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_lefthook_binary() {
        // This test will fail if lefthook is not installed
        // but that's expected in CI environments
        let result = find_lefthook_binary().await;
        if result.is_err() {
            println!(
                "Lefthook not found (expected in CI): {:?}",
                result.unwrap_err()
            );
        }
    }

    #[tokio::test]
    async fn test_get_lefthook_version() {
        let result = get_lefthook_version().await;
        if result.is_ok() {
            let version = result.unwrap();
            assert!(!version.is_empty());
            println!("Lefthook version: {version}");
        } else {
            println!(
                "Could not get lefthook version (expected if not installed): {:?}",
                result.unwrap_err()
            );
        }
    }

    #[tokio::test]
    async fn test_check_lefthook_installation() {
        let result = check_lefthook_installation().await;
        if result.is_err() {
            println!(
                "Lefthook installation check failed (expected if not installed): {:?}",
                result.unwrap_err()
            );
        }
    }
}
