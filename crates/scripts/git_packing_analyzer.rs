use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

#[derive(Debug)]
struct PackInfo {
    pack_name: String,
    object_count: u32,
    pack_size: u64,
    delta_count: u32,
    chain_depth: u32,
}

#[derive(Debug)]
struct DeltaChainInfo {
    base_hash: String,
    chain_length: u32,
    total_size: u64,
    compressed_size: u64,
    paths: Vec<String>,
}

#[derive(Debug)]
struct BlobSizeProfile {
    size_range: String,
    count: u32,
    total_size: u64,
    avg_delta_savings: f64,
    excluded_from_delta: u32,
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;
    
    match size {
        0..=KB-1 => format!("{} B", size),
        KB..=MB-1 => format!("{:.1} KB", size as f64 / KB as f64),
        MB..=GB-1 => format!("{:.1} MB", size as f64 / MB as f64),
        _ => format!("{:.1} GB", size as f64 / GB as f64),
    }
}

fn get_pack_info() -> Result<Vec<PackInfo>, Box<dyn std::error::Error>> {
    let mut packs = Vec::new();
    
    // Find pack files
    let pack_output = Command::new("find")
        .args(&[".git/objects/pack", "-name", "*.pack", "-type", "f"])
        .output()?;
    
    let pack_files = String::from_utf8(pack_output.stdout)?;
    
    for pack_file in pack_files.lines() {
        if pack_file.is_empty() {
            continue;
        }
        
        // Get pack info using git verify-pack
        let verify_output = Command::new("git")
            .args(&["verify-pack", "-v", pack_file])
            .output()?;
        
        let verify_str = String::from_utf8(verify_output.stdout)?;
        let lines: Vec<&str> = verify_str.lines().collect();
        
        if lines.len() > 1 {
            // Parse header line for pack stats
            let header = lines[0];
            if header.contains("pack") {
                let parts: Vec<&str> = header.split_whitespace().collect();
                if parts.len() >= 4 {
                    let object_count = u32::from_str(parts[1]).unwrap_or(0);
                    let pack_size = u64::from_str(parts[2]).unwrap_or(0);
                    
                    // Count deltas and find max chain depth
                    let mut delta_count = 0;
                    let mut max_chain_depth = 0;
                    
                    for line in &lines[1..] {
                        if line.contains("delta") {
                            delta_count += 1;
                            // Parse delta chain depth if available
                            if let Some(depth_str) = line.split_whitespace().nth(4) {
                                if let Ok(depth) = u32::from_str(depth_str) {
                                    max_chain_depth = max_chain_depth.max(depth);
                                }
                            }
                        }
                    }
                    
                    let pack_name = pack_file.split('/').last().unwrap_or("unknown").to_string();
                    packs.push(PackInfo {
                        pack_name,
                        object_count,
                        pack_size,
                        delta_count,
                        chain_depth: max_chain_depth,
                    });
                }
            }
        }
    }
    
    Ok(packs)
}

fn analyze_delta_chains() -> Result<Vec<DeltaChainInfo>, Box<dyn std::error::Error>> {
    let mut chains = Vec::new();
    
    // Get detailed pack info
    let verify_output = Command::new("git")
        .args(&["verify-pack", "-v", ".git/objects/pack/*.idx"])
        .output()?;
    
    let verify_str = String::from_utf8(verify_output.stdout)?;
    let mut current_chain: Option<DeltaChainInfo> = None;
    
    for line in verify_str.lines() {
        if line.contains("delta") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let hash = parts[0].to_string();
                let size = u64::from_str(parts[1]).unwrap_or(0);
                let compressed = u64::from_str(parts[2]).unwrap_or(0);
                let base_hash = parts[3].to_string();
                
                if let Some(chain) = &mut current_chain {
                    if chain.base_hash == base_hash {
                        // Continue current chain
                        chain.chain_length += 1;
                        chain.total_size += size;
                        chain.compressed_size += compressed;
                    } else {
                        // Start new chain
                        if chain.chain_length > 1 {
                            chains.push(current_chain.take().unwrap());
                        }
                        current_chain = Some(DeltaChainInfo {
                            base_hash,
                            chain_length: 1,
                            total_size: size,
                            compressed_size: compressed,
                            paths: Vec::new(),
                        });
                    }
                } else {
                    // Start first chain
                    current_chain = Some(DeltaChainInfo {
                        base_hash,
                        chain_length: 1,
                        total_size: size,
                        compressed_size: compressed,
                        paths: Vec::new(),
                    });
                }
            }
        }
    }
    
    // Add final chain
    if let Some(chain) = current_chain {
        if chain.chain_length > 1 {
            chains.push(chain);
        }
    }
    
    Ok(chains)
}

fn analyze_blob_size_profiles() -> Result<Vec<BlobSizeProfile>, Box<dyn std::error::Error>> {
    let mut profiles = Vec::new();
    
    // Define size ranges for analysis
    let ranges = vec![
        ("< 1 KB", 0..1024),
        ("1-8 KB", 1024..8192),
        ("8-200 KB", 8192..204800),
        ("200 KB-1 MB", 204800..1048576),
        ("> 1 MB", 1048576..u64::MAX),
    ];
    
    // Get all blob sizes
    let output = Command::new("git")
        .args(&["rev-list", "--objects", "--all"])
        .output()?;
    
    let output_str = String::from_utf8(output.stdout)?;
    let mut size_counts: HashMap<&str, u32> = HashMap::new();
    let mut size_totals: HashMap<&str, u64> = HashMap::new();
    let mut excluded_counts: HashMap<&str, u32> = HashMap::new();
    
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
                        // Find which range this belongs to
                        for (range_name, range) in &ranges {
                            if range.contains(&size) {
                                *size_counts.entry(range_name).or_insert(0) += 1;
                                *size_totals.entry(range_name).or_insert(0) += size;
                                
                                // Check if file is excluded from delta compression
                                if let Some(path) = parts.get(2) {
                                    if path.ends_with(".zip") || path.ends_with(".bin") || 
                                       path.ends_with(".tar.gz") || path.ends_with(".iso") {
                                        *excluded_counts.entry(range_name).or_insert(0) += 1;
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Build profiles
    for (range_name, _) in ranges {
        let count = size_counts.get(range_name).unwrap_or(&0);
        let total_size = size_totals.get(range_name).unwrap_or(&0);
        let excluded = excluded_counts.get(range_name).unwrap_or(&0);
        
        profiles.push(BlobSizeProfile {
            size_range: range_name.to_string(),
            count: *count,
            total_size: *total_size,
            avg_delta_savings: 0.0, // Would need more complex analysis
            excluded_from_delta: *excluded,
        });
    }
    
    Ok(profiles)
}

fn show_pack_analysis(packs: &[PackInfo]) {
    println!("📦 Packfile Analysis:");
    println!("=====================");
    
    let total_objects: u32 = packs.iter().map(|p| p.object_count).sum();
    let total_size: u64 = packs.iter().map(|p| p.pack_size).sum();
    let total_deltas: u32 = packs.iter().map(|p| p.delta_count).sum();
    let max_depth = packs.iter().map(|p| p.chain_depth).max().unwrap_or(0);
    
    println!("Total packfiles: {}", packs.len());
    println!("Total objects: {}", total_objects);
    println!("Total size: {}", format_size(total_size));
    println!("Total deltas: {} ({:.1}%)", total_deltas, 
        if total_objects > 0 { (total_deltas as f64 / total_objects as f64) * 100.0 } else { 0.0 });
    println!("Max delta chain depth: {}", max_depth);
    println!();
    
    for pack in packs {
        let delta_percentage = if pack.object_count > 0 {
            (pack.delta_count as f64 / pack.object_count as f64) * 100.0
        } else { 0.0 };
        
        println!("📦 {}:", pack.pack_name);
        println!("   Objects: {}", pack.object_count);
        println!("   Size: {}", format_size(pack.pack_size));
        println!("   Deltas: {} ({:.1}%)", pack.delta_count, delta_percentage);
        println!("   Max chain depth: {}", pack.chain_depth);
        println!();
    }
}

fn show_delta_chain_analysis(chains: &[DeltaChainInfo]) {
    println!("🔗 Delta Chain Analysis:");
    println!("========================");
    
    if chains.is_empty() {
        println!("No significant delta chains found.");
        println!();
        return;
    }
    
    let total_chains = chains.len();
    let avg_chain_length: f64 = chains.iter().map(|c| c.chain_length as f64).sum::<f64>() / total_chains as f64;
    let max_chain_length = chains.iter().map(|c| c.chain_length).max().unwrap_or(0);
    
    println!("Total chains: {}", total_chains);
    println!("Average chain length: {:.1}", avg_chain_length);
    println!("Max chain length: {}", max_chain_length);
    println!();
    
    // Show longest chains
    let mut sorted_chains: Vec<&DeltaChainInfo> = chains.iter().collect();
    sorted_chains.sort_by(|a, b| b.chain_length.cmp(&a.chain_length));
    
    println!("🔗 Longest Delta Chains:");
    for (i, chain) in sorted_chains.iter().take(5).enumerate() {
        let savings = if chain.total_size > 0 {
            ((chain.total_size - chain.compressed_size) as f64 / chain.total_size as f64) * 100.0
        } else { 0.0 };
        
        println!("{}. {} objects, {} → {} ({:.1}% savings)", 
            i + 1, 
            chain.chain_length,
            format_size(chain.total_size),
            format_size(chain.compressed_size),
            savings);
    }
    println!();
}

fn show_blob_size_profiles(profiles: &[BlobSizeProfile]) {
    println!("📊 Blob Size Profile Analysis:");
    println!("==============================");
    
    for profile in profiles {
        let excluded_percentage = if profile.count > 0 {
            (profile.excluded_from_delta as f64 / profile.count as f64) * 100.0
        } else { 0.0 };
        
        println!("{}:", profile.size_range);
        println!("   Count: {}", profile.count);
        println!("   Total size: {}", format_size(profile.total_size));
        println!("   Excluded from delta: {} ({:.1}%)", profile.excluded_from_delta, excluded_percentage);
        println!();
    }
}

fn show_optimization_recommendations(packs: &[PackInfo], chains: &[DeltaChainInfo], profiles: &[BlobSizeProfile]) {
    println!("⚙️  Optimization Recommendations:");
    println!("===============================");
    
    // Analyze pack efficiency
    let total_objects: u32 = packs.iter().map(|p| p.object_count).sum();
    let total_deltas: u32 = packs.iter().map(|p| p.delta_count).sum();
    let delta_ratio = if total_objects > 0 { total_deltas as f64 / total_objects as f64 } else { 0.0 };
    
    if delta_ratio < 0.3 {
        println!("🔧 Low delta usage detected:");
        println!("   • Consider: git repack -Ad --window=250 --depth=50");
        println!("   • Increase delta compression for better pack efficiency");
    } else if delta_ratio > 0.7 {
        println!("🔧 High delta usage detected:");
        println!("   • Consider: git repack -d -l -f");
        println!("   • Balance between size and access speed");
    }
    
    // Check chain depths
    let max_depth = chains.iter().map(|c| c.chain_length).max().unwrap_or(0);
    if max_depth > 50 {
        println!("🔧 Deep delta chains detected:");
        println!("   • Consider: git repack --depth=50");
        println!("   • Deep chains slow down checkouts");
    }
    
    // Check for large files
    let large_files = profiles.iter().find(|p| p.size_range == "> 1 MB");
    if let Some(large) = large_files {
        if large.count > 0 {
            println!("🔧 Large files detected:");
            println!("   • Consider Git LFS for files > 1 MB");
            println!("   • Add to .gitattributes: *.large -delta");
        }
    }
    
    // Check excluded files
    let total_excluded: u32 = profiles.iter().map(|p| p.excluded_from_delta).sum();
    if total_excluded > 0 {
        println!("🔧 Files excluded from delta compression:");
        println!("   • {} files are stored as full blobs", total_excluded);
        println!("   • This is normal for compressed binaries");
    }
    
    println!();
    println!("🚀 Pro Tips:");
    println!("• Use 'git repack -Ad --window=250 --depth=50' for full optimization");
    println!("• Use 'git repack -d -l -f' for fast, light repacking");
    println!("• Add '*.zip -delta' to .gitattributes for compressed files");
    println!("• Monitor with 'git verify-pack -v .git/objects/pack/*.idx'");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Git Packing 201: Advanced Analysis");
    println!("=====================================");
    println!();
    
    // Get pack information
    let packs = get_pack_info()?;
    if packs.is_empty() {
        println!("❌ No packfiles found. Run 'git repack' first.");
        return Ok(());
    }
    
    // Get delta chain information
    let chains = analyze_delta_chains()?;
    
    // Get blob size profiles
    let profiles = analyze_blob_size_profiles()?;
    
    show_pack_analysis(&packs);
    show_delta_chain_analysis(&chains);
    show_blob_size_profiles(&profiles);
    show_optimization_recommendations(&packs, &chains, &profiles);
    
    println!("💡 Key Insights:");
    println!("=================");
    println!("• Packing uses heuristics, not exhaustive search");
    println!("• Similar-sized blobs compress better together");
    println!("• Path similarity helps with delta reuse");
    println!("• Deep delta chains cost on access");
    println!("• Large files often become anchor nodes");
    println!("• Split packfiles scale better for large repos");
    
    Ok(())
}
