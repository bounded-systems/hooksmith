//! Core modules for the hooksmith CLI

//! Core modules for CLI functionality
//!
//! This module contains core functionality modules for the Hooksmith CLI.

/// Contract state machine for validation lifecycle
pub mod contract_state_machine;
/// Structured code generation
pub mod generator;
/// Git file states, actions, and hooks model
pub mod git_model;
/// Hierarchical contract validation
pub mod hierarchical_validation;
/// Hook building and compilation
pub mod hook_builder;
/// Lefthook configuration generation
pub mod lefthook;
/// WASM component management
pub mod wasm;
