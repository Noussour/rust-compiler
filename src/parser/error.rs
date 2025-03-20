use std::fmt;

/// Parser error types
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Syntax problem at a specific location
    SyntaxError {
        message: String,
        line: usize,
        column: usize,
    },

    /// Got a token we weren't expecting
    UnexpectedToken {
        expected: String,
        found: String,
        line: usize,
        column: usize,
    },

    /// Hit the end of file too soon
    UnexpectedEOF {
        expected: String,
        line: usize,
        column: usize,
    },

    /// Catch-all for other parser problems
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
