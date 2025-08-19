use git_filter::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🔗 Unified Contracts Demo - Flat Contracts with Shared Primitives\n");

    // Example 1: Shared primitives demonstration
    demo_shared_primitives()?;

    // Example 2: Blob contracts with character validation
    demo_blob_contracts()?;

    // Example 3: Tree contracts with type-safe modes
    demo_tree_contracts()?;

    // Example 4: Commit contracts
    demo_commit_contracts()?;

    // Example 5: Tag contracts
    demo_tag_contracts()?;

    // Example 6: Unified Git object enum
    demo_unified_git_objects()?;

    // Example 7: Unified validator
    demo_unified_validator()?;

    // Example 8: Serialization and deserialization
    demo_serialization()?;

    Ok(())
}

fn demo_shared_primitives() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Example 1: Shared Primitives Demonstration");

    // Test SHA-1 regex
    let valid_sha1 = "a1b2c3d4e5f6789012345678901234567890abcd";
    let invalid_sha1 = "invalid-hash";

    println!("  SHA-1 validation:");
    println!("    '{}' -> {}", valid_sha1, SHA1_RE.is_match(valid_sha1));
    println!(
        "    '{}' -> {}",
        invalid_sha1,
        SHA1_RE.is_match(invalid_sha1)
    );

    // Test filename regex
    let valid_filenames = vec!["README.md", "src/main.rs", "file-name.txt", "file_name.txt"];
    let invalid_filenames = vec!["file\x00name.txt", "file with spaces.txt"];

    println!("  Filename validation:");
    for filename in valid_filenames {
        println!(
            "    '{}' -> {}",
            filename,
            VALID_FILENAME_RE.is_match(filename)
        );
    }
    for filename in invalid_filenames {
        println!(
            "    '{}' -> {}",
            filename,
            VALID_FILENAME_RE.is_match(filename)
        );
    }

    // Test character regex
    let valid_chars = vec!['a', 'Z', '5', '!', ' ', '\t'];
    let invalid_chars = vec!['\x00', '\x1f', '\x7f', 'é', 'ñ'];

    println!("  Character validation:");
    for ch in valid_chars {
        let mut buf = [0u8; 4];
        let encoded = ch.encode_utf8(&mut buf);
        println!("    '{:?}' -> {}", ch, VALID_CHAR_RE.is_match(encoded));
    }
    for ch in invalid_chars {
        let mut buf = [0u8; 4];
        let encoded = ch.encode_utf8(&mut buf);
        println!("    '{:?}' -> {}", ch, VALID_CHAR_RE.is_match(encoded));
    }

    println!();
    Ok(())
}

fn demo_blob_contracts() -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Example 2: Blob Contracts with Character Validation");

    // Valid blob line
    let valid_line = UnifiedBlobLineContract {
        line: "Hello, World!".to_string(),
    };
    println!("  {}", valid_line.summary());

    // Invalid blob line
    let invalid_line = UnifiedBlobLineContract {
        line: "Hello\x00World!".to_string(),
    };
    println!("  {}", invalid_line.summary());
    if !invalid_line.get_errors().is_empty() {
        println!("    Errors: {:?}", invalid_line.get_errors());
    }

    // Valid blob
    let valid_blob = UnifiedBlobContract {
        oid: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        size: 13,
        lines: vec![
            UnifiedBlobLineContract {
                line: "Hello, ".to_string(),
            },
            UnifiedBlobLineContract {
                line: "World!".to_string(),
            },
        ],
    };
    println!("  {}", valid_blob.summary());

    // Invalid blob
    let invalid_blob = UnifiedBlobContract {
        oid: "invalid-hash".to_string(),
        size: 0,
        lines: vec![UnifiedBlobLineContract {
            line: "Hello\x00World!".to_string(),
        }],
    };
    println!("  {}", invalid_blob.summary());
    if !invalid_blob.get_errors().is_empty() {
        println!("    Errors: {:?}", invalid_blob.get_errors());
    }

    println!();
    Ok(())
}

fn demo_tree_contracts() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌳 Example 3: Tree Contracts with Type-Safe Modes");

    // Test tree modes
    println!("  Tree modes:");
    for mode_str in &["100644", "100755", "040000", "invalid"] {
        match UnifiedTreeMode::parse_from_str(mode_str) {
            Some(mode) => println!("    '{}' -> {} ({})", mode_str, mode, mode.description()),
            None => println!("    '{mode_str}' -> Invalid"),
        }
    }

    // Valid tree entry
    let valid_entry = UnifiedTreeEntryContract {
        mode: "100644".to_string(),
        oid: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        filename: "README.md".to_string(),
    };
    println!("  {}", valid_entry.summary());

    // Invalid tree entry
    let invalid_entry = UnifiedTreeEntryContract {
        mode: "invalid".to_string(),
        oid: "invalid-hash".to_string(),
        filename: "file\x00name.txt".to_string(),
    };
    println!("  {}", invalid_entry.summary());
    if !invalid_entry.get_errors().is_empty() {
        println!("    Errors: {:?}", invalid_entry.get_errors());
    }

    // Valid tree (sorted entries)
    let valid_tree = UnifiedTreeContract {
        entries: vec![
            UnifiedTreeEntryContract {
                mode: "100644".to_string(),
                oid: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
                filename: "README.md".to_string(),
            },
            UnifiedTreeEntryContract {
                mode: "100755".to_string(),
                oid: "b2c3d4e5f6789012345678901234567890abcde".to_string(),
                filename: "script.sh".to_string(),
            },
        ],
    };
    println!("  {}", valid_tree.summary());

    // Invalid tree (unsorted entries)
    let invalid_tree = UnifiedTreeContract {
        entries: vec![
            UnifiedTreeEntryContract {
                mode: "100755".to_string(),
                oid: "b2c3d4e5f6789012345678901234567890abcde".to_string(),
                filename: "script.sh".to_string(),
            },
            UnifiedTreeEntryContract {
                mode: "100644".to_string(),
                oid: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
                filename: "README.md".to_string(),
            },
        ],
    };
    println!("  {}", invalid_tree.summary());
    if !invalid_tree.get_errors().is_empty() {
        println!("    Errors: {:?}", invalid_tree.get_errors());
    }

    println!();
    Ok(())
}

fn demo_commit_contracts() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Example 4: Commit Contracts");

    // Valid commit
    let valid_commit = CommitContract {
        tree: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        parents: vec!["b2c3d4e5f6789012345678901234567890abcde".to_string()],
        author: "John Doe <john@example.com>".to_string(),
        committer: "John Doe <john@example.com>".to_string(),
        message: "Initial commit".to_string(),
    };
    println!("  {}", valid_commit.summary());

    // Invalid commit
    let invalid_commit = CommitContract {
        tree: "invalid-hash".to_string(),
        parents: vec!["invalid-parent".to_string()],
        author: "".to_string(),
        committer: "".to_string(),
        message: "".to_string(),
    };
    println!("  {}", invalid_commit.summary());
    if !invalid_commit.get_errors().is_empty() {
        println!("    Errors: {:?}", invalid_commit.get_errors());
    }

    // Merge commit
    let merge_commit = CommitContract {
        tree: "c3d4e5f6789012345678901234567890abcdef".to_string(),
        parents: vec![
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            "b2c3d4e5f6789012345678901234567890abcde".to_string(),
        ],
        author: "Jane Smith <jane@example.com>".to_string(),
        committer: "Jane Smith <jane@example.com>".to_string(),
        message: "Merge branch 'feature' into 'main'".to_string(),
    };
    println!("  {}", merge_commit.summary());

    println!();
    Ok(())
}

fn demo_tag_contracts() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏷️ Example 5: Tag Contracts");

    // Valid tag
    let valid_tag = TagContract {
        object: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        obj_type: "commit".to_string(),
        tag: "v1.0.0".to_string(),
        tagger: "John Doe <john@example.com>".to_string(),
        message: "Release v1.0.0".to_string(),
    };
    println!("  {}", valid_tag.summary());

    // Invalid tag
    let invalid_tag = TagContract {
        object: "invalid-hash".to_string(),
        obj_type: "invalid-type".to_string(),
        tag: "".to_string(),
        tagger: "".to_string(),
        message: "".to_string(),
    };
    println!("  {}", invalid_tag.summary());
    if !invalid_tag.get_errors().is_empty() {
        println!("    Errors: {:?}", invalid_tag.get_errors());
    }

    // Different object types
    let object_types = vec!["commit", "tree", "blob", "tag"];
    for obj_type in object_types {
        let tag = TagContract {
            object: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            obj_type: obj_type.to_string(),
            tag: format!("tag-{obj_type}"),
            tagger: "John Doe <john@example.com>".to_string(),
            message: format!("Tag for {obj_type}"),
        };
        println!("  {}", tag.summary());
    }

    println!();
    Ok(())
}

fn demo_unified_git_objects() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Example 6: Unified Git Object Enum");

    // Create different types of Git objects
    let git_objects = vec![
        GitObject::Blob(UnifiedBlobContract {
            oid: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            size: 5,
            lines: vec![UnifiedBlobLineContract {
                line: "Hello".to_string(),
            }],
        }),
        GitObject::Tree(UnifiedTreeContract {
            entries: vec![UnifiedTreeEntryContract {
                mode: "100644".to_string(),
                oid: "b2c3d4e5f6789012345678901234567890abcde".to_string(),
                filename: "README.md".to_string(),
            }],
        }),
        GitObject::Commit(CommitContract {
            tree: "c3d4e5f6789012345678901234567890abcdef".to_string(),
            parents: vec![],
            author: "John Doe <john@example.com>".to_string(),
            committer: "John Doe <john@example.com>".to_string(),
            message: "Initial commit".to_string(),
        }),
        GitObject::Tag(TagContract {
            object: "d4e5f6789012345678901234567890abcdef0".to_string(),
            obj_type: "commit".to_string(),
            tag: "v1.0.0".to_string(),
            tagger: "John Doe <john@example.com>".to_string(),
            message: "Release v1.0.0".to_string(),
        }),
    ];

    // Validate and summarize each object
    for obj in &git_objects {
        println!("  {}: {}", obj.kind(), obj.summary());
    }

    // Polymorphic validation
    println!("  Polymorphic validation:");
    let valid_count = git_objects.iter().filter(|obj| obj.validate()).count();
    let total_count = git_objects.len();
    println!("    Valid: {valid_count}/{total_count}");

    println!();
    Ok(())
}

fn demo_unified_validator() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Example 7: Unified Validator");

    // Create a validator that validates all object types
    let full_validator = UnifiedValidator::default();

    // Create a validator that only validates blobs and trees
    let partial_validator = UnifiedValidator::new(true, true, false, false);

    // Create test objects
    let objects = vec![
        GitObject::Blob(UnifiedBlobContract {
            oid: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            size: 5,
            lines: vec![UnifiedBlobLineContract {
                line: "Hello".to_string(),
            }],
        }),
        GitObject::Tree(UnifiedTreeContract {
            entries: vec![UnifiedTreeEntryContract {
                mode: "100644".to_string(),
                oid: "b2c3d4e5f6789012345678901234567890abcde".to_string(),
                filename: "README.md".to_string(),
            }],
        }),
        GitObject::Commit(CommitContract {
            tree: "c3d4e5f6789012345678901234567890abcdef".to_string(),
            parents: vec![],
            author: "John Doe <john@example.com>".to_string(),
            committer: "John Doe <john@example.com>".to_string(),
            message: "Initial commit".to_string(),
        }),
    ];

    // Test full validator
    println!("  Full validator (all types):");
    let full_results = full_validator.validate_objects(&objects);
    for (i, (obj, result)) in objects.iter().zip(full_results.iter()).enumerate() {
        println!(
            "    Object {} ({}): {}",
            i + 1,
            obj.kind(),
            if *result { "✅ Valid" } else { "❌ Invalid" }
        );
    }
    println!(
        "    Summary: {}",
        full_validator.summarize_validation(&objects)
    );

    // Test partial validator
    println!("  Partial validator (blobs and trees only):");
    let partial_results = partial_validator.validate_objects(&objects);
    for (i, (obj, result)) in objects.iter().zip(partial_results.iter()).enumerate() {
        println!(
            "    Object {} ({}): {}",
            i + 1,
            obj.kind(),
            if *result { "✅ Valid" } else { "❌ Invalid" }
        );
    }
    println!(
        "    Summary: {}",
        partial_validator.summarize_validation(&objects)
    );

    println!();
    Ok(())
}

fn demo_serialization() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Example 8: Serialization and Deserialization");

    // Create a Git object
    let git_object = GitObject::Blob(UnifiedBlobContract {
        oid: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        size: 5,
        lines: vec![UnifiedBlobLineContract {
            line: "Hello".to_string(),
        }],
    });

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&git_object)?;
    println!("  Serialized JSON:");
    println!("{json}");

    // Deserialize from JSON
    let deserialized: GitObject = serde_json::from_str(&json)?;
    println!("  Deserialized object: {}", deserialized.summary());

    // Test with different object types
    let tree_json = r#"{
        "kind": "tree",
        "entries": [
            {
                "mode": "100644",
                "oid": "a1b2c3d4e5f6789012345678901234567890abcd",
                "filename": "README.md"
            }
        ]
    }"#;

    let tree_object: GitObject = serde_json::from_str(tree_json)?;
    println!("  Deserialized tree: {}", tree_object.summary());

    println!();
    Ok(())
}
