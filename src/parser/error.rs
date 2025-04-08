use crate::lexer::token::Token;

// Parser error types
#[derive(Debug, PartialEq, Clone)]
pub enum SyntaxErrorType {
    // Program structure errors
    InvalidProgramStructure(String),
    MissingMainPrgm,
    MissingBeginPgOrEndPg,
    MismatchedBraces,

    // Declaration errors
    InvalidDeclarationFormat(String),
    MissingColonInDeclaration,
    MissingSemicolonAfterDeclaration,
    InvalidArraySyntax,
    NonIntegerArraySize,
    InvalidConstantSyntax,

    // Identifier errors
    InvalidIdentifierFormat(String),
    ReservedKeywordAsIdentifier(String),

    // Type errors
    InvalidType(String),
    MissingParenthesesForSignedLiteral,

    // Instruction errors
    MissingAssignmentOperator,
    MissingSemicolonAfterStatement,
    InvalidConditionalStructure,
    InvalidLoopStructure,
    MissingStepInForLoop,
    MissingParenthesesInControlStructure,

    // Operator/Expression errors
    InvalidOperator(String),
    MismatchedParentheses,
    InvalidExpressionStructure,

    // Input/Output errors
    InvalidInputOutputSyntax,
    UnquotedStringInOutput,

    // Comment errors
    UnterminatedSingleLineComment,
    UnterminatedMultiLineComment,

    // General syntax errors
    UnexpectedToken(String, String), // (expected, found)
    GenericError(String),
}


#[derive(Debug, Clone)]
pub struct SyntaxError {
    pub error_type: SyntaxErrorType,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub line_source: Option<String>, // Optional source line for better error reporting
}


impl From<lalrpop_util::ParseError<usize, Token, String>> for SyntaxError {
    fn from(err: lalrpop_util::ParseError<usize, Token, String>) -> Self {
        match err {
            lalrpop_util::ParseError::InvalidToken { location } => SyntaxError {
                error_type: SyntaxErrorType::GenericError(format!("Invalid token at position {}", location)),
                line: None,
                column: None,
            },
            lalrpop_util::ParseError::UnrecognizedEof { location, expected } => SyntaxError {
                error_type: SyntaxErrorType::GenericError(format!("Unexpected end of file, expected: {}", expected.join(", "))),
                line: None,
                column: None,
            },
            lalrpop_util::ParseError::UnrecognizedToken { token: (start, token, end), expected } => SyntaxError {
                error_type: SyntaxErrorType::UnexpectedToken(expected.join(", "), format!("{:?}", token)),
                line: None,
                column: None,
            },
            lalrpop_util::ParseError::ExtraToken { token: (start, token, end) } => SyntaxError {
                error_type: SyntaxErrorType::GenericError(format!("Extra token: {:?}", token)),
                line: None,
                column: None,
            },
            lalrpop_util::ParseError::User { error } => SyntaxError {
                error_type: SyntaxErrorType::GenericError(error),
                line: None,
                column: None,
            },
        }
    }
}