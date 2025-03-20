// Abstract Syntax Tree for MiniSoft language

/// Program is the root of the AST
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub name: String,
    pub declarations: Vec<Declaration>,
    pub statements: Vec<Statement>,
}

/// Declaration types
#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Variable(Vec<String>, Type),     // names, type
    Array(Vec<String>, Type, usize), // names, type, size
    Constant(String, Type, Literal), // name, type, value
}

/// Data types supported by MiniSoft
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

/// Statement types
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment(Expression, Expression),             // target, value
    If(Expression, Vec<Statement>, Vec<Statement>), // condition, then_block, else_block
    DoWhile(Vec<Statement>, Expression),            // body, condition
    For(String, Expression, Expression, Expression, Vec<Statement>), // var, from, to, step, body
    Input(String),                                  // variable name
    Output(Vec<Expression>),                        // expressions to output
    Empty,
}

/// Expression types
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(String),
    ArrayAccess(String, Box<Expression>), // name, index
    Literal(Literal),
    BinaryOp(Box<Expression>, Operator, Box<Expression>),
    UnaryOp(UnaryOperator, Box<Expression>),
}

/// Literal values
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

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Negate,
    Not,
}

impl Literal {
    /// Get the type of this literal
    pub fn get_type(&self) -> Type {
        match self {
            Literal::Int(_) => Type::Int,
            Literal::Float(_) => Type::Float,
            Literal::String(_) => panic!("String literals don't have a MiniSoft type"),
        }
    }
}
