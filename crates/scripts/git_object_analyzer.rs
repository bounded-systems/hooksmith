use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ObjectType {
    Blob,
    Tree,
    Commit,
    Tag,
    Note,
}

impl ObjectType {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "blob" => Some(ObjectType::Blob),
            "tree" => Some(ObjectType::Tree),
            "commit" => Some(ObjectType::Commit),
            "tag" => Some(ObjectType::Tag),
            _ => None, // Notes are stored as commits/trees
        }
    }

    fn emoji(&self) -> &'static str {
        match self {
            ObjectType::Blob => "📄",
            ObjectType::Tree => "📁",
            ObjectType::Commit => "🔗",
            ObjectType::Tag => "🏷️",
            ObjectType::Note => "📝",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            ObjectType::Blob => "File contents",
            ObjectType::Tree => "Directory listing",
            ObjectType::Commit => "Repository snapshot",
            ObjectType::Tag => "Named reference",
            ObjectType::Note => "Side-channel metadata",
        }
    }

    fn is_delta_able(&self) -> bool {
        matches!(self, ObjectType::Blob)
    }
}

#[derive(Debug)]
struct ObjectInfo {
    hash: String,
    object_type: ObjectType,
    size: u64,
    compressed_size: u64,
    delta_base: Option<String>,
    delta_depth: u32,
    paths: Vec<String>,
    reuse_count: u32,
}

#[derive(Debug)]
struct ObjectRelationship {
    from_hash: String,
    from_type: ObjectType,
    to_hash: String,
    to_type: ObjectType,
    relationship: String,
}

#[derive(Debug)]
struct PackStatistics {
    total_objects: u32,
    total_size: u64,
    total_compressed: u64,
    by_type: HashMap<ObjectType, u32>,
    delta_count: u32,
    max_delta_depth: u32,
    compression_ratio: f64,
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

fn get_all_objects() -> Result<Vec<ObjectInfo>, Box<dyn std::error::Error>> {
    let mut objects = Vec::new();
    let mut object_counts: HashMap<String, u32> = HashMap::new();
    let mut object_paths: HashMap<String, Vec<String>> = HashMap::new();
    
    // Get all objects with their references
    let output = Command::new("git")
        .args(&["rev-list", "--objects", "--all"])
        .output()?;
    
    let output_str = String::from_utf8(output.stdout)?;
    
    // Count references and collect paths
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let hash = parts[0].to_string();
            *object_counts.entry(hash.clone()).or_insert(0) += 1;
            
            if parts.len() > 2 {
                let path = parts[2..].join(" ");
                object_paths.entry(hash.clone()).or_insert_with(Vec::new).push(path);
            }
        }
    }
    
    // Get detailed object information
    for (hash, count) in object_counts {
        let type_output = Command::new("git")
            .args(&["cat-file", "-t", &hash])
            .output()?;
        
        let object_type_str = String::from_utf8(type_output.stdout)?.trim().to_string();
        
        if let Some(object_type) = ObjectType::from_str(&object_type_str) {
            let size_output = Command::new("git")
                .args(&["cat-file", "-s", &hash])
                .output()?;
            
            if let Ok(size_str) = String::from_utf8(size_output.stdout) {
                if let Ok(size) = u64::from_str(size_str.trim()) {
                    let paths = object_paths.get(&hash).cloned().unwrap_or_default();
                    
                    objects.push(ObjectInfo {
                        hash,
                        object_type,
                        size,
                        compressed_size: size, // Will be updated from pack info
                        delta_base: None,
                        delta_depth: 0,
                        paths,
                        reuse_count: count,
                    });
                }
            }
        }
    }
    
    Ok(objects)
}

fn get_pack_delta_info(objects: &mut [ObjectInfo]) -> Result<(), Box<dyn std::error::Error>> {
    // Get pack information to update delta details
    let verify_output = Command::new("git")
        .args(&["verify-pack", "-v", ".git/objects/pack/*.idx"])
        .output()?;
    
    let verify_str = String::from_utf8(verify_output.stdout)?;
    
    for line in verify_str.lines() {
        if line.contains("delta") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let hash = parts[0];
                let compressed = u64::from_str(parts[2]).unwrap_or(0);
                let base_hash = parts[3].to_string();
                
                // Find and update the object
                if let Some(obj) = objects.iter_mut().find(|o| o.hash == hash) {
                    obj.compressed_size = compressed;
                    obj.delta_base = Some(base_hash);
                    
                    // Calculate delta depth
                    let mut depth = 1;
                    let mut current_base = &base_hash;
                    while let Some(base_obj) = objects.iter().find(|o| o.hash == *current_base) {
                        if let Some(ref base) = base_obj.delta_base {
                            depth += 1;
                            current_base = base;
                        } else {
                            break;
                        }
                    }
                    obj.delta_depth = depth;
                }
            }
        }
    }
    
    Ok(())
}

fn analyze_object_relationships(objects: &[ObjectInfo]) -> Result<Vec<ObjectRelationship>, Box<dyn std::error::Error>> {
    let mut relationships = Vec::new();
    
    // Analyze commit -> tree relationships
    for obj in objects.iter().filter(|o| o.object_type == ObjectType::Commit) {
        let commit_content = Command::new("git")
            .args(&["cat-file", "-p", &obj.hash])
            .output()?;
        
        let content = String::from_utf8(commit_content.stdout)?;
        for line in content.lines() {
            if line.starts_with("tree ") {
                let tree_hash = line.split_whitespace().nth(1).unwrap_or("");
                if let Some(tree_obj) = objects.iter().find(|o| o.hash == tree_hash) {
                    relationships.push(ObjectRelationship {
                        from_hash: obj.hash.clone(),
                        from_type: ObjectType::Commit,
                        to_hash: tree_hash.to_string(),
                        to_type: ObjectType::Tree,
                        relationship: "points to".to_string(),
                    });
                }
            }
        }
    }
    
    // Analyze tree -> blob/tree relationships
    for obj in objects.iter().filter(|o| o.object_type == ObjectType::Tree) {
        let tree_content = Command::new("git")
            .args(&["cat-file", "-p", &obj.hash])
            .output()?;
        
        let content = String::from_utf8(tree_content.stdout)?;
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let mode = parts[0];
                let hash = parts[2];
                let name = parts[3];
                
                let target_type = if mode.starts_with("04") {
                    ObjectType::Tree
                } else {
                    ObjectType::Blob
                };
                
                if let Some(target_obj) = objects.iter().find(|o| o.hash == hash) {
                    relationships.push(ObjectRelationship {
                        from_hash: obj.hash.clone(),
                        from_type: ObjectType::Tree,
                        to_hash: hash.to_string(),
                        to_type: target_type,
                        relationship: format!("contains {}", name),
                    });
                }
            }
        }
    }
    
    Ok(relationships)
}

fn calculate_pack_statistics(objects: &[ObjectInfo]) -> PackStatistics {
    let mut by_type: HashMap<ObjectType, u32> = HashMap::new();
    let mut delta_count = 0;
    let mut max_delta_depth = 0;
    let mut total_size = 0;
    let mut total_compressed = 0;
    
    for obj in objects {
        *by_type.entry(obj.object_type).or_insert(0) += 1;
        total_size += obj.size;
        total_compressed += obj.compressed_size;
        
        if obj.delta_base.is_some() {
            delta_count += 1;
            max_delta_depth = max_delta_depth.max(obj.delta_depth);
        }
    }
    
    let compression_ratio = if total_size > 0 {
        (total_size - total_compressed) as f64 / total_size as f64
    } else {
        0.0
    };
    
    PackStatistics {
        total_objects: objects.len() as u32,
        total_size,
        total_compressed,
        by_type,
        delta_count,
        max_delta_depth,
        compression_ratio,
    }
}

fn show_object_type_analysis(stats: &PackStatistics) {
    println!("📊 Object Type Analysis:");
    println!("========================");
    
    for object_type in [ObjectType::Blob, ObjectType::Tree, ObjectType::Commit, ObjectType::Tag] {
        let count = stats.by_type.get(&object_type).unwrap_or(&0);
        let percentage = if stats.total_objects > 0 {
            (*count as f64 / stats.total_objects as f64) * 100.0
        } else {
            0.0
        };
        
        println!("{} {}: {} ({:.1}%)", 
            object_type.emoji(), 
            object_type.description(),
            count,
            percentage);
    }
    println!();
}

fn show_delta_analysis(stats: &PackStatistics) {
    println!("🔗 Delta Compression Analysis:");
    println!("==============================");
    
    println!("Total objects: {}", stats.total_objects);
    println!("Delta-compressed: {} ({:.1}%)", 
        stats.delta_count,
        if stats.total_objects > 0 { (stats.delta_count as f64 / stats.total_objects as f64) * 100.0 } else { 0.0 });
    println!("Max delta chain depth: {}", stats.max_delta_depth);
    println!("Compression ratio: {:.1}%", stats.compression_ratio * 100.0);
    println!("Size savings: {} → {} ({} saved)", 
        format_size(stats.total_size),
        format_size(stats.total_compressed),
        format_size(stats.total_size - stats.total_compressed));
    println!();
}

fn show_object_relationships(relationships: &[ObjectRelationship]) {
    println!("🔗 Object Relationships:");
    println!("========================");
    
    let mut commit_tree_count = 0;
    let mut tree_blob_count = 0;
    let mut tree_tree_count = 0;
    
    for rel in relationships {
        match (rel.from_type, rel.to_type) {
            (ObjectType::Commit, ObjectType::Tree) => commit_tree_count += 1,
            (ObjectType::Tree, ObjectType::Blob) => tree_blob_count += 1,
            (ObjectType::Tree, ObjectType::Tree) => tree_tree_count += 1,
            _ => {}
        }
    }
    
    println!("Commit → Tree relationships: {}", commit_tree_count);
    println!("Tree → Blob relationships: {}", tree_blob_count);
    println!("Tree → Tree relationships: {}", tree_tree_count);
    println!();
}

fn show_optimization_recommendations(stats: &PackStatistics) {
    println!("⚙️  Optimization Recommendations:");
    println!("===============================");
    
    // Delta compression analysis
    let delta_ratio = if stats.total_objects > 0 {
        stats.delta_count as f64 / stats.total_objects as f64
    } else { 0.0 };
    
    if delta_ratio < 0.2 {
        println!("🔧 Low delta usage detected:");
        println!("   • Consider: git repack -Ad --window=250 --depth=50");
        println!("   • Increase delta compression for better efficiency");
    } else if delta_ratio > 0.8 {
        println!("🔧 High delta usage detected:");
        println!("   • Consider: git repack -d -l -f");
        println!("   • Balance between size and access speed");
    }
    
    // Delta chain depth analysis
    if stats.max_delta_depth > 50 {
        println!("🔧 Deep delta chains detected:");
        println!("   • Consider: git repack --depth=50");
        println!("   • Deep chains slow down checkouts");
    }
    
    // Compression efficiency
    if stats.compression_ratio < 0.3 {
        println!("🔧 Low compression ratio detected:");
        println!("   • Consider excluding binary files from delta compression");
        println!("   • Add to .gitattributes: *.bin -delta, *.zip -delta");
    }
    
    println!();
    println!("🚀 Pro Tips:");
    println!("• Use 'git repack -Ad --window=250 --depth=50' for full optimization");
    println!("• Use 'git repack -d -l -f' for fast, light repacking");
    println!("• Add '*.zip -delta' to .gitattributes for compressed files");
    println!("• Monitor with 'git verify-pack -v .git/objects/pack/*.idx'");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Git Object Model and Packing Analysis");
    println!("========================================");
    println!();
    
    // Get all objects
    let mut objects = get_all_objects()?;
    if objects.is_empty() {
        println!("❌ No objects found. Are you in a Git repository?");
        return Ok(());
    }
    
    // Get delta information from packfiles
    get_pack_delta_info(&mut objects)?;
    
    // Analyze relationships
    let relationships = analyze_object_relationships(&objects)?;
    
    // Calculate statistics
    let stats = calculate_pack_statistics(&objects);
    
    show_object_type_analysis(&stats);
    show_delta_analysis(&stats);
    show_object_relationships(&relationships);
    show_optimization_recommendations(&stats);
    
    println!("💡 Key Insights:");
    println!("=================");
    println!("• Blobs are the only objects that can be delta-compressed");
    println!("• Trees and commits are stored as-is (no deltas)");
    println!("• Multiple commits can share the same tree structure");
    println!("• Identical blobs are automatically deduplicated");
    println!("• Path similarity improves delta reuse");
    println!("• Notes are stored as commits/trees internally");
    
    Ok(())
}
