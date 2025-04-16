use std::ops::Range;

use crate::parser::ast::{
    Expression, ExpressionKind, Literal, LiteralKind, Located, Operator, Type, UnaryOperator,
};
use crate::semantics::{analyzer_core::SemanticAnalyzer, symbol_table::SymbolKind};

pub struct ValueType {
    pub value: Option<f32>,
    pub typ: Type,
}

impl ValueType {
    fn new(typ: Type, value: Option<f32>) -> Self {
        ValueType { value, typ }
    }

    pub fn get_value(&self) -> Option<f32> {
        self.value
    }
    pub fn get_type(&self) -> &Type {
        &self.typ
    }
}

impl PartialEq<Type> for ValueType {
    fn eq(&self, other: &Type) -> bool {
        &self.typ == other
    }
}

impl From<Type> for ValueType {
    fn from(typ: Type) -> Self {
        ValueType { value: None, typ }
    }
}


impl SemanticAnalyzer {
    pub fn analyze_expression(&mut self, expr: &Expression) -> Option<ValueType> {
        match &expr.node {
            ExpressionKind::Identifier(name) => self.handle_identifier(name, &expr.span),
            ExpressionKind::ArrayAccess(name, index_expression) => {
                self.handle_array_access(name, index_expression, &expr.span)
            }
            ExpressionKind::Literal(value) => self.handle_literal(value),
            ExpressionKind::BinaryOp(left_expression, operator, right_expression) => {
                self.handle_binary_operation(left_expression, operator, right_expression)
            }
            ExpressionKind::UnaryOp(unary_operator, located) => {
                self.handle_unary_operation(unary_operator, located, &expr.span)
            }
        }
    }

    fn handle_identifier(&mut self, name: &str, span: &Range<usize>) -> Option<ValueType> {
        if !self.symbol_table.contains(name) {
            self.undeclared_identifier_error(span, name);
            return None;
        }

        let symbol = self.symbol_table.get(name).unwrap();
        Some(ValueType::new(
            symbol.symbol_type.clone(),
            symbol.value.as_ref().and_then(|v| match v {
                LiteralKind::Float(f) => Some(*f),
                LiteralKind::Int(i) => Some(*i as f32),
                _ => None,
            }),
        ))
    }

    fn handle_array_access(
        &mut self,
        name: &str,
        index_expression: &Expression,
        span: &Range<usize>,
    ) -> Option<ValueType> {
        if !self.symbol_table.contains(name) {
            self.undeclared_identifier_error(span, name);
            return None;
        }

        let symbol = self.symbol_table.get(name).unwrap();
        match &symbol.kind {
            SymbolKind::Array(size) => {
                let symbol_type = symbol.symbol_type.clone();
                let array_size = *size;

                if let ExpressionKind::Literal(Located {
                    node: LiteralKind::Int(idx),
                    ..
                }) = &index_expression.node
                {
                    if *idx < 0 || *idx as usize >= array_size {
                        self.array_index_out_of_bounds_error(
                            &index_expression.span,
                            name,
                            *idx as usize,
                            array_size,
                        );
                        return None;
                    }
                }

                let idx_type = self.analyze_expression(index_expression);
                if let Some(idx_type) = idx_type {
                    if idx_type.typ != Type::Int {
                        self.type_mismatch_error(
                            &index_expression.span,
                            &Type::Int,
                            &idx_type.typ,
                            Some("array index"),
                        );
                        return None;
                    }
                } else {
                    return None;
                }

                Some(ValueType::new(symbol_type, None))
            }
            _ => None,
        }
    }

    fn handle_literal(&mut self, literal: &Literal) -> Option<ValueType> {
        match literal.node {
            LiteralKind::Int(value) => {
                Some(ValueType::new(Type::Int, Some(value as f32)))
            }
            LiteralKind::Float(value) => {

                Some(ValueType::new(Type::Float, Some(value)))
            }
            _ => None,
        }
    }

    fn handle_binary_operation(
        &mut self,
        left: &Expression,
        operator: &Operator,
        right: &Expression,
    ) -> Option<ValueType> {
        let left_type = self.analyze_expression(left);
        let right_type = self.analyze_expression(right);

        if left_type.is_none() || right_type.is_none() {
            return None;
        }

        let left_type = left_type.unwrap();
        let right_type = right_type.unwrap();

        match operator {
            Operator::Add | Operator::Subtract | Operator::Multiply | Operator::Divide => {
                if left_type.typ != Type::Int && left_type.typ != Type::Float {
                    self.type_mismatch_error(
                        &(left.span.start..right.span.end),
                        &Type::Int,
                        &left_type.typ,
                        Some("arithmetic"),
                    );
                    return None;
                }
                if right_type.typ != Type::Int && right_type.typ != Type::Float {
                    self.type_mismatch_error(
                        &(left.span.start..right.span.end),
                        &Type::Int,
                        &right_type.typ,
                        Some("arithmetic"),
                    );
                    return None;
                }

                if *operator == Operator::Divide {
                    if let Some(right_value) = self.evaluate_constant_expression(right) {
                        match right_value {
                            LiteralKind::Int(0) => {
                                self.division_by_zero_error(&right.span);
                                return None;
                            }
                            LiteralKind::Float(f) if f == 0.0 => {
                                self.division_by_zero_error(&right.span);
                                return None;
                            }
                            _ => {}
                        }
                    }
                }

                let result_value = match (left_type.value, right_type.value, operator) {
                    (Some(l), Some(r), Operator::Add) => Some(l + r),
                    (Some(l), Some(r), Operator::Subtract) => Some(l - r),
                    (Some(l), Some(r), Operator::Multiply) => Some(l * r),
                    (Some(l), Some(r), Operator::Divide) if r != 0.0 => Some(l / r),
                    _ => None,
                };

                if left_type.typ == Type::Float || right_type.typ == Type::Float {
                    Some(ValueType::new(Type::Float, result_value))
                } else {
                    Some(ValueType::new(Type::Int, result_value))
                }
            }
            Operator::GreaterThan
            | Operator::LessThan
            | Operator::GreaterEqual
            | Operator::LessEqual
            | Operator::Equal
            | Operator::NotEqual => {
                if left_type.typ != Type::Int && left_type.typ != Type::Float {
                    self.type_mismatch_error(
                        &(left.span.start..right.span.end),
                        &Type::Int,
                        &left_type.typ,
                        Some("comparison"),
                    );
                    return None;
                }
                if right_type.typ != Type::Int && right_type.typ != Type::Float {
                    self.type_mismatch_error(
                        &(left.span.start..right.span.end),
                        &Type::Int,
                        &right_type.typ,
                        Some("comparison"),
                    );
                    return None;
                }

                Some(ValueType::new(Type::Bool, None))
            }
            Operator::And | Operator::Or => {
                if left_type.typ != Type::Bool {
                    self.type_mismatch_error(
                        &(left.span.start..right.span.end),
                        &Type::Bool,
                        &left_type.typ,
                        Some("logical"),
                    );
                    return None;
                }
                if right_type.typ != Type::Bool {
                    self.type_mismatch_error(
                        &(left.span.start..right.span.end),
                        &Type::Bool,
                        &right_type.typ,
                        Some("logical"),
                    );
                    return None; 
                }

                Some(ValueType::new(Type::Bool, None))
            }
        }
    }

    fn handle_unary_operation(
        &mut self,
        unary_operator: &UnaryOperator,
        expression: &Expression,
        span: &Range<usize>,
    ) -> Option<ValueType> {
        let expression_type = self.analyze_expression(expression)?;

        match unary_operator {
            UnaryOperator::Not => {
                if expression_type.typ != Type::Bool {
                    self.type_mismatch_error(
                        span,
                        &Type::Bool,
                        &expression_type.typ,
                        Some("logical"),
                    );
                    return None;
                }

                Some(ValueType::new(Type::Bool, None))
            }
        }
    }
}

