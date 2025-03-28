use crate::lexer::token::Token;
use logos::{Lexer as LogosLexer, Logos, SpannedIter};
use std::ops::Range;

// Position in source code with 1-based indexing
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

// Token with its source position information
#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithPosition {
    pub token: Token,
    pub text: String,
    pub position: Position,
    pub span: Range<usize>,
}

// Wrapper around Logos lexer to track position information
pub struct Lexer<'a> {
    logos_lexer: LogosLexer<'a, Token>,
    line_starts: Vec<usize>, // Offsets where each line begins
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let logos_lexer = Token::lexer(source);
        let line_starts = Self::compute_line_starts(source);

        Self {
            logos_lexer,
            line_starts,
        }
    }

    // Records starting positions of all lines in the source
    fn compute_line_starts(source: &str) -> Vec<usize> {
        let mut line_starts = vec![0];

        for (i, c) in source.char_indices() {
            if c == '\n' {
                line_starts.push(i + 1);
            }
        }

        line_starts
    }

    // Converts byte offset to line/column position
    fn offset_to_position(&self, offset: usize) -> Position {
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx - 1,
        };

        let line = line_idx + 1;
        let column = offset - self.line_starts[line_idx] + 1;

        Position { line, column }
    }

    // Enhanced method to handle potential lexer errors
    fn handle_token(&mut self, token_result: Option<Result<Token, ()>>) -> Option<TokenWithPosition> {
        let span = self.logos_lexer.span();
        let position = self.offset_to_position(span.start);
        let token_text = self.logos_lexer.slice().to_string();

        match token_result {
            Some(Ok(token)) => Some(TokenWithPosition {
                token,
                text: token_text,
                position,
                span,
            }),
            Some(Err(_)) => Some(TokenWithPosition {
                token: Token::Error,
                text: token_text,
                position,
                span,
            }),
            None => None,
        }
    }
}

// Iterator implementation to generate tokens with position data
impl Iterator for Lexer<'_> {
    type Item = TokenWithPosition;

    fn next(&mut self) -> Option<Self::Item> {
        let token_result = self.logos_lexer.next();
        
        if token_result.is_none() {
            return None;
        }
        
        let span = self.logos_lexer.span();
        let position = self.offset_to_position(span.start);
        let token_text = self.logos_lexer.slice().to_string();
        
        let token = match token_result.unwrap() {
            Ok(t) => t,
            Err(_) => Token::Error,
        };

        let token_with_pos = TokenWithPosition {
            token,
            text: token_text,
            position,
            span,
        };

        Some(token_with_pos)
    }
}
