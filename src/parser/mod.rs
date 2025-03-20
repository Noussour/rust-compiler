pub mod ast;
pub mod error;

#[allow(unused_imports)]
mod parser {
    include!(concat!(env!("OUT_DIR"), "/parser/grammar.rs"));
}

use crate::lexer::lexer_core::{Lexer, TokenWithPosition};
use crate::lexer::token::Token;
use crate::parser::ast::Program;
use crate::parser::error::ParseError;

/// Parse a sequence of tokens into an AST
///
/// This function takes a vector of tokens produced by the lexer
/// and converts them into an Abstract Syntax Tree representation
/// of the MiniSoft program.
pub fn parse(tokens: Vec<TokenWithPosition>) -> Result<Program, ParseError> {
    // Convert tokens to the format expected by LALRPOP
    let token_inputs: Vec<(usize, Token, usize)> = tokens
        .iter()
        .map(|t| (t.span.start, t.token.clone(), t.span.end))
        .collect();

    // Create a LALRPOP lexer
    let lexer = token_inputs.into_iter();

    // Use the LALRPOP-generated parser
    match parser::ProgramParser::new().parse(lexer) {
        Ok(program) => Ok(program),
        Err(err) => {
            match err {
                lalrpop_util::ParseError::InvalidToken { location } => {
                    // Find the token position from our original tokens
                    if let Some(token) = tokens.iter().find(|t| t.span.start == location) {
                        Err(ParseError::SyntaxError {
                            message: "Invalid token".to_string(),
                            line: token.position.line,
                            column: token.position.column,
                        })
                    } else {
                        Err(ParseError::Other(
                            "Invalid token at unknown location".to_string(),
                        ))
                    }
                }
                lalrpop_util::ParseError::UnrecognizedEof {
                    location: _,
                    expected,
                } => {
                    // Find the last token position
                    if let Some(last_token) = tokens.last() {
                        Err(ParseError::UnexpectedEOF {
                            expected: expected.join(", "),
                            line: last_token.position.line,
                            column: last_token.position.column + last_token.text.len(),
                        })
                    } else {
                        Err(ParseError::Other("Unexpected end of file".to_string()))
                    }
                }
                lalrpop_util::ParseError::UnrecognizedToken {
                    token: (start, token, _end),
                    expected,
                } => {
                    // Find the token position from our original tokens
                    if let Some(token_info) = tokens.iter().find(|t| t.span.start == start) {
                        Err(ParseError::UnexpectedToken {
                            expected: expected.join(", "),
                            found: format!("{}", token),
                            line: token_info.position.line,
                            column: token_info.position.column,
                        })
                    } else {
                        Err(ParseError::Other(format!("Unexpected token: {}", token)))
                    }
                }
                lalrpop_util::ParseError::ExtraToken {
                    token: (start, token, _),
                } => {
                    // Find the token position from our original tokens
                    if let Some(token_info) = tokens.iter().find(|t| t.span.start == start) {
                        Err(ParseError::SyntaxError {
                            message: format!("Extra token: {}", token),
                            line: token_info.position.line,
                            column: token_info.position.column,
                        })
                    } else {
                        Err(ParseError::Other(format!("Extra token: {}", token)))
                    }
                }
                lalrpop_util::ParseError::User { error } => {
                    Err(ParseError::Other(format!("User error: {}", error)))
                }
            }
        }
    }
}

/// Parse source code directly to AST

/// Convenience function that runs both lexing and parsing
pub fn parse_source(source: &str) -> Result<Program, ParseError> {
    let lexer = Lexer::new(source);
    let tokens: Vec<TokenWithPosition> = lexer.collect();
    parse(tokens)
}
