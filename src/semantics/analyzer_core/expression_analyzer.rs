use std::ops::Range;

use crate::parser::ast::{Expression, ExpressionKind, Literal, LiteralKind, Operator, Located, Type, UnaryOperator};
use crate::semantics::{analyzer_core::SemanticAnalyzer, symbol_table::SymbolKind};

impl SemanticAnalyzer {
    pub fn analyze_expression(&mut self, expr: &Expression) ->  Option<Type>  {
        // Implementation of expression analysis
        match &expr.node {
            ExpressionKind::Identifier(name) => {
                self.handle_identifier(name, expr.span.clone())
                    },
            ExpressionKind::ArrayAccess(name, index_expression) => {
                self.handle_array_access(name, index_expression, expr.span.clone())
                    },
            ExpressionKind::Literal(value) => {
                self.handle_literal(value, expr.span.clone())
                    },
            ExpressionKind::BinaryOp(left_expression, operator, right_expression) => {
                self.handle_binary_operation(left_expression, operator, right_expression)
                    },
            ExpressionKind::UnaryOp(unary_operator, located) => {
                self.handle_unary_operation(unary_operator, located, expr.span.clone())
                    },
        }
    }

    fn handle_identifier(&mut self, name: &str, span: Range<usize>) -> Option<Type> {
        // Check if the identifier exists in the symbol table
        if !self.symbol_table.contains(name) {
            self.undeclared_identifier_error(span, name);
            return None;
        }

        // Return the identifier's type
        let symbol = self.symbol_table.get(name).unwrap();
        Some(symbol.symbol_type.clone())
    }
    
    fn handle_array_access(&mut self, name: &str, index_expression: &Expression, span: Range<usize>) -> Option<Type> {
        // Check if the array exists in the symbol table
        if !self.symbol_table.contains(name) {
            self.undeclared_identifier_error(span, name);
            return None;
        }

        // Check if it's actually an array
        let symbol = self.symbol_table.get(name).unwrap();
        match &symbol.kind {
            SymbolKind::Array(size) => {
                // Save the symbol type before releasing the borrow
                let symbol_type = symbol.symbol_type.clone();
                let array_size = *size;

                // Check if the index is a constant and within bounds
                if let ExpressionKind::Literal(Located { node: LiteralKind::Int(idx), .. }) = &index_expression.node {
                    if *idx < 0 || *idx as usize >= array_size {
                        self.array_index_out_of_bounds_error(
                            index_expression.span.clone(),
                            name,
                            *idx as usize,
                            array_size,
                        );
                        return None;
                    }
                }

                // Check index expression type
                let idx_type = self.analyze_expression(index_expression);
                if let Some(idx_type) = idx_type {
                    if idx_type != Type::Int {
                        self.type_mismatch_error(
                            index_expression.span.clone(),
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
                None
            }
        }
    }

    fn handle_literal(&mut self, literal: &Literal, span: Range<usize>) -> Option<Type> {
        // Store zero literal positions for division by zero checks
        match literal.node {
            LiteralKind::Int(value) => {
                if value == 0 {
                    self.zero_literals.push((span.start, span.end));
                }
                Some(Type::Int)
            }
            LiteralKind::Float(value) => {
                if value == 0.0 {
                    self.zero_literals.push((span.start, span.end));
                }
                Some(Type::Float)
            }
            LiteralKind::String(_) => None, // No string type in MiniSoft
        }
    }
    fn handle_binary_operation(&mut self, left: &Expression, operator: &Operator, right: &Expression) -> Option<Type> {
        // Check the types of left and right operands
        let left_type = self.analyze_expression(left);
        let right_type = self.analyze_expression(right);

        if left_type.is_none() || right_type.is_none() {
            return None;
        }

        let left_type = left_type.unwrap();
        let right_type = right_type.unwrap();

        match operator {
            Operator::Add | Operator::Subtract | Operator::Multiply | Operator::Divide => {
                // Arithmetic operations require numeric types (Int or Float)
                if left_type != Type::Int && left_type != Type::Float {
                    self.type_mismatch_error(
                        left.span.start..right.span.end,
                        &Type::Int,
                        &left_type,
                        Some("arithmetic"),
                    );
                    return None;
                }
                if right_type != Type::Int && right_type != Type::Float {
                    self.type_mismatch_error(
                        left.span.start..right.span.end,
                        &Type::Int,
                        &right_type,
                        Some("arithmetic"),
                    );
                    return None;
                }

                // Division by zero check
                if *operator == Operator::Divide && right_type == Type::Int && right.node == ExpressionKind::Literal(Located { node: LiteralKind::Int(0), span: right.span.clone() }) {
                    self.division_by_zero_error(right.span.clone());
                    return None;
                }

                // Return the type of the result (Int or Float)
                if left_type == Type::Float || right_type == Type::Float {
                    Some(Type::Float)
                } else {
                    Some(Type::Int)
                }
            },
            Operator::GreaterThan | Operator::LessThan | Operator::GreaterEqual | Operator::LessEqual | Operator::Equal | Operator::NotEqual => {
                // Comparison operations require numeric types (Int or Float)
                if left_type != Type::Int && left_type != Type::Float {
                    self.type_mismatch_error(
                        left.span.start..right.span.end,
                        &Type::Int,
                        &left_type,
                        Some("comparison"),
                    );
                    return None;
                }
                if right_type != Type::Int && right_type != Type::Float {
                    self.type_mismatch_error(
                        left.span.start..right.span.end,
                        &Type::Int,
                        &right_type,
                        Some("comparison"),
                    );
                    return None;
                }

                // Return boolean type (Int)
                Some(Type::Bool)
            },
            Operator::And | Operator::Or => {
                // Logical operations require boolean types (Bool)
                if left_type != Type::Bool {
                    self.type_mismatch_error(
                        left.span.start..right.span.end,
                        &Type::Int,
                        &left_type,
                        Some("logical"),
                    );
                    return None;
                }
                if right_type != Type::Bool {
                    self.type_mismatch_error(
                        left.span.start..right.span.end,
                        &Type::Bool,
                        &right_type,
                        Some("logical"),
                    );
                    return None;
                }

                // Return boolean type (Bool)
                Some(Type::Bool)
            },
        }
    }

    fn handle_unary_operation(&mut self, unary_operator: &UnaryOperator, expression: &Expression, span: Range<usize>) -> Option<Type> {
        // Check the type of the operand
        let expression_type = self.analyze_expression(expression);
        expression_type.as_ref()?;

        let expression_type = expression_type.unwrap();

        match unary_operator {
            UnaryOperator::Not => {
                // Logical negation requires a boolean value (Bool)
                if expression_type != Type::Bool {
                    self.type_mismatch_error(
                        span,
                        &Type::Bool,
                        &expression_type,
                        Some("logical"),
                    );
                    return None;
                }

                // Logical negation returns a boolean (Bool)
                Some(Type::Bool)
            }
        }
    }
}


