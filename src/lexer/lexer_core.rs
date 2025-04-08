use logos::{Lexer, Logos};
use crate::lexer::token::Token;
use crate::lexer::error::LexicalErrorType;
use std::ops::Range;

// Token with its source position information
#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithMetaData {
    pub kind: Token,
    pub value: String,
    pub line: usize,
    pub column: usize,
    pub span: Range<usize>,
}

pub struct LexicalError {
    pub invalid_token: TokenWithMetaData,
    pub error_type: LexicalErrorType,
}

fn get_position<'a>(lexer: &Lexer<'a, Token>, byte_offset: usize) -> (usize, usize) {
    let line = lexer.extras.line_number;
    let col = byte_offset - lexer.extras.line_start;
    (line, col)
}

// Logic for detecting error type
fn detect_error_type(text: &str) -> Option<LexicalErrorType> {
    if text.starts_with('"') && !text.ends_with('"') {
        return Some(LexicalErrorType::UnterminatedString);
    }
    if text.contains(|c: char| !c.is_ascii()) {
        return Some(LexicalErrorType::NonAsciiCharacters);
    }
    if text.len() > 14 && text.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Some(LexicalErrorType::IdentifierTooLong);
    }
    if text.contains("__") {
        return Some(LexicalErrorType::ConsecutiveUnderscores);
    }
    if text.ends_with("_") {
        return Some(LexicalErrorType::TrailingUnderscore);
    }
    if text.starts_with(|c: char| c.is_numeric()) && text.contains(|c: char| c.is_alphabetic()) {
        return Some(LexicalErrorType::IdentifierStartsWithNumber);
    }
    if text.chars().all(|c: char| c.is_numeric() || c == '+' || c == '-') {
        if let Ok(num) = text.parse::<i32>() {
            if !(-32768..=32767).contains(&num) {
                return Some(LexicalErrorType::IntegerOutOfRange);
            }
        }
    }
    None
}

pub fn tokenize(source: &str) -> (Vec<TokenWithMetaData>, Vec<LexicalError>) {
    let mut lexer = Token::lexer(source);
    let mut valid_tokens = Vec::new();
    let mut errors = Vec::new();

    while let Some(valid_result) = lexer.next() {
        let span = lexer.span();
        let value = lexer.slice().to_string();
        let (line, column) = get_position(&lexer, span.start);

        match valid_result {
            Ok(kind) => {
                valid_tokens.push(TokenWithMetaData {
                    kind,
                    value,
                    line,
                    column,
                    span,
                });
            },
            Err(_) => {
                let error_type = detect_error_type(&value);
                errors.push(LexicalError {
                    invalid_token: TokenWithMetaData {
                        kind: Token::Error,
                        value: value.clone(),
                        line,
                        column,
                        span,
                    },
                    error_type: error_type.unwrap_or(LexicalErrorType::InvalidToken), // Default to InvalidToken if no specific error
                });
            },
        };
    }

    (valid_tokens, errors)
}
