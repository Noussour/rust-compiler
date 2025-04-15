use std::ops::Range;

use crate::parser::ast::{
    Declaration, DeclarationKind, Expression, Literal, LiteralKind, Type,
};
use crate::semantics::analyzer_core::SemanticAnalyzer;
use crate::semantics::symbol_table::{Symbol, SymbolKind};

impl SemanticAnalyzer {
    pub fn analyze_declaration(&mut self, declaration: &Declaration) {
        // Implementation of declaration analysis
        match &declaration.node {
            DeclarationKind::Variable(items, typ) => {
                for item in items {
                    self.handle_variable_declaration(item, typ, declaration.span.clone());
                }
            }
            DeclarationKind::Array(items, typ, size) => {
                for item in items {
                    self.handle_array_declaration(item, typ, *size, declaration.span.clone());
                }
            }
            DeclarationKind::VariableWithInit(items, typ, expression) => {
                for item in items {
                    self.handle_variable_declaration_with_init(
                        item,
                        typ,
                        expression,
                        declaration.span.clone(),
                    );
                }
            }
            DeclarationKind::ArrayWithInit(items, typ, size, expressions) => {
                for item in items {
                    self.handle_array_declaration_with_init(
                        item,
                        typ,
                        *size,
                        expressions,
                        declaration.span.clone(),
                    );
                }
            }
            DeclarationKind::Constant(value, typ, literal) => {
                self.handle_constant_declaration(value, typ, literal, declaration.span.clone());
            }
        }
    }

    fn handle_constant_declaration(
        &mut self,
        value: &str,
        typ: &Type,
        literal: &Literal,
        span: Range<usize>,
    ) {
        // Check for duplicate declaration
        if self.symbol_table.contains(value) {
            let existing = self.symbol_table.get(value).unwrap();
            self.duplicate_declaration_error(span, value, existing.line, existing.column);
            return;
        }

        // Track zero literals for division checks
        match literal.node {
            LiteralKind::Int(n) if n == 0 => {
                let line = self.source_map.get_line(span.clone());
                let column = self.source_map.get_column(span.clone());
                self.zero_literals.push((line, column));
            }
            LiteralKind::Float(f) if f == 0.0 => {
                let line = self.source_map.get_line(span.clone());
                let column = self.source_map.get_column(span.clone());
                self.zero_literals.push((line, column));
            }
            _ => {}
        }

        // Validate literal type matches declared type
        let value_type = match literal.node {
            LiteralKind::Int(_) => Type::Int,
            LiteralKind::Float(_) => Type::Float,
            LiteralKind::String(_) => {
                return;
            }
        };

        if value_type != *typ {
            self.type_mismatch_error(span, typ, &value_type, Some("constant"));
            return;
        }

        // Add to symbol table
        let line = self.source_map.get_line(span.clone());
        let column = self.source_map.get_column(span.clone());

        let symbol = Symbol {
            name: value.to_string(),
            kind: SymbolKind::Constant,
            symbol_type: typ.clone(),
            value: Some(literal.node.clone()),
            line,
            column,
            is_constant: true, // Set is_constant to true for constants
        };

        self.symbol_table.add_symbol(symbol);
    }

    fn handle_variable_declaration(&mut self, name: &str, typ: &Type, span: Range<usize>) {
        // Check for duplicate declaration
        if self.symbol_table.contains(name) {
            let existing = self.symbol_table.get(name).unwrap();
            self.duplicate_declaration_error(span, name, existing.line, existing.column);
            return;
        }

        // Add to symbol table
        let line = self.source_map.get_line(span.clone());
        let column = self.source_map.get_column(span.clone());

        let symbol = Symbol {
            name: name.to_string(),
            kind: SymbolKind::Variable,
            symbol_type: typ.clone(),
            value: None,
            line,
            column,
            is_constant: false, // Set is_constant to false for variables
        };
        self.symbol_table.add_symbol(symbol);
    }

    fn handle_array_declaration(&mut self, name: &str, typ: &Type, size: usize, span: Range<usize>) {
        // Check for duplicate declaration
        if self.symbol_table.contains(name) {
            let existing = self.symbol_table.get(name).unwrap();
            self.duplicate_declaration_error(span, name, existing.line, existing.column);
            return;
        }

        // Add to symbol table
        let line = self.source_map.get_line(span.clone());
        let column = self.source_map.get_column(span);

        let symbol = Symbol {
            name: name.to_string(),
            kind: SymbolKind::Array(size),
            symbol_type: typ.clone(),
            value: None,
            line,
            column,

            is_constant: false,
        };

        self.symbol_table.add_symbol(symbol);
    }

    fn handle_variable_declaration_with_init(
        &mut self,
        name: &str,
        typ: &Type,
        expr: &Expression,
        span: Range<usize>,
    ) {
        // First, check the expression
        let expr_type = self.analyze_expression(expr);

        if let Some(expr_type) = expr_type {
            if expr_type != *typ {
                self.type_mismatch_error(span.clone(), typ, &expr_type, Some("assignment"));
            }
        }

        // Add to symbol table
        self.handle_variable_declaration(name, typ, span);
    }

    fn handle_array_declaration_with_init(
        &mut self,
        name: &str,
        typ: &Type,
        size: usize,
        exprs: &[Expression],
        span: Range<usize>,
    ) {
        // Check that array size matches number of initializers
        if exprs.len() == size {
            println!("to check later")
        }

        // Check each value's type
        for expr in exprs {
            let value_type = self.analyze_expression(expr);
            if let Some(value_type) = value_type {
                if value_type != *typ {
                    self.type_mismatch_error(span.clone(), typ, &value_type, Some("array initializer"));
                }
            }
        }

        // Add to symbol table
        self.handle_array_declaration(name, typ, size, span);
    }
}
