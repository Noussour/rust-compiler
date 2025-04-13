use crate::parser::ast::{Expression, Literal, Operator, Type, UnaryOperator};
use crate::semantics::analyzer_core::SemanticAnalyzer;
use crate::semantics::error::SemanticError;

impl SemanticAnalyzer {
    /// Analyzes an expression to determine its type
    /// Returns the type of the expression, or None if there is an error
    pub(crate) fn analyze_expression(&mut self, expr: &Expression) -> Option<Type> {
        match &expr.node {
            Expression::Identifier(name) => {
                // Check if the identifier exists
                if !self.symbol_table.contains(name) {
                    self.undeclared_identifier_error(expr.span, name);
                    return None;
                }

                // Return the identifier's type
                let symbol = self.symbol_table.get(name).unwrap();
                Some(symbol.symbol_type.clone())
            }
            Expression::ArrayAccess(name, index_expr) => {
                // Check if the array exists
                if !self.symbol_table.contains(name) {
                    self.undeclared_identifier_error(expr.span, name);
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
                        if let Expression::Literal(Literal::Int(idx)) = &index_expr.node {
                            if *idx < 0 || *idx as usize >= array_size {
                                self.array_index_out_of_bounds_error(
                                    expr.span,
                                    name,
                                    *idx as usize,
                                    array_size,
                                );
                                return None;
                            }
                        }

                        // Check index expression type
                        let idx_type = self.analyze_expression(index_expr);
                        if let Some(idx_type) = idx_type {
                            if idx_type != Type::Int {
                                self.type_mismatch_error(
                                    index_expr.span,
                                    &Type::Int,
                                    &idx_type,
                                    Some("array index"),
                                );
                                return None;
                            }
                        } else {
                            return None;
                        }

                        // Return the array element type
                        Some(symbol_type)
                    }
                    _ => {
                        self.other_error(format!("Cannot index non-array variable '{}'", name));
                        None
                    }
                }
            }
            Expression::Literal(lit) => {
                // Store zero literal positions for division by zero checks
                match lit {
                    Literal::Int(value) => {
                        if *value == 0 {
                            let (line, column) = self.get_span_location(&expr.span);
                            self.zero_literals.push((line, column));
                        }
                        Some(Type::Int)
                    }
                    Literal::Float(value) => {
                        if *value == 0.0 {
                            let (line, column) = self.get_span_location(&expr.span);
                            self.zero_literals.push((line, column));
                        }
                        Some(Type::Float)
                    }
                    Literal::String(_) => None, // No string type in MiniSoft
                }
            }
            Expression::BinaryOp(left, op, right) => {
                // Check the types of left and right operands
                let left_type = self.analyze_expression(left);
                let right_type = self.analyze_expression(right);

                if left_type.is_none() || right_type.is_none() {
                    return None;
                }

                let left_type = left_type.unwrap();
                let right_type = right_type.unwrap();

                match op {
                    // Arithmetic operators
                    Operator::Add | Operator::Subtract | Operator::Multiply | Operator::Divide => {
                        // Check for division by zero
                        if *op == Operator::Divide {
                            if let Expression::Literal(Literal::Int(0)) = right.node {
                                self.division_by_zero_error(right.span);
                                return None;
                            } else if let Expression::Literal(Literal::Float(f)) = right.node {
                                if f == 0.0 {
                                    self.division_by_zero_error(right.span);
                                    return None;
                                }
                            }
                        }

                        // For arithmetic operations, allow mixed numeric types
                        if self.are_types_compatible(&left_type, &right_type) {
                            Some(self.resulting_type(&left_type, &right_type))
                        } else {
                            self.type_mismatch_error(
                                expr.span,
                                &left_type,
                                &right_type,
                                Some("arithmetic"),
                            );
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
                            self.type_mismatch_error(
                                expr.span,
                                &left_type,
                                &right_type,
                                Some("comparison"),
                            );
                            None
                        }
                    }

                    // Logical operators
                    Operator::And | Operator::Or => {
                        // Logical operations work on boolean values (Int)
                        if left_type != Type::Int || right_type != Type::Int {
                            self.type_mismatch_error(
                                expr.span,
                                &Type::Int,
                                &Type::Int,
                                Some("logical"),
                            );
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
                    UnaryOperator::Not => {
                        // Logical negation requires a boolean value (Int)
                        if expr_type != Type::Int {
                            self.type_mismatch_error(
                                expr.span,
                                &Type::Int,
                                &expr_type,
                                Some("logical"),
                            );
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
