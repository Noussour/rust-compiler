pub mod ast;
pub mod error;

#[allow(unused_imports)]
mod grammar_parser {
    include!(concat!(env!("OUT_DIR"), "/parser/grammar.rs"));
}

use crate::lexer::lexer_core::{Lexer, TokenWithPosition};
use crate::lexer::token::Token;
use crate::parser::ast::Program;
use crate::parser::error::ParseError;

/// Parses tokens into an AST
///
/// Takes the lexer's tokens and turns them into a proper syntax tree
/// that represents our MiniSoft program
pub fn parse(tokens: Vec<TokenWithPosition>) -> Result<Program, ParseError> {
    // Need to reformat tokens for LALRPOP
    let token_inputs: Vec<(usize, Token, usize)> = tokens
        .iter()
        .map(|t| (t.span.start, t.token.clone(), t.span.end))
        .collect();

    let lexer = token_inputs.into_iter();

    match grammar_parser::ProgramParser::new().parse(lexer) {
        Ok(program) => Ok(program),
        Err(err) => {
            match err {
                lalrpop_util::ParseError::InvalidToken { location } => {
                    // Grab the token info based on position
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
                    // Get position from the last token
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
                    // Look up the original token info
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
                    // Find matching token in our original list
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

/// One-step parsing from source to AST
/// Handles both lexing and parsing in a single function
pub fn parse_source(source: &str) -> Result<Program, ParseError> {
    let lexer = Lexer::new(source);
    let tokens: Vec<TokenWithPosition> = lexer.collect();
    parse(tokens)
}
