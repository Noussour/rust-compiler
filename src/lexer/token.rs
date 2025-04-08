// Token definitions for the compiler
use logos::Logos;
use std::fmt;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    #[logos(skip r"[ \t\n\f]+")] // Skip all whitespace characters
    // Language keywords
    #[token("MainPrgm")]
    MainPrgm,
    #[token("Var")]
    Var,
    #[token("BeginPg")]
    BeginPg,
    #[token("EndPg")]
    EndPg,
    #[token("let")]
    Let,
    #[token("Int")]
    Int,
    #[token("Float")]
    Float,

    // Control flow
    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("do")]
    Do,
    #[token("from")]
    From,
    #[token("to")]
    To,
    #[token("step")]
    Step,

    // I/O operations
    #[token("input")]
    Input,
    #[token("output")]
    Output,
    #[token("@define")]
    Define,
    #[token("Const")]
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
    #[token("AND")]
    And,
    #[token("OR")]
    Or,
    #[token("!")]
    Not,

    // Literals
    #[regex("(\\([+-][0-9]+\\))|([0-9]+)", parse_int_literal)]
    IntLiteral(i32),

    #[regex("(\\([+-][0-9]+\\.[0-9]+\\))|([0-9]+\\.[0-9]+)", parse_float_literal)]
    FloatLiteral(f32),

    #[regex("\"[^\"]*\"", parse_string_literal)]
    StringLiteral(String),

    #[regex("[a-zA-Z][a-z0-9_]*", parse_identifier)]
    Identifier(String),

    // Ignored tokens
    #[regex("<\\!-([^-\n]|(-[^!\n]))*-\\!>", logos::skip)]
    #[regex("\\{--([^-]|(-[^-]))*--\\}", logos::skip)]
    Comment,


    Error,
}

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


fn parse_int_literal(lex: &mut logos::Lexer<Token>) -> Option<i32> {
    let s = lex.slice();
    let parsed: Option<i32> = if s.starts_with('(') {
        s[1..s.len()-1].parse().ok()
    } else {
        s.parse().ok()
    };
    
    // Only accept values in i16 range
    parsed.filter(|&val| (-32768..=32767).contains(&val))
}

fn parse_float_literal(lex: &mut logos::Lexer<Token>) -> Option<f32> {
    let s = lex.slice();
    if s.starts_with('(') {
        s[1..s.len()-1].parse().ok()
    } else {
        s.parse().ok()
    }
}

fn parse_string_literal(lex: &mut logos::Lexer<Token>) -> Option<String> {
    let s = lex.slice();
    Some(s[1..s.len()-1].to_string())
}

fn parse_identifier(lex: &mut logos::Lexer<Token>) -> Option<String> {
    let s = lex.slice();
    if s.len() <= 14 && !s.contains("__") && !s.ends_with("_") {
        Some(s.to_string())
    } else {
        None
    }
}