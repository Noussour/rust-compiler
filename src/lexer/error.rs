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



// Implement Display for LexicalError
impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lexical error: {:?} at line {}, column {}: {}",
            match self.error_type {
                LexicalErrorType::UnterminatedString => "Unterminated string",
                LexicalErrorType::NonAsciiCharacters => "Non-ASCII characters",
                LexicalErrorType::IdentifierTooLong => "Identifier too long",
                LexicalErrorType::ConsecutiveUnderscores => "Consecutive underscores",
                LexicalErrorType::TrailingUnderscore => "Trailing underscore",
                LexicalErrorType::IdentifierStartsWithNumber => "Identifier starts with number",
                LexicalErrorType::IntegerOutOfRange => "Integer out of range",
                LexicalErrorType::InvalidToken => "Invalid token",
            },
            self.invalid_token.line,
            self.invalid_token.column,
            self.invalid_token.value
        )
    }
}

// Implement the Error trait for LexicalError
impl Error for LexicalError {}