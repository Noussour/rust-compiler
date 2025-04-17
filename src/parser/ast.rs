use std::ops::Range;

/// Program is the root of the AST

#[derive(Debug, Clone, PartialEq)]
pub struct Located<T> {
    pub node: T,
    pub span: Range<usize>,
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
    String,
}

impl Default for Type {
    fn default() -> Self {
        Type::Int
    }
}

impl Type {
    /// Determines if `self` can be implicitly converted to `target`.
    /// Returns true if the types are compatible for assignment or operation.
    pub fn is_compatible_with(&self, target: &Type) -> bool {
        match (self, target) {
            // Same types are always compatible
            (Type::Int, Type::Int) => true,
            (Type::Float, Type::Float) => true,
            (Type::String, Type::String) => true,
            
            // Int can be converted to Float
            // (Type::Int, Type::Float) => true,
            
            // All other combinations are incompatible
            _ => false,
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::String => write!(f, "String"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    Assignment(Expression, Expression),
    IfThen(Expression, Vec<Statement>),
    IfThenElse(Expression, Vec<Statement>, Vec<Statement>),
    DoWhile(Vec<Statement>, Expression),
    For(Expression, Expression, Expression, Expression, Vec<Statement>),
    Input(Expression),
    Output(Vec<Expression>),
    Scope(Vec<Statement>),
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

impl LiteralKind {
    pub fn literal_kind_to_type(&self) -> Type {
        match self {
            LiteralKind::Int(_) => Type::Int,
            LiteralKind::Float(_) => Type::Float,
            LiteralKind::String(_) => Type::String,
        }
    }
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
    Not,
}

impl LiteralKind {
    /// Get the type of this literal
    pub fn get_type(&self) -> Type {
        match self {
            LiteralKind::Int(_) => Type::Int,
            LiteralKind::Float(_) => Type::Float,
            LiteralKind::String(_) => Type::String,
        }
    }
}



impl Program {
    pub fn pretty_print(&self) {
        println!("Program: {}", self.name);
        println!("├── Declarations:");
        for (i, decl) in self.declarations.iter().enumerate() {
            let is_last = i == self.declarations.len() - 1 && self.statements.is_empty();
            decl.pretty_print("│   ", is_last);
        }
        println!("└── Statements:");
        for (i, stmt) in self.statements.iter().enumerate() {
            let is_last = i == self.statements.len() - 1;
            stmt.pretty_print("    ", is_last);
        }
    }
}

// ...existing code...

impl Declaration {
    fn pretty_print(&self, prefix: &str, is_last: bool) {
        let branch = if is_last { "└──" } else { "├──" };
        let new_prefix = if is_last { format!("{}    ", prefix) } else { format!("{}│   ", prefix) };
        match &self.node {
            DeclarationKind::Variable(names, ty) => {
                println!("{}{} Variable: {:?} : {}", prefix, branch, names, ty);
            }
            DeclarationKind::Array(names, ty, size) => {
                println!("{}{} Array: {:?} : {} [{}]", prefix, branch, names, ty, size);
            }
            DeclarationKind::VariableWithInit(names, ty, expr) => {
                println!("{}{} VariableWithInit: {:?} : {}", prefix, branch, names, ty);
                expr.pretty_print(&new_prefix, true);
            }
            DeclarationKind::ArrayWithInit(names, ty, size, exprs) => {
                println!("{}{} ArrayWithInit: {:?} : {} [{}]", prefix, branch, names, ty, size);
                for (i, expr) in exprs.iter().enumerate() {
                    expr.pretty_print(&new_prefix, i == exprs.len() - 1);
                }
            }
            DeclarationKind::Constant(name, ty, lit) => {
                println!("{}{} Constant: {} : {} = {:?}", prefix, branch, name, ty, lit.node);
            }
        }
    }
}

impl Statement {
    fn pretty_print(&self, prefix: &str, is_last: bool) {
        let branch = if is_last { "└──" } else { "├──" };
        let new_prefix = if is_last { format!("{}    ", prefix) } else { format!("{}│   ", prefix) };
        match &self.node {
            StatementKind::Assignment(lhs, rhs) => {
                println!("{}{} Assignment:", prefix, branch);
                lhs.pretty_print(&new_prefix, false);
                rhs.pretty_print(&new_prefix, true);
            }
            StatementKind::IfThen(cond, stmts) => {
                println!("{}{} IfThen:", prefix, branch);
                cond.pretty_print(&new_prefix, false);
                for (i, stmt) in stmts.iter().enumerate() {
                    stmt.pretty_print(&new_prefix, i == stmts.len() - 1);
                }
            }
            StatementKind::IfThenElse(cond, then_stmts, else_stmts) => {
                println!("{}{} IfThenElse:", prefix, branch);
                cond.pretty_print(&new_prefix, false);

                // Then branch
                let then_prefix = format!("{}{}", new_prefix, "├── Then:");
                println!("{}", then_prefix);
                let then_child_prefix = if else_stmts.is_empty() && then_stmts.len() > 0 {
                    format!("{}    ", new_prefix)
                } else {
                    format!("{}│   ", new_prefix)
                };
                for (i, stmt) in then_stmts.iter().enumerate() {
                    stmt.pretty_print(&then_child_prefix, i == then_stmts.len() - 1);
                }

                // Else branch
                let else_prefix = format!("{}{}", new_prefix, "└── Else:");
                println!("{}", else_prefix);
                let else_child_prefix = format!("{}    ", new_prefix);
                for (i, stmt) in else_stmts.iter().enumerate() {
                    stmt.pretty_print(&else_child_prefix, i == else_stmts.len() - 1);
                }
            }
            StatementKind::DoWhile(stmts, cond) => {
                println!("{}{} DoWhile:", prefix, branch);
                for (_i, stmt) in stmts.iter().enumerate() {
                    stmt.pretty_print(&new_prefix, false);
                }
                cond.pretty_print(&new_prefix, true);
            }
            StatementKind::For(init, cond, step, end, stmts) => {
                println!("{}{} For:", prefix, branch);
                init.pretty_print(&new_prefix, false);
                cond.pretty_print(&new_prefix, false);
                step.pretty_print(&new_prefix, false);
                end.pretty_print(&new_prefix, false);
                for (i, stmt) in stmts.iter().enumerate() {
                    stmt.pretty_print(&new_prefix, i == stmts.len() - 1);
                }
            }
            StatementKind::Input(expr) => {
                println!("{}{} Input:", prefix, branch);
                expr.pretty_print(&new_prefix, true);
            }
            StatementKind::Output(exprs) => {
                println!("{}{} Output:", prefix, branch);
                for (i, expr) in exprs.iter().enumerate() {
                    expr.pretty_print(&new_prefix, i == exprs.len() - 1);
                }
            }
            StatementKind::Scope(stmts) => {
                println!("{}{} Scope:", prefix, branch);
                for (i, stmt) in stmts.iter().enumerate() {
                    stmt.pretty_print(&new_prefix, i == stmts.len() - 1);
                }
            }
            StatementKind::Empty => {
                println!("{}{} Empty", prefix, branch);
            }
        }
    }
}

impl Expression {
    fn pretty_print(&self, prefix: &str, is_last: bool) {
        let branch = if is_last { "└──" } else { "├──" };
        let new_prefix = if is_last { format!("{}    ", prefix) } else { format!("{}│   ", prefix) };
        match &self.node {
            ExpressionKind::Identifier(name) => {
                println!("{}{} Identifier: {}", prefix, branch, name);
            }
            ExpressionKind::ArrayAccess(name, idx) => {
                println!("{}{} ArrayAccess: {}", prefix, branch, name);
                idx.pretty_print(&new_prefix, true);
            }
            ExpressionKind::Literal(lit) => {
                println!("{}{} Literal: {:?}", prefix, branch, lit.node);
            }
            ExpressionKind::BinaryOp(lhs, op, rhs) => {
                println!("{}{} BinaryOp: {:?}", prefix, branch, op);
                lhs.pretty_print(&new_prefix, false);
                rhs.pretty_print(&new_prefix, true);
            }
            ExpressionKind::UnaryOp(op, expr) => {
                println!("{}{} UnaryOp: {:?}", prefix, branch, op);
                expr.pretty_print(&new_prefix, true);
            }
        }
    }
}