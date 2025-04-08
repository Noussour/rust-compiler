use std::fmt;
use lalrpop_util::ParseError;

#[derive(Debug)]
pub enum SyntaxError {
    InvalidToken {
        position: usize,
        message: String,
    },
    UnexpectedEOF {
        position: usize,
        expected: Vec<String>,
    },
    UnexpectedToken {
        token: String,
        position: (usize, usize),
        expected: Vec<String>,
    },
    ExtraToken {
        token: String,
        position: (usize, usize),
    },
    Custom(String),
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SyntaxError::InvalidToken { position, message } => {
                write!(f, "Invalid token at position {}: {}", position, message)
            },
            SyntaxError::UnexpectedEOF { position, expected } => {
                write!(f, "Unexpected end of file at position {}. Expected: {}", 
                      position, expected.join(", "))
            },
            SyntaxError::UnexpectedToken { token, position, expected } => {
                write!(f, "Unexpected token '{}' at position {:?}. Expected: {}", 
                      token, position, expected.join(", "))
            },
            SyntaxError::ExtraToken { token, position } => {
                write!(f, "Extra token '{}' at position {:?}", token, position)
            },
            SyntaxError::Custom(message) => {
                write!(f, "Parse error: {}", message)
            },
        }
    }
}

impl std::error::Error for SyntaxError {}

// Function to convert LALRPOP errors to your custom error type
pub fn convert_lalrpop_error<T>(error: ParseError<usize, T, String>) -> SyntaxError 
where 
    T: ToString
{
    match error {
        ParseError::InvalidToken { location } => {
            SyntaxError::InvalidToken {
                position: location,
                message: "Invalid token found".to_string(),
            }
        },
        ParseError::UnrecognizedEof { location, expected } => {
            SyntaxError::UnexpectedEOF {
                position: location,
                expected,
            }
        },
        ParseError::UnrecognizedToken { token: (start, token, end), expected } => {
            SyntaxError::UnexpectedToken {
                token: token.to_string(),
                position: (start, end),
                expected,
            }
        },
        ParseError::ExtraToken { token: (start, token, end) } => {
            SyntaxError::ExtraToken {
                token: token.to_string(),
                position: (start, end),
            }
        },
        ParseError::User { error } => {
            SyntaxError::Custom(error)
        }
    }
}