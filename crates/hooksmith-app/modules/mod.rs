//! Core modules for the Hooksmith system

/// Agreement management system
pub mod agreement_manager;
/// Contract state machine
pub mod contract_state_machine;
/// Contract validation system
pub mod contract_validation;
/// Contract-to-crate boundary mapping
pub mod crate_contract_mapper;
/// Functional contract validation pipeline
pub mod functional_contract_pipeline;
/// Code generator
pub mod generator;
/// Git bindings and operations
pub mod git_bindings;
/// Git model and types
pub mod git_model;
/// Git native operations
pub mod git_native;
/// Hierarchical validation system
pub mod hierarchical_validation;
/// Hook builder system
pub mod hook_builder;
/// Lefthook integration
pub mod lefthook;
/// Static hook management
pub mod static_hook;
/// Tree-aware fix plan cache
pub mod tree_fix_cache;
/// Tree split planning and analysis
pub mod tree_split_planner;
/// WebAssembly management
pub mod wasm;
