use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
enum BlobSizeCategory {
    TooSmall,      // < 1 KB
    Mixed,         // 1-8 KB
    SweetSpot,     // 8-200 KB
    Varies,        // 200 KB-1 MB
    TooLarge,      // > 1 MB
}

impl BlobSizeCategory {
    fn from_size(size_bytes: u64) -> Self {
        match size_bytes {
            0..=1023 => BlobSizeCategory::TooSmall,
            1024..=8191 => BlobSizeCategory::Mixed,
            8192..=204800 => BlobSizeCategory::SweetSpot,
            204801..=1048576 => BlobSizeCategory::Varies,
            _ => BlobSizeCategory::TooLarge,
        }
    }

    fn emoji(&self) -> &'static str {
        match self {
            BlobSizeCategory::TooSmall => "❌",
            BlobSizeCategory::Mixed => "⚠️",
            BlobSizeCategory::SweetSpot => "✅",
            BlobSizeCategory::Varies => "⚠️",
            BlobSizeCategory::TooLarge => "❌",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            BlobSizeCategory::TooSmall => "Too small; often cheaper to store raw",
            BlobSizeCategory::Mixed => "Only delta'd if near-identical",
            BlobSizeCategory::SweetSpot => "Sweet spot for delta compression; favored by Git",
            BlobSizeCategory::Varies => "Still usable; more expensive to compute",
            BlobSizeCategory::TooLarge => "Git avoids deltaing large blobs unless tweaked",
        }
    }
}

#[derive(Debug)]
struct BlobInfo {
    hash: String,
    size: u64,
    category: BlobSizeCategory,
    path: Option<String>,
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

fn get_blob_sizes() -> Result<Vec<BlobInfo>, Box<dyn std::error::Error>> {
    // Get all objects with their sizes using git cat-file
    let output = Command::new("git")
        .args(&["rev-list", "--objects", "--all"])
        .output()?;
    
    let output_str = String::from_utf8(output.stdout)?;
    let mut blobs = Vec::new();
    
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let hash = parts[0].to_string();
            
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
                        let category = BlobSizeCategory::from_size(size);
                        let path = if parts.len() > 2 {
                            Some(parts[2..].join(" "))
                        } else {
                            None
                        };
                        
                        blobs.push(BlobInfo {
                            hash,
                            size,
                            category,
                            path,
                        });
                    }
                }
            }
        }
    }
    
    Ok(blobs)
}

fn analyze_blob_distribution(blobs: &[BlobInfo]) {
    let mut category_counts: HashMap<BlobSizeCategory, u32> = HashMap::new();
    let mut category_sizes: HashMap<BlobSizeCategory, u64> = HashMap::new();
    
    for blob in blobs {
        *category_counts.entry(blob.category).or_insert(0) += 1;
        *category_sizes.entry(blob.category).or_insert(0) += blob.size;
    }
    
    println!("🔍 Git Blob Size Analysis - Goldilocks Zone");
    println!("============================================");
    println!();
    
    let total_blobs = blobs.len() as u32;
    let total_size: u64 = blobs.iter().map(|b| b.size).sum();
    
    for category in [
        BlobSizeCategory::TooSmall,
        BlobSizeCategory::Mixed,
        BlobSizeCategory::SweetSpot,
        BlobSizeCategory::Varies,
        BlobSizeCategory::TooLarge,
    ] {
        let count = category_counts.get(&category).unwrap_or(&0);
        let size = category_sizes.get(&category).unwrap_or(&0);
        let percentage = if total_blobs > 0 {
            (*count as f64 / total_blobs as f64) * 100.0
        } else {
            0.0
        };
        let size_percentage = if total_size > 0 {
            (*size as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };
        
        println!("{} {:?}:", category.emoji(), category);
        println!("   Count: {} ({:.1}%)", count, percentage);
        println!("   Size: {} ({:.1}%)", format_size(*size), size_percentage);
        println!("   Note: {}", category.description());
        println!();
    }
    
    println!("📊 Summary:");
    println!("   Total blobs: {}", total_blobs);
    println!("   Total size: {}", format_size(total_size));
    println!("   Sweet spot blobs: {} ({:.1}%)", 
        category_counts.get(&BlobSizeCategory::SweetSpot).unwrap_or(&0),
        if total_blobs > 0 {
            (*category_counts.get(&BlobSizeCategory::SweetSpot).unwrap_or(&0) as f64 / total_blobs as f64) * 100.0
        } else { 0.0 }
    );
}

fn show_sweet_spot_examples(blobs: &[BlobInfo], limit: usize) {
    let sweet_spot: Vec<&BlobInfo> = blobs
        .iter()
        .filter(|b| matches!(b.category, BlobSizeCategory::SweetSpot))
        .collect();
    
    println!("✅ Sweet Spot Examples (8-200 KB):");
    println!("=" .repeat(50));
    
    for (i, blob) in sweet_spot.iter().take(limit).enumerate() {
        println!("{}. {} - {}", 
            i + 1, 
            format_size(blob.size),
            blob.path.as_deref().unwrap_or("(no path)"));
    }
    
    if sweet_spot.len() > limit {
        println!("... and {} more", sweet_spot.len() - limit);
    }
    println!();
}

fn show_large_blob_examples(blobs: &[BlobInfo], limit: usize) {
    let large_blobs: Vec<&BlobInfo> = blobs
        .iter()
        .filter(|b| matches!(b.category, BlobSizeCategory::TooLarge))
        .collect();
    
    if !large_blobs.is_empty() {
        println!("❌ Large Blobs (>1 MB) - Consider Git LFS:");
        println!("=" .repeat(50));
        
        for (i, blob) in large_blobs.iter().take(limit).enumerate() {
            println!("{}. {} - {}", 
                i + 1, 
                format_size(blob.size),
                blob.path.as_deref().unwrap_or("(no path)"));
        }
        
        if large_blobs.len() > limit {
            println!("... and {} more", large_blobs.len() - limit);
        }
        println!();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Analyzing Git blob sizes for delta compression optimization...");
    println!();
    
    let blobs = get_blob_sizes()?;
    
    if blobs.is_empty() {
        println!("❌ No blobs found. Are you in a Git repository?");
        return Ok(());
    }
    
    analyze_blob_distribution(&blobs);
    show_sweet_spot_examples(&blobs, 10);
    show_large_blob_examples(&blobs, 5);
    
    println!("💡 Recommendations:");
    println!("• Files in the 8-200 KB range are ideal for Git delta compression");
    println!("• Consider Git LFS for files > 1 MB");
    println!("• Small files (< 1 KB) are often stored raw anyway");
    println!("• Use 'git repack -ad --depth=50 --window=250' for optimal packing");
    
    Ok(())
}
