//! Core modules for the hooksmith CLI

//! Core modules for CLI functionality
//!
//! This module contains core functionality modules for the Hooksmith CLI.

/// Structured code generation
pub mod generator;
/// Git file states, actions, and hooks model
pub mod git_model;
/// Lefthook configuration generation
pub mod lefthook;
/// WASM component management
pub mod wasm;
/// Hook building and compilation
pub mod hook_builder;
/// Hierarchical contract validation
pub mod hierarchical_validation;
/// Contract state machine for validation lifecycle
pub mod contract_state_machine;
