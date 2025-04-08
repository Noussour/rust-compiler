#[allow(unused_imports)]
mod grammar_parser {
    include!(concat!(env!("OUT_DIR"), "/parser/grammar.rs"));
}

use crate::lexer::lexer_core::TokenWithMetaData;
use crate::lexer::token::Token;
use crate::parser::ast::Program;
use crate::parser::error::{
    SyntaxError,
    convert_lalrpop_error,
};


// Add a new function to generate LALRPOP compatible tokens
pub fn tokenize_for_lalrpop(tokens: Vec<TokenWithMetaData>) -> Vec<Result<(usize, Token, usize), String>> {
    tokens
        .into_iter()
        .map(|token| {
            Ok((token.span.start, token.kind, token.span.end))
        })
        .collect()
}


/// Parses tokens into an AST
/// Takes the lexer's tokens and turns them into a proper syntax tree
pub fn parse(tokens: Vec<TokenWithMetaData>) -> Result<Program, SyntaxError> {
    // Convert tokens to LALRPOP format
     let lalrpop_tokens = tokenize_for_lalrpop(tokens);
    
     // Create an iterator that LALRPOP can use
     let token_iter = lalrpop_tokens.into_iter();
     
    match grammar_parser::ProgramParser::new().parse(token_iter) {
        Ok(program) => Ok(program),
        Err(e) => Err(convert_lalrpop_error(e)),
    }
}

/// One-step parsing from source to AST
/// Handles both lexing and parsing in a single function
pub fn parse_source(source: &str) -> Result<Program, SyntaxError> {
    // Tokenize the source code
    let (tokens, _) = crate::lexer::lexer_core::tokenize(source);
    parse(tokens)
}