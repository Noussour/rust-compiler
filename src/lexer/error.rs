use crate::lexer::lexer_core::TokenWithMetaData;
use std::fmt;
use std::error::Error;
use crate::error_reporter::format_code_context;
use crate::error_reporter::ErrorReporter;
use colored::Colorize;

#[derive(Debug, Clone, PartialEq)]
pub enum LexicalErrorType {
    UnterminatedString,
    NonAsciiCharacters,
    IdentifierTooLong ,
    ConsecutiveUnderscores,
    TrailingUnderscore,
    IdentifierStartsWithNumber,
    IntegerOutOfRange,
    InvalidToken,
}

#[derive(Debug)]
pub struct LexicalError {
    pub invalid_token: String,
    pub line: usize,
    pub column: usize,
    pub error_type: LexicalErrorType,
}

impl LexicalError {
    /// Create a new `LexicalError` by analyzing the token value.
    pub fn new(token: TokenWithMetaData) -> Self {
        let error_type = if token.value.starts_with('"') && !token.value.ends_with('"') {
            LexicalErrorType::UnterminatedString
        } else if token.value.contains(|c: char| !c.is_ascii()) {
            LexicalErrorType::NonAsciiCharacters
        } else if token.value.len() > 14 {
            LexicalErrorType::IdentifierTooLong
        } else if token.value.contains("__") {
            LexicalErrorType::ConsecutiveUnderscores
        } else if token.value.ends_with('_') {
            LexicalErrorType::TrailingUnderscore
        } else if token.value.starts_with(|c: char| c.is_numeric()) {
            LexicalErrorType::IdentifierStartsWithNumber
        } else {
            LexicalErrorType::InvalidToken
        };

        LexicalError {
            invalid_token: token.value,
            line: token.line,
            column: token.column,
            error_type,
        }
    }
}

impl ErrorReporter for LexicalError {
    fn report(&self, source_code: Option<&str>) -> String {
        let mut result = String::new();

        // Error header with type and location
        result.push_str(&format!("{}: {}\n", 
            "Lexical Error".red().bold(), 
            self.get_error_description()));
        
        // File and position information
        result.push_str(&format!("{} line {}, column {}\n", 
            "-->".blue(),
            self.line,
            self.column));

        // Source context if available
        if let Some(source) = source_code {
            // Split source code into lines and get the relevant line
            let lines: Vec<&str> = source.lines().collect();
            if self.line <= lines.len() {
                let line: &str = lines[self.line - 1];
                result.push_str(&format_code_context(
                    line, 
                    self.column, 
                    self.invalid_token.len()));
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
        match &self.error_type {
            LexicalErrorType::UnterminatedString => {
                Some(format!("Add a closing quote: {}\"", self.invalid_token))
            },
            LexicalErrorType::NonAsciiCharacters => {
                Some("Use only ASCII characters in identifiers and strings".to_string())
            },
            LexicalErrorType::IdentifierTooLong => {
                Some("Identifiers must be 14 characters or less".to_string())
            },
            LexicalErrorType::ConsecutiveUnderscores => {
                let fixed = self.invalid_token.replace("__", "_");
                Some(format!("Use single underscores: '{}'", fixed))
            },
            LexicalErrorType::TrailingUnderscore => {
                let fixed = self.invalid_token.trim_end_matches('_');
                Some(format!("Remove trailing underscore: '{}'", fixed))
            },
            LexicalErrorType::IdentifierStartsWithNumber => {
                let _first_non_digit = self.invalid_token
                    .find(|c: char| !c.is_numeric())
                    .unwrap_or(0);
                let fixed = format!("_{}", self.invalid_token);
                Some(format!("Identifiers can't start with numbers. Try: '{}'", fixed))
            },
            LexicalErrorType::IntegerOutOfRange => {
                Some("Integer literals must be within the valid range".to_string())
            },
            LexicalErrorType::InvalidToken => {
                Some("Check for unrecognized symbols or incorrect syntax".to_string())
            }
        }
    }

    fn get_error_name(&self) -> String {
        "Lexical Error".to_string()
    }

    fn get_location_info(&self) -> (usize, usize) {
        (self.line, self.column)
    }
}

impl LexicalError {
    fn get_error_description(&self) -> String {
        match self.error_type {
            LexicalErrorType::UnterminatedString => 
                format!("Unterminated string '{}' - missing closing quote", self.invalid_token),
            LexicalErrorType::NonAsciiCharacters => 
                format!("Non-ASCII characters in '{}'", self.invalid_token),
            LexicalErrorType::IdentifierTooLong => 
                format!("Identifier '{}' exceeds maximum length of 14 characters", self.invalid_token),
            LexicalErrorType::ConsecutiveUnderscores => 
                format!("Consecutive underscores in identifier '{}'", self.invalid_token),
            LexicalErrorType::TrailingUnderscore => 
                format!("Identifier '{}' ends with underscore", self.invalid_token),
            LexicalErrorType::IdentifierStartsWithNumber => 
                format!("Identifier '{}' starts with a number", self.invalid_token),
            LexicalErrorType::IntegerOutOfRange => 
                format!("Integer '{}' is out of range", self.invalid_token),
            LexicalErrorType::InvalidToken => 
                format!("Invalid token '{}'", self.invalid_token),
        }
    }
}

// Implement Display for LexicalError
impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.report(None))
    }
}

// Implement the Error trait for LexicalError
impl Error for LexicalError {}