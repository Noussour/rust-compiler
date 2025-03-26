pub mod analyzer;
pub mod error;
pub mod symbol_table;

// Re-export commonly used items for easier access
pub use analyzer::SemanticAnalyzer;
pub use error::SemanticError;
pub use symbol_table::{Symbol, SymbolKind, SymbolTable};
