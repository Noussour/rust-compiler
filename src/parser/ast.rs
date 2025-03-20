// Minimal AST for verification

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub name: String,
    pub declarations: Vec<Declaration>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    VariableDecl(String, String), // name, type
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Empty,
}