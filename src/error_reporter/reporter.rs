use crate::parser::error::SyntaxError;
use crate::semantics::error::SemanticError;
use colored::*;
use std::cmp::Ordering;
use std::fmt;

/// Represents different types of compiler errors
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ErrorKind {
    Lexical,
    Syntax,
    Semantic,
    Internal,
}

/// A structured compiler error with location information
#[derive(Debug, Clone)]
pub struct CompilerError {
    pub kind: ErrorKind,
    pub message: String,
    pub file_path: String,
    pub line: usize,
    pub column: usize,
    pub source_line: Option<String>,
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {}:{}:{}: {}",
            self.kind_str(),
            self.file_path,
            self.line,
            self.column,
            self.message
        )
    }
}

impl CompilerError {
    fn kind_str(&self) -> ColoredString {
        match self.kind {
            ErrorKind::Lexical => "Lexical error".red().bold(),
            ErrorKind::Syntax => "Syntax error".red().bold(),
            ErrorKind::Semantic => "Semantic error".red().bold(),
            ErrorKind::Internal => "Internal error".red().bold(),
        }
    }
}

// Add PartialEq implementation for testing and sorting
impl PartialEq for CompilerError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.message == other.message
            && self.line == other.line
            && self.column == other.column
    }
}

// Add ordering for sorting errors
impl PartialOrd for CompilerError {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.line
                .cmp(&other.line)
                .then_with(|| self.column.cmp(&other.column))
                .then_with(|| self.kind.partial_cmp(&other.kind).unwrap()),
        )
    }
}

/// The main error reporter that collects and displays errors
pub struct ErrorReporter {
    file_path: String,
    source_lines: Vec<String>,
    errors: Vec<CompilerError>,
}

impl ErrorReporter {
    /// Create a new error reporter for a specific source file
    pub fn new(source: &str, file_path: &str) -> Self {
        let source_lines = source.lines().map(String::from).collect();

        Self {
            file_path: file_path.to_string(),
            source_lines,
            errors: Vec::new(),
        }
    }

    /// Add a generic error
    pub fn add_error(&mut self, kind: ErrorKind, message: &str, line: usize, column: usize) {
        let source_line = if line > 0 && line <= self.source_lines.len() {
            Some(self.source_lines[line - 1].clone())
        } else {
            None
        };

        self.errors.push(CompilerError {
            kind,
            message: message.to_string(),
            file_path: self.file_path.clone(),
            line,
            column,
            source_line,
        });
    }

    /// Display all collected errors with detailed formatting and suggestions
    pub fn report_errors(&self) -> bool {
        if self.errors.is_empty() {
            return false;
        }

        println!(
            "\n{}\n",
            format!("Found {} errors:", self.errors.len()).red().bold()
        );

        // Sort errors by line number for a more logical presentation
        let sorted_errors = self.sort_errors_by_line();

        // Group errors by kind for better organization
        let mut lexical_errors = Vec::new();
        let mut syntax_errors = Vec::new();
        let mut semantic_errors = Vec::new();
        let mut other_errors = Vec::new();

        for error in &sorted_errors {
            match error.kind {
                ErrorKind::Lexical => lexical_errors.push(*error),
                ErrorKind::Syntax => syntax_errors.push(*error),
                ErrorKind::Semantic => semantic_errors.push(*error),
                ErrorKind::Internal => other_errors.push(*error),
            }
        }

        // Print each category with a header
        if !lexical_errors.is_empty() {
            println!("{}", "Lexical Errors:".yellow().bold());
            self.print_error_group(&lexical_errors);
        }

        if !syntax_errors.is_empty() {
            println!("{}", "Syntax Errors:".yellow().bold());
            self.print_error_group(&syntax_errors);
        }

        if !semantic_errors.is_empty() {
            println!("{}", "Semantic Errors:".yellow().bold());
            self.print_error_group(&semantic_errors);
        }

        if !other_errors.is_empty() {
            println!("{}", "Other Errors:".yellow().bold());
            self.print_error_group(&other_errors);
        }

        true
    }

    // Helper method to sort errors by line number for better presentation
    fn sort_errors_by_line(&self) -> Vec<&CompilerError> {
        let mut errors = self.errors.iter().collect::<Vec<_>>();
        errors.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.column.cmp(&b.column)));
        errors
    }

    fn print_error_group(&self, errors: &[&CompilerError]) {
        for (i, error) in errors.iter().enumerate() {
            // Print error header with number and message
            println!(
                "  {} {}: {}",
                (i + 1).to_string().cyan(),
                "Error".red(),
                error.message.white()
            );
            println!(
                "    {} {}:{}:{}",
                "-->".blue(),
                error.file_path,
                error.line,
                error.column
            );

            // Print source context if available
            if let Some(source_line) = &error.source_line {
                // Line number and source code
                let line_num = format!("{:>6} | ", error.line).blue();
                println!("{}{}", line_num, source_line);

                // Error indicator pointing to the exact column
                let mut pointer = String::new();
                for _ in 0..error.column.saturating_sub(1) {
                    pointer.push(' ');
                }
                pointer.push_str("^---");

                println!("{}{}", "       | ".blue(), pointer.bright_red().bold());
            }

            // Add suggestions for fixing common errors
            if let Some(suggestion) = self.get_suggestion_for_error(error) {
                println!("       {}: {}", "Suggestion".cyan().bold(), suggestion);
            }

            println!(); // Add spacing between errors
        }
    }

    // Provide helpful suggestions for common errors
    fn get_suggestion_for_error(&self, error: &CompilerError) -> Option<String> {
        match error.kind {
            ErrorKind::Lexical => {
                if error.message.contains("Unterminated string") {
                    return Some("Add a closing double quote".to_string());
                } else if error.message.contains("Identifier too long") {
                    return Some("Shorten the identifier to 14 characters or less".to_string());
                } else if error.message.contains("consecutive underscores") {
                    return Some("Remove consecutive underscores".to_string());
                } else if error.message.contains("cannot start with a number") {
                    return Some("Start identifier with a letter instead of a number".to_string());
                } else if error.message.contains("Integer literal out of range") {
                    return Some("Ensure integer values are between -32768 and 32767".to_string());
                }
            }
            ErrorKind::Syntax => {
                if error.message.contains("Unexpected token") {
                    return Some("Check syntax around this position".to_string());
                } else if error.message.contains("Unexpected end of file") {
                    return Some("The program may be incomplete".to_string());
                }
            }
            ErrorKind::Semantic => {
                if error.message.contains("Undeclared identifier") {
                    return Some("Declare this variable before using it".to_string());
                } else if error.message.contains("Duplicate declaration") {
                    return Some("Use a different name for this variable".to_string());
                } else if error.message.contains("Type mismatch") {
                    return Some(
                        "Ensure the types match or add an explicit conversion".to_string(),
                    );
                } else if error.message.contains("Division by zero") {
                    return Some("Check your division operation to avoid zero divisor".to_string());
                } else if error.message.contains("Cannot modify constant") {
                    return Some("Constants cannot be modified after declaration".to_string());
                } else if error.message.contains("Array index out of bounds") {
                    return Some("Ensure index is within the array size".to_string());
                }
            }
            _ => {}
        }
        None
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get all errors
    pub fn get_errors(&self) -> &[CompilerError] {
        &self.errors
    }
}
