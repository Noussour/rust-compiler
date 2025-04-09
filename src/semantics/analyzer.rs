mod decl_analyzer;
mod expr_analyzer;
mod stmt_analyzer;
mod type_utils;

use crate::parser::ast::Program;
use crate::semantics::error::SemanticError;
use crate::semantics::symbol_table::SymbolTable;
use std::collections::{HashMap, HashSet};

// Enhanced position tracking for expressions
#[derive(Debug, Clone, PartialEq)]
struct ExpressionPosition {
    pub line: usize,
    pub column: usize,
}

/// The semantic analyzer for MiniSoft
#[derive(Default)]
pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    errors: Vec<SemanticError>,
    // Store position information from the lexer
    positions: HashMap<String, (usize, usize)>,
    // Track current position for expressions
    current_expr_pos: Option<ExpressionPosition>,
    // Track expression positions
    expression_positions: HashMap<String, (usize, usize)>,
    // Keep track of division by zero literals for better error reporting
    zero_literals: Vec<(usize, usize)>,
    // Keep track of reported error keys to avoid duplicates
    reported_errors: HashSet<String>,
}

impl SemanticAnalyzer {
    /// Creates a new semantic analyzer with position information
    pub fn new_with_positions(positions: HashMap<String, (usize, usize)>) -> Self {
        SemanticAnalyzer {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            positions,
            current_expr_pos: None,
            expression_positions: HashMap::new(),
            zero_literals: Vec::new(),
            reported_errors: HashSet::new(),
        }
    }

    /// Gets position information for an identifier
    fn get_position(&self, name: &str) -> (usize, usize) {
        // First try identifier's known position from the map
        if let Some(pos) = self.positions.get(name) {
            return *pos;
        }

        // Try the expression positions map (for computed expressions)
        if let Some(pos) = self.expression_positions.get(name) {
            return *pos;
        }

        // Fall back to current expression position
        if let Some(pos) = &self.current_expr_pos {
            return (pos.line, pos.column);
        }

        // Default position
        (1, 1)
    }

    /// Set the current expression position context
    fn set_current_expr_pos(&mut self, line: usize, column: usize) {
        self.current_expr_pos = Some(ExpressionPosition { line, column });
    }

    /// Track a position for a specific expression key
    fn track_expression_pos(&mut self, key: String, line: usize, column: usize) {
        self.expression_positions.insert(key, (line, column));
    }

    /// Clear the current expression position context
    fn clear_current_expr_pos(&mut self) {
        self.current_expr_pos = None;
    }

    /// Analyzes a program for semantic errors
    pub fn analyze(&mut self, program: &Program) {
        // Set position for program name
        if let Some(pos) = self.positions.get(&program.name) {
            self.set_current_expr_pos(pos.0, pos.1);
        }

        // Process all declarations
        for declaration in &program.declarations {
            self.analyze_declaration(declaration);
        }

        // Clear any accumulated positions to start fresh for statements
        self.clear_current_expr_pos();

        // Process all statements
        for statement in &program.statements {
            self.analyze_statement(statement);
        }

        self.clear_current_expr_pos();
    }

    /// Gets all semantic errors found during analysis
    pub fn get_errors(&self) -> &[SemanticError] {
        &self.errors
    }

    /// Gets the completed symbol table
    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    /// Adds an error if it hasn't been reported yet
    fn add_error(&mut self, error: SemanticError) {
        // Create a unique key for this error to avoid duplicates
        let error_key = match &error {
            SemanticError::TypeMismatch {
                expected,
                found,
                line,
                column,
                context,
            } => {
                format!("type_mismatch:{}:{}:{}:{}:{}", expected, found, line, column, context.as_ref().unwrap_or(&"".to_string()))
            }
            SemanticError::UndeclaredIdentifier { name, line, column } => {
                format!("undeclared:{}:{}:{}", name, line, column)
            }
            // Add other error types as needed
            _ => format!("{:?}", error),
        };

        // Only add the error if we haven't reported it yet
        if !self.reported_errors.contains(&error_key) {
            self.errors.push(error);
            self.reported_errors.insert(error_key);
        }
    }
}
