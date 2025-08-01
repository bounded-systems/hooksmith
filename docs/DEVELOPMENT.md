# Development Guide

This guide provides information for developers working on Hooksmith.

## Prerequisites

- Rust (latest stable)
- Git
- Bash (for build scripts)

## Development Workflow

### 1. Build the Project


### 2. Run Tests


### 3. Generate Documentation


### 4. Run CLI Commands


## Project Structure

- `src/main.rs`: Main CLI application
- `src/lib.rs`: Library exports
- `components/cli-core/`: Core CLI functionality
- `tests/`: Test files
- `scripts/`: Build and documentation scripts

## Code Style

This project uses Trunk for code quality:


## Documentation

- API docs: `cargo doc --no-deps --open`
- CLI help: `cargo run -- --help`
- Project docs: `docs/` directory
