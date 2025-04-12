pub mod analyzer_core;
pub mod error;
pub mod symbol_table;

// Re-export commonly used items for easier access
pub use analyzer_core::SemanticAnalyzer;
pub use symbol_table::SymbolKind;
