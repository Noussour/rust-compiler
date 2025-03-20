// AST for MiniSoft language

/// Program is the root of the AST
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub name: String,
    pub declarations: Vec<Declaration>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Variable(Vec<String>, Type),
    Array(Vec<String>, Type, usize),
    Constant(String, Type, Literal),
}

/// Data types in MiniSoft
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment(Expression, Expression),
    If(Expression, Vec<Statement>, Vec<Statement>),
    DoWhile(Vec<Statement>, Expression),
    For(String, Expression, Expression, Expression, Vec<Statement>),
    Input(String),
    Output(Vec<Expression>),
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(String),
    ArrayAccess(String, Box<Expression>),
    Literal(Literal),
    BinaryOp(Box<Expression>, Operator, Box<Expression>),
    UnaryOp(UnaryOperator, Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i32),
    Float(f32),
    String(String),
}

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
    Negate,
    Not,
}

impl Literal {
    /// Get the type of this literal
    #[allow(dead_code)]
    pub fn get_type(&self) -> Type {
        match self {
            Literal::Int(_) => Type::Int,
            Literal::Float(_) => Type::Float,
            Literal::String(_) => panic!("String literals don't have a MiniSoft type"),
        }
    }
}
