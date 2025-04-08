use crate::lexer::lexer_core::TokenWithMetaData;
use std::fmt;
use std::error::Error;

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
    pub invalid_token: TokenWithMetaData,
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
            invalid_token: token,
            error_type,
        }
    }
}


// Implement Display for LexicalErrorType
impl fmt::Display for LexicalErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexicalErrorType::UnterminatedString => write!(f, "Unterminated string literal"),
            LexicalErrorType::NonAsciiCharacters => write!(f, "Non-ASCII characters found"),
            LexicalErrorType::IdentifierTooLong => write!(f, "Identifier is too long"),
            LexicalErrorType::ConsecutiveUnderscores => write!(f, "Consecutive underscores are not allowed"),
            LexicalErrorType::TrailingUnderscore => write!(f, "Trailing underscore is not allowed"),
            LexicalErrorType::IdentifierStartsWithNumber => write!(f, "Identifier starts with a number"),
            LexicalErrorType::IntegerOutOfRange => write!(f, "Integer is out of range"),
            LexicalErrorType::InvalidToken => write!(f, "Invalid token"),
        }
    }
}

// Implement Display for LexicalError
impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lexical error: {} in token {:?}",
            self.error_type, self.invalid_token
        )
    }
}

// Implement the Error trait for LexicalError
impl Error for LexicalError {}