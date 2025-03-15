use crate::lexer::token::Token;
use logos::{Lexer as LogosLexer, Logos};
use std::ops::Range;

/// Tracks a token's position in source code
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub line: usize,   // 1-based line number
    pub column: usize, // 1-based column number
}

/// Token with its location information
#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithPosition {
    pub token: Token,       // The token itself
    pub text: String,       // The actual text of the token
    pub position: Position, // Where we found it
    pub span: Range<usize>, // Character range in source
}

/// Lexer breaks source code into tokens
pub struct Lexer<'a> {
    logos_lexer: LogosLexer<'a, Token>, // Using logos for tokenization
    line_starts: Vec<usize>,            // Where each line begins
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given source
    pub fn new(source: &'a str) -> Self {
        let logos_lexer = Token::lexer(source);
        let line_starts = Self::compute_line_starts(source);

        Self {
            logos_lexer,
            line_starts,
        }
    }

    /// Figures out where each line begins
    fn compute_line_starts(source: &str) -> Vec<usize> {
        let mut line_starts = vec![0]; // First line starts at position 0

        // Find start positions by looking for newlines
        for (i, c) in source.char_indices() {
            if c == '\n' {
                line_starts.push(i + 1); // Line starts after newline
            }
        }

        line_starts
    }

    /// Converts byte offset to line/column position
    fn offset_to_position(&self, offset: usize) -> Position {
        // Find the line containing this offset
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx,      // Offset is exactly at line start
            Err(idx) => idx - 1, // Offset is within a line
        };

        let line = line_idx + 1; // 1-based line number
        let column = offset - self.line_starts[line_idx] + 1; // 1-based column

        Position { line, column }
    }
}

// Make lexer work as an iterator for easy token processing
impl Iterator for Lexer<'_> {
    type Item = TokenWithPosition;

    fn next(&mut self) -> Option<Self::Item> {
        // Get next token from logos
        let token = self.logos_lexer.next()?;

        // Track its position
        let span = self.logos_lexer.span();
        let position = self.offset_to_position(span.start);

        // Get the actual text value of the token
        let token_text = self.logos_lexer.slice().to_string();

        // Create the token with position
        let token_with_pos = TokenWithPosition {
            token: token.unwrap_or(Token::Error), // Default to Error for problems
            text: token_text,
            position,
            span,
        };

        Some(token_with_pos)
    }
}
