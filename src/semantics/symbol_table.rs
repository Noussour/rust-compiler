use crate::parser::ast::{Literal, Type};
use std::collections::HashMap;

/// Symbol kind (variable, constant, or array)
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable,
    Constant,
    Array(usize), // Contains array size
}

/// Symbol information stored in the symbol table
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub symbol_type: Type,
    pub value: Option<Literal>,
    pub line: usize,
    pub column: usize,
}

/// SymbolTable tracks all declared identifiers and their information
#[derive(Debug, Default, Clone)]
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
}

impl SymbolTable {
    /// Creates a new empty symbol table
    pub fn new() -> Self {
        SymbolTable {
            symbols: HashMap::new(),
        }
    }

    /// Adds a variable or constant to the symbol table
    /// Returns true if successful, false if the symbol already exists
    pub fn add_symbol(&mut self, symbol: Symbol) -> bool {
        if self.symbols.contains_key(&symbol.name) {
            return false;
        }
        self.symbols.insert(symbol.name.clone(), symbol);
        true
    }

    /// Checks if a symbol exists in the table
    pub fn contains(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    /// Gets a symbol by name
    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    /// Gets all symbols
    pub fn get_all(&self) -> Vec<&Symbol> {
        self.symbols.values().collect()
    }

    /// Updates the value of an existing symbol
    pub fn update_value(&mut self, name: &str, value: Literal) -> bool {
        if let Some(symbol) = self.symbols.get_mut(name) {
            // Check if symbol is a constant
            if let SymbolKind::Constant = symbol.kind {
                return false; // Can't modify a constant
            }
            symbol.value = Some(value);
            true
        } else {
            false
        }
    }
}
