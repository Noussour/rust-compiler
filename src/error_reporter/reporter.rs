use crate::parser::error::ParseError;
use crate::semantics::error::SemanticError;
use colored::*;
use std::fmt;

/// Represents different types of compiler errors
#[derive(Debug, Clone)]
pub enum ErrorKind {
    Lexical,
    Syntax,
    Semantic,
    Internal,
}

/// A structured compiler error with location information
#[derive(Debug, Clone)]
pub struct CompilerError {
    kind: ErrorKind,
    message: String,
    file_path: String,
    line: usize,
    column: usize,
    source_line: Option<String>,
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
    
    /// Add a syntax error from the parser
    pub fn add_parse_error(&mut self, error: &ParseError) {
        match error {
            ParseError::SyntaxError { message, line, column } => {
                self.add_error(ErrorKind::Syntax, message, *line, *column);
            }
            ParseError::UnexpectedToken { expected, found, line, column } => {
                let msg = format!("Unexpected token: expected {}, found {}", expected, found);
                self.add_error(ErrorKind::Syntax, &msg, *line, *column);
            }
            ParseError::UnexpectedEOF { expected, line, column } => {
                let msg = format!("Unexpected end of file, expected {}", expected);
                self.add_error(ErrorKind::Syntax, &msg, *line, *column);
            }
            ParseError::Other(msg) => {
                self.add_error(ErrorKind::Syntax, msg, 1, 1);
            }
        }
    }
    
    /// Add a semantic error from the analyzer
    pub fn add_semantic_error(&mut self, error: &SemanticError) {
        match error {
            SemanticError::UndeclaredIdentifier { name, line, column } => {
                let msg = format!("Undeclared identifier '{}'", name);
                self.add_error(ErrorKind::Semantic, &msg, *line, *column);
            }
            SemanticError::DuplicateDeclaration { name, line, column, original_line, original_column } => {
                let msg = format!(
                    "Duplicate declaration of '{}' (originally declared at line {}, column {})",
                    name, original_line, original_column
                );
                self.add_error(ErrorKind::Semantic, &msg, *line, *column);
            }
            SemanticError::TypeMismatch { expected, found, line, column } => {
                let msg = format!("Type mismatch: expected {}, found {}", expected, found);
                self.add_error(ErrorKind::Semantic, &msg, *line, *column);
            }
            SemanticError::DivisionByZero { line, column } => {
                self.add_error(ErrorKind::Semantic, "Division by zero", *line, *column);
            }
            SemanticError::ConstantModification { name, line, column } => {
                let msg = format!("Cannot modify constant '{}'", name);
                self.add_error(ErrorKind::Semantic, &msg, *line, *column);
            }
            SemanticError::ArrayIndexOutOfBounds { name, index, size, line, column } => {
                let msg = format!(
                    "Array index out of bounds: index {} exceeds size {} for array '{}'", 
                    index, size, name
                );
                self.add_error(ErrorKind::Semantic, &msg, *line, *column);
            }
            SemanticError::Other(msg) => {
                self.add_error(ErrorKind::Semantic, msg, 1, 1);
            }
        }
    }
    
    /// Add a lexical error directly from a token
    pub fn add_lexical_error(&mut self, text: &str, line: usize, column: usize) {
        let message = format!("Invalid token: '{}'", text);
        self.add_error(ErrorKind::Lexical, &message, line, column);
    }
    
    /// Display all collected errors with detailed formatting
    pub fn report_errors(&self) -> bool {
        if self.errors.is_empty() {
            return false;
        }
        
        println!("\n{}\n", format!("Found {} errors:", self.errors.len()).red().bold());
        
        // Group errors by kind for better organization
        let mut lexical_errors = Vec::new();
        let mut syntax_errors = Vec::new();
        let mut semantic_errors = Vec::new();
        let mut other_errors = Vec::new();
        
        for error in &self.errors {
            match error.kind {
                ErrorKind::Lexical => lexical_errors.push(error),
                ErrorKind::Syntax => syntax_errors.push(error),
                ErrorKind::Semantic => semantic_errors.push(error),
                ErrorKind::Internal => other_errors.push(error),
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
    
    fn print_error_group(&self, errors: &[&CompilerError]) {
        for (i, error) in errors.iter().enumerate() {
            // Print error header with number and message
            println!("  {} {}: {}", (i + 1).to_string().cyan(), "Error".red(), error.message.white().bold());
            println!("    {} {}:{}:{}", "-->".blue(), error.file_path, error.line, error.column);
            
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
            
            println!(); // Add spacing between errors
        }
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
