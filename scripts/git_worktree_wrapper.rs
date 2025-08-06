#!/usr/bin/env rustc
//! Git Worktree Wrapper Script
//! This script provides guidance for worktree management

use std::env;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    
    // Colors for output
    let red = "\x1b[0;31m";
    let green = "\x1b[0;32m";
    let yellow = "\x1b[1;33m";
    let blue = "\x1b[0;34m";
    let purple = "\x1b[0;35m";
    let cyan = "\x1b[0;36m";
    let nc = "\x1b[0m"; // No Color
    
    show_worktree_guidance(&red, &green, &yellow, &blue, &purple, &cyan, &nc);
    
    // Check if any arguments were passed
    if args.is_empty() {
        println!();
        show_worktree_status(&blue, &cyan, &yellow);
    } else {
        println!();
        println!("{}🔧 To execute the original git worktree command, use:{}", yellow, nc);
        println!("{}git xworktree {}{}", green, args.join(" "), nc);
        println!();
        println!("{}Or use the worktree aliases:{}", yellow, nc);
        println!("{}git wtl{}  - List worktrees", cyan, nc);
        println!("{}git wtc{}  - Create worktree", cyan, nc);
        println!("{}git wts{}  - Switch worktree", cyan, nc);
        println!("{}git wtr{}  - Remove worktree", cyan, nc);
        println!();
    }
    
    Ok(())
}

fn show_worktree_guidance(red: &str, green: &str, yellow: &str, blue: &str, purple: &str, cyan: &str, nc: &str) {
    println!("{}🌳 Worktree Management{}", purple, nc);
    println!("{}=================={}", purple, nc);
    println!();
    println!("{}❌ Please use worktree commands instead of git worktree:{}", red, nc);
    println!();
    println!("{}  📋 List worktrees:     {}cargo xtask worktree list{}", cyan, green, nc);
    println!("{}  ➕ Create worktree:    {}cargo xtask worktree create --branch <branch>{}", cyan, green, nc);
    println!("{}  🔄 Switch worktree:    {}cargo xtask worktree switch --worktree <name>{}", cyan, green, nc);
    println!("{}  🗑️  Remove worktree:    {}cargo xtask worktree remove --worktree <name>{}", cyan, green, nc);
    println!("{}  🛠️  Setup tools:        {}cargo xtask worktree setup{}", cyan, green, nc);
    println!();
    println!("{}  📁 Or use {}git xworktree{} for direct git worktree access{}", yellow, green, yellow, nc);
    println!();
    println!("{}💡 Tip: Worktrees are now created in .wt/ directory by default{}", green, nc);
    println!();
    println!("{}🔧 Available aliases:{}", blue, nc);
    println!("{}  git wtl{}  - List worktrees", cyan, nc);
    println!("{}  git wtc{}  - Create worktree", cyan, nc);
    println!("{}  git wts{}  - Switch worktree", cyan, nc);
    println!("{}  git wtr{}  - Remove worktree", cyan, nc);
    println!();
}

fn show_worktree_status(blue: &str, cyan: &str, yellow: &str) {
    println!("{}📊 Current Worktree Status:{}", blue, "");
    println!("{}========================{}", blue, "");
    
    // Check if cargo is available
    if Command::new("cargo").arg("--version").output().is_ok() {
        println!("{}Running: cargo xtask worktree list{}", cyan, "");
        let output = Command::new("cargo")
            .args(["xtask", "worktree", "list"])
            .output();
        
        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    println!("{}", stdout);
                } else {
                    println!("{}⚠️  Cargo xtask not available{}", yellow, "");
                }
            }
            Err(_) => {
                println!("{}⚠️  Cargo xtask not available{}", yellow, "");
            }
        }
    } else {
        println!("{}⚠️  Cargo not available{}", yellow, "");
    }
    
    println!();
    println!("{}📁 .wt Directory Contents:{}", blue, "");
    
    // Check if .wt directory exists
    if std::path::Path::new(".wt").exists() {
        let output = Command::new("ls")
            .args(["-la", ".wt/"])
            .output();
        
        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    println!("{}", stdout);
                } else {
                    println!("{}⚠️  Cannot list .wt directory{}", yellow, "");
                }
            }
            Err(_) => {
                println!("{}⚠️  Cannot list .wt directory{}", yellow, "");
            }
        }
    } else {
        println!("{}⚠️  .wt directory not found{}", yellow, "");
    }
} 