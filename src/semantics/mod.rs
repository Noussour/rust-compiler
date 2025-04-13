// Public modules
pub mod analyzer_core;
pub mod error;
pub mod symbol_table;
pub mod source_map;

// Re-export the main interface
pub use analyzer_core::SemanticAnalyzer;
