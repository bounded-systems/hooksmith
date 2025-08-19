use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

#[derive(Debug)]
struct BlobReuseInfo {
    hash: String,
    size: u64,
    reuse_count: u32,
    paths: Vec<String>,
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    
    match size {
        0..=KB-1 => format!("{} B", size),
        KB..=MB-1 => format!("{:.1} KB", size as f64 / KB as f64),
        _ => format!("{:.1} MB", size as f64 / MB as f64),
    }
}

fn get_blob_reuse_data() -> Result<Vec<BlobReuseInfo>, Box<dyn std::error::Error>> {
    // Get all objects with their paths
    let output = Command::new("git")
        .args(&["rev-list", "--objects", "--all"])
        .output()?;
    
    let output_str = String::from_utf8(output.stdout)?;
    let mut blob_counts: HashMap<String, u32> = HashMap::new();
    let mut blob_paths: HashMap<String, Vec<String>> = HashMap::new();
    
    // Count occurrences and collect paths
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let hash = parts[0].to_string();
            *blob_counts.entry(hash.clone()).or_insert(0) += 1;
            
            if parts.len() > 2 {
                let path = parts[2..].join(" ");
                blob_paths.entry(hash.clone()).or_insert_with(Vec::new).push(path);
            }
        }
    }
    
    // Get sizes for blobs only
    let mut blob_infos = Vec::new();
    for (hash, count) in blob_counts {
        // Check if it's a blob
        let type_output = Command::new("git")
            .args(&["cat-file", "-t", &hash])
            .output()?;
        
        let object_type = String::from_utf8(type_output.stdout)?.trim().to_string();
        
        if object_type == "blob" {
            let size_output = Command::new("git")
                .args(&["cat-file", "-s", &hash])
                .output()?;
            
            if let Ok(size_str) = String::from_utf8(size_output.stdout) {
                if let Ok(size) = u64::from_str(size_str.trim()) {
                    let paths = blob_paths.get(&hash).cloned().unwrap_or_default();
                    blob_infos.push(BlobReuseInfo {
                        hash,
                        size,
                        reuse_count: count,
                        paths,
                    });
                }
            }
        }
    }
    
    Ok(blob_infos)
}

fn analyze_reuse_by_size_category(blobs: &[BlobReuseInfo]) {
    let mut categories: HashMap<&str, Vec<&BlobReuseInfo>> = HashMap::new();
    
    for blob in blobs {
        let category = match blob.size {
            0..=1023 => "❌ < 1 KB",
            1024..=8191 => "⚠️  1-8 KB",
            8192..=204800 => "✅ 8-200 KB",
            204801..=1048576 => "⚠️  200 KB-1 MB",
            _ => "❌ > 1 MB",
        };
        categories.entry(category).or_insert_with(Vec::new).push(blob);
    }
    
    println!("📈 Size Categories with Reuse:");
    println!("===============================");
    
    for (category, blobs_in_category) in categories {
        let total_reuse: u32 = blobs_in_category.iter().map(|b| b.reuse_count).sum();
        let avg_reuse = if !blobs_in_category.is_empty() {
            total_reuse as f64 / blobs_in_category.len() as f64
        } else {
            0.0
        };
        
        println!("{}:", category);
        println!("   Average reuse: {:.1}x", avg_reuse);
    }
    println!();
}

fn show_most_reused_blobs(blobs: &[BlobReuseInfo], limit: usize) {
    let mut sorted_blobs: Vec<&BlobReuseInfo> = blobs.iter().collect();
    sorted_blobs.sort_by(|a, b| b.reuse_count.cmp(&a.reuse_count));
    
    println!("🔄 Most Reused Blobs (Deduplication Working):");
    println!("==============================================");
    
    for (i, blob) in sorted_blobs.iter().take(limit).enumerate() {
        let example_path = blob.paths.first().unwrap_or(&String::from("(no path)"));
        println!("{}. {} ({}x reused) - {}", 
            i + 1, 
            format_size(blob.size),
            blob.reuse_count,
            example_path);
    }
    
    if sorted_blobs.len() > limit {
        println!("... and {} more", sorted_blobs.len() - limit);
    }
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Git Blob Deduplication Analysis");
    println!("==================================");
    println!();
    
    let blobs = get_blob_reuse_data()?;
    
    if blobs.is_empty() {
        println!("❌ No blobs found. Are you in a Git repository?");
        return Ok(());
    }
    
    // Calculate overall statistics
    let total_references: u32 = blobs.iter().map(|b| b.reuse_count).sum();
    let unique_blobs = blobs.len() as u32;
    let reuse_ratio = if unique_blobs > 0 {
        total_references as f64 / unique_blobs as f64
    } else {
        0.0
    };
    
    println!("📊 Blob Reuse Statistics:");
    println!("==========================");
    println!("Total references: {}", total_references);
    println!("Unique blobs: {}", unique_blobs);
    println!("Average reuse: {:.2}x", reuse_ratio);
    println!();
    
    show_most_reused_blobs(&blobs, 10);
    analyze_reuse_by_size_category(&blobs);
    
    println!("💡 Key Insights:");
    println!("=================");
    println!("• Git automatically deduplicates identical blobs (content-addressed)");
    println!("• High reuse counts = efficient storage (no extra cost for duplicates)");
    println!("• Same blob can appear in multiple files, branches, or commits");
    println!("• Only unique content consumes additional storage space");
    println!("• Delta compression works on unique blobs, not references");
    println!();
    
    println!("🚀 Storage Efficiency:");
    println!("======================");
    if reuse_ratio > 1.5 {
        println!("✅ Good deduplication: {:.2}x average reuse", reuse_ratio);
    } else if reuse_ratio > 1.1 {
        println!("⚠️  Moderate deduplication: {:.2}x average reuse", reuse_ratio);
    } else {
        println!("❌ Low deduplication: {:.2}x average reuse", reuse_ratio);
    }
    
    Ok(())
}
