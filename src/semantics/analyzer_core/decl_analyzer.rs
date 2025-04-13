use crate::parser::ast::{Declaration, DeclarationKind, Expression, Literal, Type};
use crate::semantics::analyzer_core::SemanticAnalyzer;
use crate::semantics::symbol_table::{Symbol, SymbolKind};

impl SemanticAnalyzer {
    pub(crate) fn analyze_declaration(&mut self, declaration: &Declaration) {
        match &declaration.node {
            DeclarationKind::Constant(name, typ, value) => {
                // Check for duplicate declaration
                if self.symbol_table.contains(name) {
                    let existing = self.symbol_table.get(name).unwrap();
                    self.duplicate_declaration_error(
                        declaration.span,
                        name,
                        existing.line,
                        existing.column,
                    );
                    return;
                }

                // Track zero literals for division checks
                if let Literal::Int(0) = value {
                    let line = self.source_map.get_line(declaration.span);
                    let column = self.source_map.get_column(declaration.span);
                    self.zero_literals.push((line, column));
                } else if let Literal::Float(f) = value {
                    if *f == 0.0 {
                        let line = self.source_map.get_line(declaration.span);
                        let column = self.source_map.get_column(declaration.span);
                        self.zero_literals.push((line, column));
                    }
                }

                // Validate literal type matches declared type
                let value_type = match value {
                    Literal::Int(_) => Type::Int,
                    Literal::Float(_) => Type::Float,
                    Literal::String(_) => {
                        self.other_error(
                            "String literals are not supported as constants".to_string(),
                        );
                        return;
                    }
                };

                if value_type != *typ {
                    self.type_mismatch_error(declaration.span, typ, &value_type, Some("constant"));
                    return;
                }

                // Add to symbol table
                let line = self.source_map.get_line(declaration.span);
                let column = self.source_map.get_column(declaration.span);

                let symbol = Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Constant,
                    symbol_type: typ.clone(),
                    value: Some(value.clone()),
                    line,
                    column,
                };
                self.symbol_table.add_symbol(symbol);
            }

            DeclarationKind::VariableWithInit(names, typ, expr) => {
                // First, check the expression
                let expr_type = self.analyze_expression(expr);

                if let Some(expr_type) = expr_type {
                    if expr_type != *typ {
                        self.type_mismatch_error(
                            declaration.span,
                            typ,
                            &expr_type,
                            Some("assignment"),
                        );
                    }
                }

                // Add all variables to symbol table
                for name in names {
                    // Check for duplicate declaration
                    if self.symbol_table.contains(name) {
                        let existing = self.symbol_table.get(name).unwrap();
                        self.duplicate_declaration_error(
                            declaration.span,
                            name,
                            existing.line,
                            existing.column,
                        );
                        continue;
                    }

                    // Get line/column from span
                    let line = self.source_map.get_line(declaration.span);
                    let column = self.source_map.get_column(declaration.span);

                    // Add to symbol table
                    let symbol = Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Variable,
                        symbol_type: typ.clone(),
                        value: None,
                        line,
                        column,
                    };
                    self.symbol_table.add_symbol(symbol);
                }
            }

            // Handle other declaration types similarly
            DeclarationKind::Variable(names, typ) => {
                for name in names {
                    if self.symbol_table.contains(name) {
                        let existing = self.symbol_table.get(name).unwrap();
                        self.duplicate_declaration_error(
                            declaration.span,
                            name,
                            existing.line,
                            existing.column,
                        );
                        continue;
                    }

                    let line = self.source_map.get_line(declaration.span);
                    let column = self.source_map.get_column(declaration.span);

                    let symbol = Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Variable,
                        symbol_type: typ.clone(),
                        value: None,
                        line,
                        column,
                    };
                    self.symbol_table.add_symbol(symbol);
                }
            }

            DeclarationKind::Array(names, typ, size) => {
                for name in names {
                    if self.symbol_table.contains(name) {
                        let existing = self.symbol_table.get(name).unwrap();
                        self.duplicate_declaration_error(
                            declaration.span,
                            name,
                            existing.line,
                            existing.column,
                        );
                        continue;
                    }

                    let line = self.source_map.get_line(declaration.span);
                    let column = self.source_map.get_column(declaration.span);

                    let symbol = Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Array(*size),
                        symbol_type: typ.clone(),
                        value: None,
                        line,
                        column,
                    };
                    self.symbol_table.add_symbol(symbol);
                }
            }

            DeclarationKind::ArrayWithInit(names, typ, size, values) => {
                // Check that array size matches number of initializers
                if values.len() > *size {
                    self.other_error(format!(
                        "Too many initializers: expected {}, found {}",
                        size,
                        values.len()
                    ));
                }

                // Check each value's type
                for value in values {
                    let value_type = self.analyze_expression(value);
                    if let Some(value_type) = value_type {
                        if value_type != *typ {
                            self.type_mismatch_error(
                                value.span,
                                typ,
                                &value_type,
                                Some("array initializer"),
                            );
                        }
                    }
                }

                // Add all arrays to symbol table
                for name in names {
                    if self.symbol_table.contains(name) {
                        let existing = self.symbol_table.get(name).unwrap();
                        self.duplicate_declaration_error(
                            declaration.span,
                            name,
                            existing.line,
                            existing.column,
                        );
                        continue;
                    }

                    let line = self.source_map.get_line(declaration.span);
                    let column = self.source_map.get_column(declaration.span);

                    let symbol = Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Array(*size),
                        symbol_type: typ.clone(),
                        value: None,
                        line,
                        column,
                    };
                    self.symbol_table.add_symbol(symbol);
                }
            }
        }
    }

    // Helper to extract literal value from expression (if possible)
    pub(crate) fn extract_literal(&self, expr: &Expression) -> Option<Literal> {
        match &expr.node {
            crate::parser::ast::ExpressionKind::Literal(lit) => Some(lit.clone()),
            _ => None,
        }
    }
}
