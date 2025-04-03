use crate::parser::ast::{Expression, Literal, Operator, Type, UnaryOperator};
use crate::semantics::analyzer::SemanticAnalyzer;
use crate::semantics::error::SemanticError;

impl SemanticAnalyzer {
    /// Analyzes an expression to determine its type
    /// Returns the type of the expression, or None if there is an error
    pub(crate) fn analyze_expression(&mut self, expr: &Expression) -> Option<Type> {
        match expr {
            Expression::Identifier(name) => {
                // Check if the identifier exists
                if !self.symbol_table.contains(name) {
                    let (line, column) = self.get_position(name);
                    self.add_error(SemanticError::UndeclaredIdentifier {
                        name: name.clone(),
                        line,
                        column,
                    });
                    return None;
                }

                // Return the identifier's type
                let symbol = self.symbol_table.get(name).unwrap();
                Some(symbol.symbol_type.clone())
            }
            Expression::ArrayAccess(name, index_expr) => {
                // Check if the array exists
                if !self.symbol_table.contains(name) {
                    let (line, column) = self.get_position(name);
                    self.add_error(SemanticError::UndeclaredIdentifier {
                        name: name.clone(),
                        line,
                        column,
                    });
                    return None;
                }

                // Check if it's actually an array
                let symbol = self.symbol_table.get(name).unwrap();
                match &symbol.kind {
                    crate::semantics::symbol_table::SymbolKind::Array(size) => {
                        // Save the symbol type before releasing the borrow
                        let symbol_type = symbol.symbol_type.clone();
                        let array_size = *size;

                        // Check if the index is a constant and within bounds
                        if let Expression::Literal(Literal::Int(idx)) = &**index_expr {
                            if *idx < 0 || *idx as usize >= array_size {
                                let (line, column) = self.get_position(name);
                                self.add_error(SemanticError::ArrayIndexOutOfBounds {
                                    name: name.clone(),
                                    index: *idx as usize,
                                    size: array_size,
                                    line,
                                    column,
                                });
                                return None;
                            }
                        }

                        // Check index expression type
                        let idx_type = self.analyze_expression(index_expr);
                        if let Some(idx_type) = idx_type {
                            if idx_type != Type::Int {
                                let (line, column) = self.get_position(name);
                                self.add_error(SemanticError::TypeMismatch {
                                    expected: "Int".to_string(),
                                    found: format!("{}", idx_type),
                                    line,
                                    column,
                                });
                                return None;
                            }
                        } else {
                            return None;
                        }

                        // Return the array element type
                        Some(symbol_type)
                    }
                    _ => {
                        self.add_error(SemanticError::Other(format!(
                            "Cannot index non-array variable '{}'",
                            name
                        )));
                        None
                    }
                }
            }
            Expression::Literal(lit) => {
                // Check for division by zero in constant literals
                match lit {
                    Literal::Int(_) => Some(Type::Int),
                    Literal::Float(_) => Some(Type::Float),
                    Literal::String(_) => None, // No string type in MiniSoft
                }
            }
            Expression::BinaryOp(left, op, right) => {
                // Save the current position before recursion
                let saved_pos = self.current_expr_pos.clone();

                // Check the types of left and right operands
                let left_type = self.analyze_expression(left);
                let right_type = self.analyze_expression(right);

                // Restore the position after recursion
                self.current_expr_pos = saved_pos;

                if left_type.is_none() || right_type.is_none() {
                    return None;
                }

                let left_type = left_type.unwrap();
                let right_type = right_type.unwrap();

                // Get the current position for error reporting
                let (line, column) = if let Some(pos) = &self.current_expr_pos {
                    (pos.line, pos.column)
                } else {
                    (1, 1) // Default
                };

                match op {
                    // Arithmetic operators
                    Operator::Add | Operator::Subtract | Operator::Multiply | Operator::Divide => {
                        // Check for division by zero
                        if *op == Operator::Divide {
                            if let Expression::Literal(Literal::Int(0)) = **right {
                                // Get the position of the zero literal if possible
                                let (zero_line, zero_col) = if !self.zero_literals.is_empty() {
                                    self.zero_literals[0]
                                } else {
                                    (line, column + 1) // Estimate position of the divisor
                                };

                                self.add_error(SemanticError::DivisionByZero {
                                    line: zero_line,
                                    column: zero_col,
                                });
                                return None;
                            } else if let Expression::Literal(Literal::Float(f)) = **right {
                                if f == 0.0 {
                                    // Get position of the zero literal
                                    let (zero_line, zero_col) = if !self.zero_literals.is_empty() {
                                        self.zero_literals[0]
                                    } else {
                                        (line, column + 1) // Estimate position
                                    };

                                    self.add_error(SemanticError::DivisionByZero {
                                        line: zero_line,
                                        column: zero_col,
                                    });
                                    return None;
                                }
                            }
                        }

                        // For arithmetic operations, allow mixed numeric types
                        if self.are_types_compatible(&left_type, &right_type) {
                            // Return the resulting type (Float if mixing Int and Float)
                            Some(self.resulting_type(&left_type, &right_type))
                        } else {
                            self.add_error(SemanticError::TypeMismatch {
                                expected: format!("{}", left_type),
                                found: format!("{}", right_type),
                                line,
                                column,
                            });
                            None
                        }
                    }

                    // Comparison operators
                    Operator::GreaterThan
                    | Operator::LessThan
                    | Operator::GreaterEqual
                    | Operator::LessEqual
                    | Operator::Equal
                    | Operator::NotEqual => {
                        // For comparison operations, allow mixed numeric types
                        if self.are_types_compatible(&left_type, &right_type) {
                            // Comparison operations return boolean (represented as Int)
                            Some(Type::Int)
                        } else {
                            self.add_error(SemanticError::TypeMismatch {
                                expected: format!("{}", left_type),
                                found: format!("{}", right_type),
                                line,
                                column,
                            });
                            None
                        }
                    }

                    // Logical operators
                    Operator::And | Operator::Or => {
                        // Logical operations work on boolean values (Int)
                        if left_type != Type::Int || right_type != Type::Int {
                            self.add_error(SemanticError::TypeMismatch {
                                expected: "Int".to_string(),
                                found: format!("{}, {}", left_type, right_type),
                                line,
                                column,
                            });
                            return None;
                        }

                        // Logical operations return boolean (represented as Int)
                        Some(Type::Int)
                    }
                }
            }
            Expression::UnaryOp(op, expr) => {
                // Check the type of the operand
                let expr_type = self.analyze_expression(expr);
                expr_type.as_ref()?;

                let expr_type = expr_type.unwrap();

                match op {
                    // UnaryOperator::Negate => {
                    //     // Negation requires a numeric type
                    //     if expr_type != Type::Int && expr_type != Type::Float {
                    //         let (line, column) = if let Some(pos) = &self.current_expr_pos {
                    //             (pos.line, pos.column)
                    //         } else {
                    //             (1, 1)
                    //         };
                    //
                    //         self.add_error(SemanticError::TypeMismatch {
                    //             expected: "numeric type".to_string(),
                    //             found: format!("{}", expr_type),
                    //             line,
                    //             column,
                    //         });
                    //         return None;
                    //     }
                    //
                    //     // Negation returns the same type
                    //     Some(expr_type)
                    // }
                    UnaryOperator::Not => {
                        // Logical negation requires a boolean value (Int)
                        if expr_type != Type::Int {
                            let (line, column) = if let Some(pos) = &self.current_expr_pos {
                                (pos.line, pos.column)
                            } else {
                                (1, 1)
                            };

                            self.add_error(SemanticError::TypeMismatch {
                                expected: "Int".to_string(),
                                found: format!("{}", expr_type),
                                line,
                                column,
                            });
                            return None;
                        }

                        // Logical negation returns a boolean (Int)
                        Some(Type::Int)
                    }
                }
            }
        }
    }
}
