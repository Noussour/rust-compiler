// Defines all tokens recognized by our compiler
// Each token represents the smallest meaningful unit in the language syntax

use logos::Logos; // For lexical analysis
use std::fmt; // For token display formatting

// All possible tokens in our language
#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // Language keywords
    // (priority=1 ensures keywords take precedence over identifiers)
    #[token("MainPrgm", priority = 1)]
    MainPrgm,
    #[token("Var", priority = 1)]
    Var,
    #[token("BeginPg", priority = 1)]
    BeginPg,
    #[token("EndPg", priority = 1)]
    EndPg,
    #[token("let", priority = 1)]
    Let,
    #[token("Int", priority = 1)]
    Int,
    #[token("Float", priority = 1)]
    Float,
    // Control flow
    #[token("if", priority = 1)]
    If,
    #[token("then", priority = 1)]
    Then,
    #[token("else", priority = 1)]
    Else,
    #[token("while", priority = 1)]
    While,
    #[token("for", priority = 1)]
    For,
    #[token("do", priority = 1)]
    Do,
    #[token("from", priority = 1)]
    From,
    #[token("to", priority = 1)]
    To,
    #[token("step", priority = 1)]
    Step,
    // I/O operations
    #[token("input", priority = 1)]
    Input,
    #[token("output", priority = 1)]
    Output,
    #[token("@define", priority = 1)]
    Define,
    #[token("Const", priority = 1)]
    Const,

    // Punctuation and symbols
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,

    // Assignment
    #[token(":=")]
    Assign,
    #[token("=")]
    Equals,

    // Arithmetic operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Multiply,
    #[token("/")]
    Divide,

    // Comparison operators
    #[token(">")]
    GreaterThan,
    #[token("<")]
    LessThan,
    #[token(">=")]
    GreaterEqual,
    #[token("<=")]
    LessEqual,
    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,

    // Logic operators
    #[token("AND", priority = 1)]
    And,
    #[token("OR", priority = 1)]
    Or,
    #[token("!")]
    Not,

    // Literal values
    // Integer literals
    #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
    IntLiteral(i32),

    // Float literals
    #[regex("[0-9]+\\.[0-9]+", |lex| lex.slice().parse().ok())]
    FloatLiteral(f32),

    // String literals
    #[regex("\"[^\"]*\"", |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())  // Strip quotes
    })]
    StringLiteral(String),

    // Identifiers (variable and function names)
    #[regex("[a-zA-Z][a-zA-Z0-9_]*[a-zA-Z0-9]", |lex| {
        let s = lex.slice();
        // Max 14 chars, no consecutive underscores
        if s.len() <= 14 && !s.contains("__") {
            Some(s.to_string())
        } else {
            None
        }
    }, priority = 0)] // Lower priority than keywords
    Identifier(String),

    // Ignored tokens
    // Comments - both styles get skipped
    #[regex("<\\s*!-([^-]|(-[^!]))*-\\s*!>", logos::skip)]
    #[regex("\\{--[^-]*--\\}", logos::skip)]
    Comment,

    // Whitespace gets skipped too
    #[regex(r"[ \t\n\r]+", logos::skip)]
    Whitespace,

    // For invalid input
    Error,
}

// Custom display formatting for tokens
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Identifier(s) => write!(f, "Identifier({})", s),
            Token::IntLiteral(n) => write!(f, "IntLiteral({})", n),
            Token::FloatLiteral(x) => write!(f, "FloatLiteral({})", x),
            Token::StringLiteral(s) => write!(f, "StringLiteral(\"{}\")", s),
            _ => write!(f, "{:?}", self),
        }
    }
}
