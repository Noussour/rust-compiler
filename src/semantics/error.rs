use colored::Colorize;
use std::fmt;
use crate::error_reporter::ErrorReporter;
use crate::error_reporter::format_code_context;


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
        context: Option<String>,
    },

    /// Division by zero
    DivisionByZero { 
        line: usize, 
        column: usize 
    },

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

impl ErrorReporter for SemanticError {
    fn report(&self, source_code: Option<&str>) -> String {
        let mut result = String::new();
        
        // Error header with detailed message
        result.push_str(&format!("{}: {}\n", 
            "Semantic Error".red().bold(), 
            self.get_detailed_message()));
        
        // File and position information
        let (line, column) = self.get_location_info();
        result.push_str(&format!("{} line {}, column {}\n", 
            "-->".blue(),
            line,
            column));
        
        // Source context if available
        if let Some(source) = source_code {
            let lines: Vec<&str> = source.lines().collect();
            if line <= lines.len() {
                let line_content = lines[line - 1];
                
                // For duplicate declarations, highlight both occurrences
                if let SemanticError::DuplicateDeclaration { original_line, .. } = self {
                    // Current declaration
                    result.push_str(&format_code_context(line_content, column, self.get_token_length()));
                    
                    // Original declaration
                    if *original_line <= lines.len() {
                        let original_content = lines[original_line - 1];
                        result.push_str(&format!("\n{} {}\n", 
                            "First declared at line:".yellow(),
                            original_line));
                        result.push_str(&format!("{}{}\n", 
                            " | ".blue(),
                            original_content));
                    }
                } else {
                    result.push_str(&format_code_context(line_content, column, self.get_token_length()));
                }
            }
        }
        
        // Add suggestion if available
        if let Some(suggestion) = self.get_suggestion() {
            result.push_str(&format!("{}: {}\n", 
                "Suggestion".cyan().bold(), 
                suggestion));
        }
        
        result
    }

    fn get_suggestion(&self) -> Option<String> {
        match self {
            SemanticError::UndeclaredIdentifier { name, .. } => {
                Some(format!("Declare variable '{}' before using it", name))
            },
            SemanticError::DuplicateDeclaration { name, .. } => {
                Some(format!("Use a different name for the second declaration of '{}'", name))
            },
            SemanticError::TypeMismatch { expected, found, context, .. } => {
                match context {
                    Some(ctx) if ctx == "assignment" => {
                        Some(format!("Make sure the types match. Try converting from '{}' to '{}'", found, expected))
                    }
                    Some(ctx) if ctx == "condition" => {
                        Some(format!("Conditions must be of boolean type, found '{}' instead", found))
                    }
                    Some(ctx) if ctx == "arithmetic" => {
                        Some(format!("Cannot perform arithmetic operation between '{}' and '{}'", expected, found))
                    }
                    _ => {
                        Some(format!("Expected type '{}', but found '{}'. Consider adding a type conversion", expected, found))
                    }
                }
            },
            SemanticError::DivisionByZero { .. } => {
                Some("Check for division by zero or ensure denominators are non-zero".to_string())
            },
            SemanticError::ConstantModification { name, .. } => {
                Some(format!("'{}' is a constant and cannot be modified. Consider using a variable instead", name))
            },
            SemanticError::ArrayIndexOutOfBounds { name, size, .. } => {
                Some(format!("Array '{}' has size {}. Use indices from 0 to {}", name, size, size - 1))
            },
            SemanticError::Other(_) => None,
        }
    }

    fn get_error_name(&self) -> String {
        "Semantic Error".to_string()
    }

    fn get_location_info(&self) -> (usize, usize) {
        match self {
            SemanticError::UndeclaredIdentifier { line, column, .. } => (*line, *column),
            SemanticError::DuplicateDeclaration { line, column, .. } => (*line, *column),
            SemanticError::TypeMismatch { line, column, .. } => (*line, *column),
            SemanticError::DivisionByZero { line, column } => (*line, *column),
            SemanticError::ConstantModification { line, column, .. } => (*line, *column),
            SemanticError::ArrayIndexOutOfBounds { line, column, .. } => (*line, *column),
            SemanticError::Other(_) => (0, 0),
        }
    }
}

impl SemanticError {
    fn get_detailed_message(&self) -> String {
        match self {
            SemanticError::UndeclaredIdentifier { name, .. } => {
                format!("Undeclared identifier '{}'", name)
            },
            SemanticError::DuplicateDeclaration { name, original_line, original_column, .. } => {
                format!("Duplicate declaration of '{}' (originally declared at line {}, column {})", 
                    name, original_line, original_column)
            },
            SemanticError::TypeMismatch { expected, found, context, .. } => {
                match context {
                    Some(ctx) => format!("Type mismatch in {}: expected {}, found {}", ctx, expected, found),
                    None => format!("Type mismatch: expected {}, found {}", expected, found),
                }
            },
            SemanticError::DivisionByZero { .. } => {
                "Division by zero detected".to_string()
            },
            SemanticError::ConstantModification { name, .. } => {
                format!("Attempt to modify constant '{}'", name)
            },
            SemanticError::ArrayIndexOutOfBounds { name, index, size, .. } => {
                format!("Array index out of bounds: index {} exceeds size {} for array '{}'", 
                    index, size, name)
            },
            SemanticError::Other(msg) => {
                msg.clone()
            },
        }
    }
    
    fn get_token_length(&self) -> usize {
        match self {
            SemanticError::UndeclaredIdentifier { name, .. } => name.len(),
            SemanticError::DuplicateDeclaration { name, .. } => name.len(),
            SemanticError::TypeMismatch { .. } => 1, // Default token length
            SemanticError::DivisionByZero { .. } => 1,
            SemanticError::ConstantModification { name, .. } => name.len(),
            SemanticError::ArrayIndexOutOfBounds { name, .. } => name.len(),
            SemanticError::Other(_) => 1,
        }
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.report(None))
    }
}

impl std::error::Error for SemanticError {}
