pub mod ast;
pub mod error;

#[allow(unused_imports)]
mod parser {
    include!(concat!(env!("OUT_DIR"), "/parser/grammar.rs"));
}

use crate::lexer::lexer_core::TokenWithPosition;
use crate::parser::ast::Program;
use crate::parser::error::ParseError;

pub fn parse(_tokens: Vec<TokenWithPosition>) -> Result<Program, ParseError> {
    // For now, just return a minimal program to verify compilation
    Ok(Program {
        name: "TestProgram".to_string(),
        declarations: vec![],
        statements: vec![],
    })
}