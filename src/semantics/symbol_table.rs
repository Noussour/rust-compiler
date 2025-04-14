use crate::parser::ast::{LiteralKind, Type};
use std::{collections::HashMap, default};

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
    pub value: Option<LiteralKind>,
    pub is_constant: bool,
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
}

impl default::Default for Symbol {
    fn default() -> Self {
        Symbol {
            name: String::new(),
            kind: SymbolKind::Variable,
            symbol_type: Type::default(),
            value: None,
            line: 0,
            column: 0,
            is_constant: false,
        }
    }
}
