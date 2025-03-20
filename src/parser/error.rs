use std::fmt;

#[derive(Debug, Clone)]
pub enum ParseError {
    SyntaxError(String),
    Other(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
            ParseError::Other(msg) => write!(f, "Parser error: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}