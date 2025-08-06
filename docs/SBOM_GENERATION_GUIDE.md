# SBOM Generation Guide

This guide explains how to generate and manage Software Bill of Materials (SBOM) for the Hooksmith project using the integrated xtask system.

## Overview

SBOM (Software Bill of Materials) is a formal, machine-readable inventory of software components and dependencies. Hooksmith provides integrated SBOM generation capabilities through the xtask system, supporting multiple formats:

- **CycloneDX**: Industry-standard format for security and compliance
- **SPDX**: Software Package Data Exchange format
- **Cargo Metadata**: Raw Rust dependency information

## Quick Start

### Generate All SBOM Formats

```bash
# Generate all SBOM formats at once
cargo xtask sbom generate

# This creates:
# - .devcontracts/sbom/sbom.cyclonedx.json
# - .devcontracts/sbom/sbom.spdx.json  
# - .devcontracts/sbom/sbom.cargo-metadata.json
```

### Generate Specific Format

```bash
# Generate only CycloneDX format
cargo xtask sbom generate --format cyclonedx

# Generate only SPDX format
cargo xtask sbom generate --format spdx

# Generate only Cargo metadata
cargo xtask sbom generate --format cargo-metadata
```

## Prerequisites

### Install Required Tools

#### CycloneDX Generator
```bash
# Install cyclonedx-bom for CycloneDX format
cargo install cyclonedx-bom
```

#### SPDX Generator
```bash
# Install protobom for SPDX format (requires Go)
go install github.com/github/protobom/cmd/protobom@latest
```

## Available Commands

### Generate SBOMs
```bash
# Generate all formats
cargo xtask sbom generate

# Generate specific format
cargo xtask sbom generate --format <format>
```

### Validate SBOMs
```bash
# Validate existing SBOM files
cargo xtask sbom validate
```

### Generate Report
```bash
# Generate SBOM status report
cargo xtask sbom report
```

## SBOM Formats

### CycloneDX Format
- **File**: `sbom.cyclonedx.json`
- **Standard**: [CycloneDX Specification](https://cyclonedx.org/specification/overview/)
- **Use Cases**: Security scanning, vulnerability assessment, compliance
- **Features**: 
  - Component inventory with versions
  - License information
  - Vulnerability data (when available)
  - Dependency relationships

### SPDX Format
- **File**: `sbom.spdx.json`
- **Standard**: [SPDX Specification](https://spdx.dev/specifications/)
- **Use Cases**: License compliance, legal review, open source governance
- **Features**:
  - Detailed license information
  - Copyright notices
  - File-level granularity
  - Package relationships

### Cargo Metadata Format
- **File**: `sbom.cargo-metadata.json`
- **Use Cases**: Rust-specific analysis, development tooling
- **Features**:
  - Raw Cargo workspace information
  - Dependency resolution details
  - Build configuration
  - Workspace structure

## Directory Structure

```
.devcontracts/
└── sbom/
    ├── sbom.cyclonedx.json      # CycloneDX format
    ├── sbom.spdx.json           # SPDX format
    └── sbom.cargo-metadata.json # Cargo metadata
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Generate SBOM
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  sbom:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Install SBOM tools
        run: |
          cargo install cyclonedx-bom
          go install github.com/github/protobom/cmd/protobom@latest
          
      - name: Generate SBOM
        run: cargo xtask sbom generate
        
      - name: Upload SBOM artifacts
        uses: actions/upload-artifact@v4
        with:
          name: sbom-files
          path: .devcontracts/sbom/
```

### Validation in CI

```yaml
      - name: Validate SBOM
        run: cargo xtask sbom validate
        
      - name: Generate SBOM report
        run: cargo xtask sbom report
```

## Security and Compliance

### Vulnerability Scanning

Use the generated CycloneDX SBOM with security tools:

```bash
# Using OWASP Dependency Check
dependency-check --scan .devcontracts/sbom/sbom.cyclonedx.json

# Using Trivy
trivy fs --format cyclonedx .devcontracts/sbom/sbom.cyclonedx.json
```

### License Compliance

Use the SPDX SBOM for license analysis:

```bash
# Using SPDX tools
spdx-tools-verify .devcontracts/sbom/sbom.spdx.json
```

## Advanced Usage

### Custom Output Directory

```bash
# The SBOM generator uses .devcontracts/sbom/ by default
# You can modify this in the sbom.rs module if needed
```

### Integration with Contract Validation

The SBOM system integrates with Hooksmith's contract validation:

```bash
# Generate SBOM as part of validation workflow
cargo xtask sbom generate && cargo xtask validate --all
```

### Automated SBOM Updates

```bash
# Add to pre-commit hooks
cargo xtask sbom generate --format cyclonedx
git add .devcontracts/sbom/
```

## Troubleshooting

### Common Issues

#### cyclonedx-bom not found
```bash
# Install the tool
cargo install cyclonedx-bom

# Verify installation
which cyclonedx-bom
```

#### protobom not found
```bash
# Install Go first, then protobom
go install github.com/github/protobom/cmd/protobom@latest

# Verify installation
which protobom
```

#### Permission Issues
```bash
# Ensure write permissions to .devcontracts/sbom/
mkdir -p .devcontracts/sbom/
chmod 755 .devcontracts/sbom/
```

### Validation Errors

```bash
# Check SBOM file validity
cargo xtask sbom validate

# Generate detailed report
cargo xtask sbom report
```

## Best Practices

### 1. Regular Updates
- Generate SBOMs regularly (weekly/monthly)
- Include in CI/CD pipelines
- Version control SBOM files

### 2. Validation
- Always validate generated SBOMs
- Use multiple formats for different use cases
- Integrate with security scanning tools

### 3. Documentation
- Document SBOM generation process
- Include SBOM files in project documentation
- Maintain SBOM generation scripts

### 4. Security
- Review SBOMs for sensitive information
- Use SBOMs for vulnerability assessment
- Integrate with security tools

## Integration with Hooksmith Architecture

The SBOM system aligns with Hooksmith's dual-agent architecture:

1. **Contract Layer**: SBOM generation is part of the contract validation system
2. **Validation Pipeline**: SBOMs are generated as part of the validation workflow
3. **Event Routing**: SBOM generation events can be routed to appropriate handlers
4. **State Machine**: SBOM generation follows the same state machine patterns

## Future Enhancements

- **Automated Vulnerability Scanning**: Integrate SBOM generation with vulnerability assessment
- **License Compliance**: Automated license compliance checking
- **Dependency Tracking**: Track dependency changes over time
- **Integration APIs**: REST APIs for SBOM generation and validation

## References

- [CycloneDX Specification](https://cyclonedx.org/specification/overview/)
- [SPDX Specification](https://spdx.dev/specifications/)
- [Cargo Metadata Format](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)
- [OWASP Dependency Check](https://owasp.org/www-project-dependency-check/)
- [Trivy Vulnerability Scanner](https://trivy.dev/) 
