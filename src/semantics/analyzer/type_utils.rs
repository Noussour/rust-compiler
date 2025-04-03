use crate::parser::ast::{Expression, Literal, Type};
use crate::semantics::analyzer::SemanticAnalyzer;

impl SemanticAnalyzer {
    /// Checks if two types are compatible for operations
    pub(crate) fn are_types_compatible(&self, type1: &Type, type2: &Type) -> bool {
        // Same types are always compatible
        if type1 == type2 {
            return true;
        }

        // For numeric operations, Int and Float can work together
        matches!(
            (type1, type2),
            (Type::Int, Type::Float) | (Type::Float, Type::Int)
        )
    }

    /// Determines the resulting type when operating on two types
    pub(crate) fn resulting_type(&self, type1: &Type, type2: &Type) -> Type {
        if type1 == type2 {
            return type1.clone();
        }

        // If either type is Float, the result is Float
        if *type1 == Type::Float || *type2 == Type::Float {
            return Type::Float;
        }

        // Default to the first type
        type1.clone()
    }

    /// Extracts a literal value from an expression if possible
    pub(crate) fn extract_literal(&self, expr: &Expression) -> Option<Literal> {
        match expr {
            Expression::Literal(lit) => Some(lit.clone()),
            _ => None,
        }

        // let mut _current_expr = expr;
        //
        // loop {
        //     match current_expr {
        //         Expression::Literal(lit) => return Some(lit.clone()),
        //         Expression::UnaryOp(UnaryOperator::Negate, inner) => match inner.as_ref() {
        //             Expression::Literal(Literal::Int(val)) => {
        //                 return Some(Literal::Int(-val));
        //             }
        //             crate::parser::ast::Expression::Literal(
        //                 crate::parser::ast::Literal::Float(val),
        //             ) => {
        //                 return Some(Literal::Float(-val));
        //             }
        //             _ => current_expr = inner,
        //         },
        //         _ => return None,
        //     }
        // }
    }
}
