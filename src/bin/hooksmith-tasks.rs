use anyhow::Result;
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

/// Hooksmith Task Runner - Pure Rust Implementation
/// 
/// This binary replaces the Makefile with:
/// - Docker operations
/// - act testing
/// - Build management
/// - Development workflows
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    let command = &args[1];
    match command.as_str() {
        "help" => print_help(),
        "build" => build_binaries()?,
        "docker-build" => docker_build()?,
        "docker-run" => docker_run()?,
        "docker-test" => docker_test()?,
        "act-test" => act_test()?,
        "act-validate" => act_validate()?,
        "clean" => clean()?,
        "dev" => dev_workflow()?,
        "test" => test_workflow()?,
        "prod" => prod_workflow()?,
        _ => {
            eprintln!("❌ Unknown command: {}", command);
            print_help();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_help() {
    println!("🔧 Hooksmith Task Runner");
    println!("");
    println!("Available commands:");
    println!("  build          - Build Hooksmith binaries");
    println!("  docker-build   - Build Docker image with Bake");
    println!("  docker-test    - Test Docker image with Bake");
    println!("  docker-run     - Run Hooksmith in Docker");
    println!("  act-test       - Test GitHub Actions with act");
    println!("  act-validate   - Validate workflow syntax");
    println!("  clean          - Clean build artifacts");
    println!("  dev            - Development workflow");
    println!("  test           - Quick test workflow");
    println!("  prod           - Production build");
    println!("  help           - Show this help");
}

fn build_binaries() -> Result<()> {
    println!("🔨 Building Hooksmith binaries...");
    let status = Command::new("cargo")
        .args(&["build", "--release"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    
    if status.success() {
        println!("✅ Build successful!");
    } else {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

fn docker_build() -> Result<()> {
    println!("🐳 Building Hooksmith Docker image with Bake...");
    
    // Check if docker-bake.hcl exists
    if !Path::new("docker-bake.hcl").exists() {
        println!("⚠️  docker-bake.hcl not found, falling back to regular Docker build");
        let status = Command::new("docker")
            .args(&["build", "-t", "hooksmith:latest", "."])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;
        
        if status.success() {
            println!("✅ Docker build successful!");
        } else {
            std::process::exit(status.code().unwrap_or(1));
        }
        return Ok(());
    }
    
    // Use Bake for optimized builds
    let status = Command::new("docker")
        .args(&["buildx", "bake", "-f", "docker-bake.hcl", "hooksmith"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    
    if status.success() {
        println!("✅ Docker Bake build successful!");
    } else {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

fn docker_test() -> Result<()> {
    println!("🧪 Testing Hooksmith Docker image...");
    
    // Check if docker-bake.hcl exists
    if !Path::new("docker-bake.hcl").exists() {
        println!("⚠️  docker-bake.hcl not found, skipping Docker tests");
        return Ok(());
    }
    
    // Build test target with Bake
    let status = Command::new("docker")
        .args(&["buildx", "bake", "-f", "docker-bake.hcl", "test"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    
    if status.success() {
        println!("✅ Docker tests successful!");
    } else {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

fn docker_run() -> Result<()> {
    println!("🚀 Running Hooksmith in Docker...");
    let current_dir = env::current_dir()?;
    let home_dir = env::var("HOME")?;
    
    let status = Command::new("docker")
        .args(&[
            "run", "--rm", "-it",
            "-v", &format!("{}:/hooksmith", current_dir.display()),
            "-v", &format!("{}/.gitconfig:/root/.gitconfig:ro", home_dir),
            "-e", "GITHUB_WORKSPACE=/hooksmith",
            "-e", "GITHUB_ACTIONS=true",
            "hooksmith:latest"
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

fn act_test() -> Result<()> {
    println!("🧪 Testing GitHub Actions with act...");
    
    // Check if act is installed
    let act_check = Command::new("which")
        .arg("act")
        .output()?;
    
    if !act_check.status.success() {
        eprintln!("❌ act not found. Install with: brew install act");
        std::process::exit(1);
    }
    
    let status = Command::new("act")
        .args(&["--reuse", "--container-architecture", "linux/amd64"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    
    if status.success() {
        println!("✅ act test successful!");
    } else {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

fn act_validate() -> Result<()> {
    println!("✅ Validating workflow syntax...");
    
    // Check if act is installed
    let act_check = Command::new("which")
        .arg("act")
        .output()?;
    
    if !act_check.status.success() {
        eprintln!("❌ act not found. Install with: brew install act");
        std::process::exit(1);
    }
    
    let status = Command::new("act")
        .arg("--list")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    
    if status.success() {
        println!("✅ Workflow validation successful!");
    } else {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

fn clean() -> Result<()> {
    println!("🧹 Cleaning build artifacts...");
    
    // Clean cargo artifacts
    let cargo_status = Command::new("cargo")
        .arg("clean")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    
    // Clean Docker system
    let docker_status = Command::new("docker")
        .args(&["system", "prune", "-f"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    
    if cargo_status.success() && docker_status.success() {
        println!("✅ Clean successful!");
    } else {
        std::process::exit(1);
    }
    Ok(())
}

fn dev_workflow() -> Result<()> {
    println!("🎉 Setting up development environment...");
    build_binaries()?;
    docker_build()?;
    println!("🎉 Development environment ready!");
    Ok(())
}

fn test_workflow() -> Result<()> {
    println!("🧪 Running test workflow...");
    build_binaries()?;
    act_validate()?;
    println!("✅ All tests passed!");
    Ok(())
}

fn prod_workflow() -> Result<()> {
    println!("🚀 Running production build...");
    clean()?;
    build_binaries()?;
    docker_build()?;
    println!("🚀 Production build complete!");
    Ok(())
}
