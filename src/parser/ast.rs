// AST for MiniSoft language

/// Program is the root of the AST
/// Represents a position in source code
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// Wrapper for any AST node that includes position information
#[derive(Debug, Clone, PartialEq)]
pub struct Located<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Located<T> {
    pub fn into_inner(self) -> T {
        self.node
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub name: String,
    pub declarations: Vec<Declaration>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeclarationKind {
    Variable(Vec<String>, Type),
    Array(Vec<String>, Type, usize),
    VariableWithInit(Vec<String>, Type, Expression),
    ArrayWithInit(Vec<String>, Type, usize, Vec<Expression>),
    Constant(String, Type, Literal),
}

pub type Declaration = Located<DeclarationKind>;

/// Data types in MiniSoft
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
}

impl Default for Type {
    fn default() -> Self {
        Type::Float
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::Bool => write!(f, "Bool"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    Assignment(Expression, Expression),
    IfThen(Expression, Vec<Statement>),
    IfThenElse(Expression, Vec<Statement>, Vec<Statement>),
    DoWhile(Vec<Statement>, Expression),
    For(String, Expression, Expression, Expression, Vec<Statement>),
    Input(Expression),
    Output(Vec<Expression>),
    Block(Vec<Statement>),
    Empty,
}

pub type Statement = Located<StatementKind>;

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    Identifier(String),
    ArrayAccess(String, Box<Expression>),
    Literal(Literal),
    BinaryOp(Box<Expression>, Operator, Box<Expression>),
    UnaryOp(UnaryOperator, Box<Expression>),
}

pub type Expression = Located<ExpressionKind>;

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralKind {
    Int(i32),
    Float(f32),
    String(String),
}

pub type Literal = Located<LiteralKind>;

/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,

    // Comparison
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    Equal,
    NotEqual,

    // Logical
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    // Negate,
    Not,
}

impl LiteralKind {
    /// Get the type of this literal
    pub fn get_type(&self) -> Type {
        match self {
            LiteralKind::Int(_) => Type::Int,
            LiteralKind::Float(_) => Type::Float,
            LiteralKind::String(_) => panic!("String literals don't have a MiniSoft type"),
        }
    }
}
