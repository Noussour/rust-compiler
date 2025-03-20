use std::fmt;

/// Errors that can occur during parsing
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Invalid token or syntax
    SyntaxError {
        message: String,
        line: usize,
        column: usize,
    },

    /// Unknown token
    UnknownToken {
        token: String,
        line: usize,
        column: usize,
    },

    /// Unexpected token
    UnexpectedToken {
        expected: String,
        found: String,
        line: usize,
        column: usize,
    },

    /// Unexpected end of file
    UnexpectedEOF {
        expected: String,
        line: usize,
        column: usize,
    },

    /// General parser error
    Other(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::SyntaxError {
                message,
                line,
                column,
            } => write!(
                f,
                "Syntax error at line {}, column {}: {}",
                line, column, message
            ),

            ParseError::UnknownToken {
                token,
                line,
                column,
            } => write!(
                f,
                "Unknown token '{}' at line {}, column {}",
                token, line, column
            ),

            ParseError::UnexpectedToken {
                expected,
                found,
                line,
                column,
            } => write!(
                f,
                "Unexpected token at line {}, column {}: expected {}, found {}",
                line, column, expected, found
            ),

            ParseError::UnexpectedEOF {
                expected,
                line,
                column,
            } => write!(
                f,
                "Unexpected end of file at line {}, column {}: expected {}",
                line, column, expected
            ),

            ParseError::Other(msg) => write!(f, "Parser error: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

/// Convert line and column to a displayable location string
pub fn location_string(line: usize, column: usize) -> String {
    format!("{}:{}", line, column)
}
