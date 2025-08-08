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
    reuse_count: u32,
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
    // Get all blob hashes and sizes with reuse counts
    let output = Command::new("git")
        .args(&["rev-list", "--objects", "--all"])
        .output()?;
    
    let output_str = String::from_utf8(output.stdout)?;
    let mut blob_counts: HashMap<String, u32> = HashMap::new();
    let mut blob_sizes: HashMap<String, u64> = HashMap::new();
    let mut blob_paths: HashMap<String, Vec<String>> = HashMap::new();
    
    // First pass: count occurrences and collect paths
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
    
    // Second pass: get sizes for unique blobs
    for (hash, count) in &blob_counts {
        let size_output = Command::new("git")
            .args(&["cat-file", "-s", hash])
            .output()?;
        
        if let Ok(size_str) = String::from_utf8(size_output.stdout) {
            if let Ok(size) = u64::from_str(size_str.trim()) {
                blob_sizes.insert(hash.clone(), size);
            }
        }
    }
    
    // Build final blob info
    let mut blobs = Vec::new();
    for (hash, count) in blob_counts {
        if let Some(&size) = blob_sizes.get(&hash) {
            let category = BlobSizeCategory::from_size(size);
            let paths = blob_paths.get(&hash).cloned().unwrap_or_default();
            let path = paths.first().cloned();
            
            blobs.push(BlobInfo {
                hash,
                size,
                category,
                path,
                reuse_count: count,
            });
        }
    }
    
    Ok(blobs)
}

fn analyze_blob_distribution(blobs: &[BlobInfo]) {
    let mut category_counts: HashMap<BlobSizeCategory, u32> = HashMap::new();
    let mut category_sizes: HashMap<BlobSizeCategory, u64> = HashMap::new();
    let mut category_reuse: HashMap<BlobSizeCategory, u32> = HashMap::new();
    
    for blob in blobs {
        *category_counts.entry(blob.category).or_insert(0) += 1;
        *category_sizes.entry(blob.category).or_insert(0) += blob.size;
        *category_reuse.entry(blob.category).or_insert(0) += blob.reuse_count;
    }
    
    println!("🔍 Git Blob Size Analysis for Delta Compression");
    println!("=" .repeat(60));
    println!();
    
    let total_blobs = blobs.len() as u32;
    let total_size: u64 = blobs.iter().map(|b| b.size).sum();
    let total_references: u32 = blobs.iter().map(|b| b.reuse_count).sum();
    
    for category in [
        BlobSizeCategory::TooSmall,
        BlobSizeCategory::Mixed,
        BlobSizeCategory::SweetSpot,
        BlobSizeCategory::Varies,
        BlobSizeCategory::TooLarge,
    ] {
        let count = category_counts.get(&category).unwrap_or(&0);
        let size = category_sizes.get(&category).unwrap_or(&0);
        let reuse = category_reuse.get(&category).unwrap_or(&0);
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
        println!("   Unique blobs: {} ({:.1}%)", count, percentage);
        println!("   Total references: {} (avg {:.1} per blob)", reuse, 
            if *count > 0 { *reuse as f64 / *count as f64 } else { 0.0 });
        println!("   Size: {} ({:.1}%)", format_size(*size), size_percentage);
        println!("   Note: {}", category.description());
        println!();
    }
    
    println!("📊 Summary:");
    println!("   Total unique blobs: {}", total_blobs);
    println!("   Total references: {}", total_references);
    println!("   Average reuse: {:.1}x", 
        if total_blobs > 0 { total_references as f64 / total_blobs as f64 } else { 0.0 });
    println!("   Total size: {}", format_size(total_size));
    println!("   Sweet spot blobs: {} ({:.1}%)", 
        category_counts.get(&BlobSizeCategory::SweetSpot).unwrap_or(&0),
        if total_blobs > 0 {
            (*category_counts.get(&BlobSizeCategory::SweetSpot).unwrap_or(&0) as f64 / total_blobs as f64) * 100.0
        } else { 0.0 }
    );
}

fn show_high_reuse_blobs(blobs: &[BlobInfo], limit: usize) {
    let mut high_reuse: Vec<&BlobInfo> = blobs
        .iter()
        .filter(|b| b.reuse_count > 1)
        .collect();
    
    high_reuse.sort_by(|a, b| b.reuse_count.cmp(&a.reuse_count));
    
    if !high_reuse.is_empty() {
        println!("🔄 High Reuse Blobs (Deduplication Working):");
        println!("=" .repeat(60));
        
        for (i, blob) in high_reuse.iter().take(limit).enumerate() {
            println!("{}. {} ({}x reused) - {}", 
                i + 1, 
                format_size(blob.size),
                blob.reuse_count,
                blob.path.as_deref().unwrap_or("(no path)"));
        }
        
        if high_reuse.len() > limit {
            println!("... and {} more", high_reuse.len() - limit);
        }
        println!();
    }
}

fn show_sweet_spot_examples(blobs: &[BlobInfo], limit: usize) {
    let sweet_spot: Vec<&BlobInfo> = blobs
        .iter()
        .filter(|b| matches!(b.category, BlobSizeCategory::SweetSpot))
        .collect();
    
    println!("✅ Sweet Spot Examples (8-200 KB):");
    println!("=" .repeat(50));
    
    for (i, blob) in sweet_spot.iter().take(limit).enumerate() {
        let reuse_info = if blob.reuse_count > 1 {
            format!(" ({}x reused)", blob.reuse_count)
        } else {
            String::new()
        };
        println!("{}. {}{} - {}", 
            i + 1, 
            format_size(blob.size),
            reuse_info,
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
            let reuse_info = if blob.reuse_count > 1 {
                format!(" ({}x reused)", blob.reuse_count)
            } else {
                String::new()
            };
            println!("{}. {}{} - {}", 
                i + 1, 
                format_size(blob.size),
                reuse_info,
                blob.path.as_deref().unwrap_or("(no path)"));
        }
        
        if large_blobs.len() > limit {
            println!("... and {} more", large_blobs.len() - limit);
        }
        println!();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Analyzing Git blob sizes and deduplication patterns...");
    println!();
    
    let blobs = get_blob_sizes()?;
    
    if blobs.is_empty() {
        println!("❌ No blobs found. Are you in a Git repository?");
        return Ok(());
    }
    
    analyze_blob_distribution(&blobs);
    show_high_reuse_blobs(&blobs, 10);
    show_sweet_spot_examples(&blobs, 10);
    show_large_blob_examples(&blobs, 5);
    
    println!("💡 Recommendations:");
    println!("• Files in the 8-200 KB range are ideal for Git delta compression");
    println!("• Git automatically deduplicates identical blobs (content-addressed)");
    println!("• High reuse counts indicate efficient storage - no extra cost for duplicates");
    println!("• Consider Git LFS for files > 1 MB");
    println!("• Use 'git repack -ad --depth=50 --window=250' for optimal packing");
    
    Ok(())
}
