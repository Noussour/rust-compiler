mod declaration_analyzer;
mod statement_analyzer;
mod expression_analyzer;


use crate::parser::ast::{Program, Type};
use crate::semantics::error::SemanticError;
use crate::semantics::source_map::SourceMap;
use crate::semantics::symbol_table::SymbolTable;
use std::collections::HashSet;
use std::ops::Range;

pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    errors: Vec<SemanticError>,
    reported_errors: HashSet<String>,
    source_map: SourceMap,
    zero_literals: Vec<(usize, usize)>, // For tracking division by zero
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        // Create with an empty source map
        SemanticAnalyzer {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            reported_errors: HashSet::new(),
            source_map: SourceMap::new(String::new()),
            zero_literals: Vec::new(),
        }
    }

    pub fn new_with_source_code(source_code: String) -> Self {
        SemanticAnalyzer {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            reported_errors: HashSet::new(),
            source_map: SourceMap::new(source_code),
            zero_literals: Vec::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) {
        // First pass: analyze declarations
        for decl in &program.declarations {
            self.analyze_declaration(decl);
        }

        // Second pass: analyze statements
        for stmt in &program.statements {
            self.analyze_statement(stmt);
        }
    }

    // Error helper methods
    fn type_mismatch_error(
        &mut self,
        span: &Range<usize>,
        expected: &Type,
        found: &Type,
        context: Option<&str>,
    ) {
        self.add_error(SemanticError::TypeMismatch {
            expected: format!("{}", expected),
            found: format!("{}", found),
            line: self.source_map.get_line(span),
            column: self.source_map.get_column(span),
            context: context.map(|s| s.to_string()),
        });
    }

    fn undeclared_identifier_error(&mut self, span: &Range<usize>, name: &str) {
        self.add_error(SemanticError::UndeclaredIdentifier {
            name: name.to_string(),
            line: self.source_map.get_line(span),
            column: self.source_map.get_column(span),
        });
    }

    fn constant_modification_error(&mut self, span: &Range<usize>, name: &str) {
        self.add_error(SemanticError::ConstantModification {
            name: name.to_string(),
            line: self.source_map.get_line(span),
            column: self.source_map.get_column(span),
        });
    }

    fn array_index_out_of_bounds_error(
        &mut self,
        span: &Range<usize>,
        name: &str,
        index: usize,
        size: usize,
    ) {
        self.add_error(SemanticError::ArrayIndexOutOfBounds {
            name: name.to_string(),
            index,
            size,
            line: self.source_map.get_line(span),
            column: self.source_map.get_column(span),
        });
    }

    fn division_by_zero_error(&mut self, span: &Range<usize>) {
        self.add_error(SemanticError::DivisionByZero {
            line: self.source_map.get_line(span),
            column: self.source_map.get_column(span),
        });
    }

    fn duplicate_declaration_error(
        &mut self,
        span: &Range<usize>,
        name: &str,
        original_line: usize,
        original_column: usize,
    ) {
        self.add_error(SemanticError::DuplicateDeclaration {
            name: name.to_string(),
            line: self.source_map.get_line(span),
            column: self.source_map.get_column(span),
            original_line,
            original_column,
        });
    }


    pub fn add_error(&mut self, error: SemanticError) {
        // Only add the error if it hasn't been reported yet
        let error_key = format!("{:?}", error);
        if !self.reported_errors.contains(&error_key) {
            self.reported_errors.insert(error_key);
            self.errors.push(error);
        }
    }


    pub fn get_errors(&self) -> &[SemanticError] {
        &self.errors
    }

    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }
}
