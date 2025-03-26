use std::fmt;

/// Semantic error types
#[derive(Debug, Clone)]
pub enum SemanticError {
    /// Variable not declared before use
    UndeclaredIdentifier {
        name: String,
        line: usize,
        column: usize,
    },

    /// Variable declared multiple times
    DuplicateDeclaration {
        name: String,
        line: usize,
        column: usize,
        original_line: usize,
        original_column: usize,
    },

    /// Type mismatch in operations or assignments
    TypeMismatch {
        expected: String,
        found: String,
        line: usize,
        column: usize,
    },

    /// Division by zero
    DivisionByZero { line: usize, column: usize },

    /// Attempt to modify a constant
    ConstantModification {
        name: String,
        line: usize,
        column: usize,
    },

    /// Array index out of bounds
    ArrayIndexOutOfBounds {
        name: String,
        index: usize,
        size: usize,
        line: usize,
        column: usize,
    },

    /// Catch-all for other semantic problems
    Other(String),
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticError::UndeclaredIdentifier { name, line, column } => {
                write!(
                    f,
                    "Undeclared identifier '{}' at line {}, column {}",
                    name, line, column
                )
            }

            SemanticError::DuplicateDeclaration {
                name,
                line,
                column,
                original_line,
                original_column,
            } => {
                write!(
                    f,
                    "Duplicate declaration of '{}' at line {}, column {}. Originally declared at line {}, column {}",
                    name, line, column, original_line, original_column
                )
            }

            SemanticError::TypeMismatch {
                expected,
                found,
                line,
                column,
            } => {
                write!(
                    f,
                    "Type mismatch at line {}, column {}: expected {}, found {}",
                    line, column, expected, found
                )
            }

            SemanticError::DivisionByZero { line, column } => {
                write!(f, "Division by zero at line {}, column {}", line, column)
            }

            SemanticError::ConstantModification { name, line, column } => {
                write!(
                    f,
                    "Attempt to modify constant '{}' at line {}, column {}",
                    name, line, column
                )
            }

            SemanticError::ArrayIndexOutOfBounds {
                name,
                index,
                size,
                line,
                column,
            } => {
                write!(
                    f,
                    "Array index out of bounds for '{}' at line {}, column {}: index {} exceeds size {}",
                    name, line, column, index, size
                )
            }

            SemanticError::Other(msg) => {
                write!(f, "Semantic error: {}", msg)
            }
        }
    }
}

impl std::error::Error for SemanticError {}
