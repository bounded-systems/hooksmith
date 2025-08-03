use hooksmith::modules::lefthook;
use serde_json::json;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_schema_validation() {
    // Test with a valid configuration
    let valid_config = json!({
        "pre-commit": {
            "commands": {
                "test": {
                    "run": "cargo test"
                }
            }
        }
    });

    // This should pass validation
    let result = lefthook::validate_against_schema(&valid_config).await;
    assert!(
        result.is_ok(),
        "Valid configuration should pass schema validation"
    );

    // Test with an invalid configuration
    let invalid_config = json!({
        "pre-commit": {
            "invalid_field": "this should not be allowed"
        }
    });

    // This should fail validation
    let result = lefthook::validate_against_schema(&invalid_config).await;
    assert!(
        result.is_err(),
        "Invalid configuration should fail schema validation"
    );
}

#[tokio::test]
async fn test_existing_config_validation() {
    // Create a temporary valid lefthook.yml file
    let temp_file = NamedTempFile::new().unwrap();
    let yaml_content = r#"
pre-commit:
  commands:
    test:
      run: "cargo test"
      glob: "*.rs"
"#;
    std::fs::write(temp_file.path(), yaml_content).unwrap();

    // This should pass validation
    let result = lefthook::validate_existing_config(temp_file.path()).await;
    assert!(
        result.is_ok(),
        "Valid YAML configuration should pass validation"
    );
}
