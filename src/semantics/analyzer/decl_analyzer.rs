use crate::parser::ast::{Declaration, Literal, Type};
use crate::semantics::analyzer::SemanticAnalyzer;
use crate::semantics::error::SemanticError;
use crate::semantics::symbol_table::{Symbol, SymbolKind};

impl SemanticAnalyzer {
    /// Analyzes a declaration
    pub(crate) fn analyze_declaration(&mut self, declaration: &Declaration) {
        match declaration {
            Declaration::Variable(names, typ) => {
                for name in names {
                    let (line, column) = self.get_position(name);

                    // Check for duplicate declaration
                    if self.symbol_table.contains(name) {
                        let existing = self.symbol_table.get(name).unwrap();
                        self.errors.push(SemanticError::DuplicateDeclaration {
                            name: name.clone(),
                            line,
                            column,
                            original_line: existing.line,
                            original_column: existing.column,
                        });
                        continue;
                    }

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
            Declaration::Array(names, typ, size) => {
                for name in names {
                    let (line, column) = self.get_position(name);

                    // Check for duplicate declaration
                    if self.symbol_table.contains(name) {
                        let existing = self.symbol_table.get(name).unwrap();
                        self.errors.push(SemanticError::DuplicateDeclaration {
                            name: name.clone(),
                            line,
                            column,
                            original_line: existing.line,
                            original_column: existing.column,
                        });
                        continue;
                    }

                    // Add to symbol table
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
            Declaration::Constant(name, typ, value) => {
                let (line, column) = self.get_position(name);

                // Check for duplicate declaration
                if self.symbol_table.contains(name) {
                    let existing = self.symbol_table.get(name).unwrap();
                    self.errors.push(SemanticError::DuplicateDeclaration {
                        name: name.clone(),
                        line,
                        column,
                        original_line: existing.line,
                        original_column: existing.column,
                    });
                    return;
                }

                // Check value type matches declaration type
                let value_type = match value {
                    Literal::Int(_) => Type::Int,
                    Literal::Float(_) => Type::Float,
                    Literal::String(_) => {
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: format!("{}", typ),
                            found: "String".to_string(),
                            line,
                            column,
                        });
                        return;
                    }
                };

                // Check for division by zero in constants
                if let Literal::Int(0) = value {
                    self.zero_literals.push((line, column));
                } else if let Literal::Float(f) = value {
                    if *f == 0.0 {
                        self.zero_literals.push((line, column));
                    }
                }

                if value_type != *typ {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: format!("{}", typ),
                        found: format!("{}", value_type),
                        line,
                        column,
                    });
                    return;
                }

                // Add to symbol table
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
            Declaration::VariableWithInit(names, typ, expr) => {
                // First, check the expression
                let expr_type = self.analyze_expression(expr);
                
                // Try to extract the literal value
                let literal_value = self.extract_literal(expr);
                
                if let Some(expr_type) = expr_type {
                    if expr_type != *typ {
                        let (line, column) = self.get_position(&names[0]);
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: format!("{}", typ),
                            found: format!("{}", expr_type),
                            line,
                            column,
                        });
                    }
                }

                // Now add the variables to the symbol table
                for name in names {
                    let (line, column) = self.get_position(name);

                    // Check for duplicate declaration
                    if self.symbol_table.contains(name) {
                        let existing = self.symbol_table.get(name).unwrap();
                        self.errors.push(SemanticError::DuplicateDeclaration {
                            name: name.clone(),
                            line,
                            column,
                            original_line: existing.line,
                            original_column: existing.column,
                        });
                        continue;
                    }

                    // Add to symbol table with value if it's a literal
                    let symbol = Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Variable,
                        symbol_type: typ.clone(),
                        value: literal_value.clone(), // Store the value if it's a literal
                        line,
                        column,
                    };
                    self.symbol_table.add_symbol(symbol);
                }
            }
            Declaration::ArrayWithInit(names, typ, size, values) => {
                // Check if number of initializer values matches array size
                if values.len() > *size {
                    self.errors.push(SemanticError::Other(format!(
                        "Too many initializer values for array. Expected {}, got {}",
                        size,
                        values.len()
                    )));
                }

                // Try to extract all literal values
                let mut literal_values = Vec::new();
                let mut all_literals = true;
                
                for value in values {
                    if let Some(lit) = self.extract_literal(value) {
                        literal_values.push(lit);
                    } else {
                        all_literals = false;
                        break;
                    }
                    
                    let value_type = self.analyze_expression(value);
                    if let Some(value_type) = value_type {
                        if value_type != *typ {
                            let (line, column) = self.get_position(&names[0]);
                            self.errors.push(SemanticError::TypeMismatch {
                                expected: format!("{}", typ),
                                found: format!("{}", value_type),
                                line,
                                column,
                            });
                        }
                    }
                }

                // Now add the arrays to the symbol table
                for name in names {
                    let (line, column) = self.get_position(name);

                    // Check for duplicate declaration
                    if self.symbol_table.contains(name) {
                        let existing = self.symbol_table.get(name).unwrap();
                        self.errors.push(SemanticError::DuplicateDeclaration {
                            name: name.clone(),
                            line,
                            column,
                            original_line: existing.line,
                            original_column: existing.column,
                        });
                        continue;
                    }

                    // Create an array initializer literal if we have all literals
                    let array_value = if all_literals && !literal_values.is_empty() {
                        Some(Literal::String(format!("{:?}", literal_values))) // Use String type as a temp container
                    } else {
                        None
                    };

                    // Add to symbol table
                    let symbol = Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Array(*size),
                        symbol_type: typ.clone(),
                        value: array_value, // Store array literal values if all are literals
                        line,
                        column,
                    };
                    self.symbol_table.add_symbol(symbol);
                }
            }
        }
    }
}
