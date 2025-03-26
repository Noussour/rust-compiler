use crate::parser::ast::{
    Declaration, Expression, Literal, Operator, Program, Statement, Type, UnaryOperator,
};
use crate::semantics::error::SemanticError;
use crate::semantics::symbol_table::{Symbol, SymbolKind, SymbolTable};
use std::collections::HashMap;

// Add a struct for literals with position info
#[derive(Debug, Clone, PartialEq)]
struct ExpressionPosition {
    pub line: usize,
    pub column: usize,
}

/// The semantic analyzer for MiniSoft
#[derive(Default)]
pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    errors: Vec<SemanticError>,
    // Store position information from the lexer
    positions: HashMap<String, (usize, usize)>,
    // Track current position for expressions
    current_expr_pos: Option<ExpressionPosition>,
}

impl SemanticAnalyzer {
    /// Creates a new semantic analyzer with position information
    pub fn new_with_positions(positions: HashMap<String, (usize, usize)>) -> Self {
        SemanticAnalyzer {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            positions,
            current_expr_pos: None,
        }
    }

    /// Gets position information for an identifier
    fn get_position(&self, name: &str) -> (usize, usize) {
        // First try identifier's known position
        if let Some(pos) = self.positions.get(name) {
            return *pos;
        }

        // Fall back to current expression position
        if let Some(pos) = &self.current_expr_pos {
            return (pos.line, pos.column);
        }

        // Default position
        (1, 1)
    }

    /// Set the current expression position context
    fn set_current_expr_pos(&mut self, line: usize, column: usize) {
        self.current_expr_pos = Some(ExpressionPosition { line, column });
    }

    /// Clear the current expression position context
    fn clear_current_expr_pos(&mut self) {
        self.current_expr_pos = None;
    }

    /// Analyzes a program for semantic errors
    pub fn analyze(&mut self, program: &Program) {
        // Set position for program name
        if let Some(pos) = self.positions.get(&program.name) {
            self.set_current_expr_pos(pos.0, pos.1);
        }

        // Process all declarations
        for declaration in &program.declarations {
            self.analyze_declaration(declaration);
        }

        // Process all statements
        for statement in &program.statements {
            self.analyze_statement(statement);
        }

        self.clear_current_expr_pos();
    }

    /// Gets all semantic errors found during analysis
    pub fn get_errors(&self) -> &[SemanticError] {
        &self.errors
    }

    /// Gets the completed symbol table
    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    /// Analyzes a declaration
    fn analyze_declaration(&mut self, declaration: &Declaration) {
        match declaration {
            Declaration::Variable(names, typ) => {
                for name in names {
                    let (line, column) = self.get_position(name);

                    // Check for duplicate declaration
                    if self.symbol_table.contains(name) {
                        let existing = self.symbol_table.get(name).unwrap();
                        self.errors.push(SemanticError::DuplicateDeclaration {
                            name: name.clone(),
                            line,
                            column,
                            original_line: existing.line,
                            original_column: existing.column,
                        });
                        continue;
                    }

                    // Add to symbol table
                    let symbol = Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Variable,
                        symbol_type: typ.clone(),
                        value: None,
                        line,
                        column,
                    };
                    self.symbol_table.add_symbol(symbol);
                }
            }
            Declaration::Array(names, typ, size) => {
                for name in names {
                    let (line, column) = self.get_position(name);

                    // Check for duplicate declaration
                    if self.symbol_table.contains(name) {
                        let existing = self.symbol_table.get(name).unwrap();
                        self.errors.push(SemanticError::DuplicateDeclaration {
                            name: name.clone(),
                            line,
                            column,
                            original_line: existing.line,
                            original_column: existing.column,
                        });
                        continue;
                    }

                    // Add to symbol table
                    let symbol = Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Array(*size),
                        symbol_type: typ.clone(),
                        value: None,
                        line,
                        column,
                    };
                    self.symbol_table.add_symbol(symbol);
                }
            }
            Declaration::Constant(name, typ, value) => {
                let (line, column) = self.get_position(name);

                // Check for duplicate declaration
                if self.symbol_table.contains(name) {
                    let existing = self.symbol_table.get(name).unwrap();
                    self.errors.push(SemanticError::DuplicateDeclaration {
                        name: name.clone(),
                        line,
                        column,
                        original_line: existing.line,
                        original_column: existing.column,
                    });
                    return;
                }

                // Check value type matches declaration type
                let value_type = match value {
                    Literal::Int(_) => Type::Int,
                    Literal::Float(_) => Type::Float,
                    Literal::String(_) => {
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: format!("{}", typ),
                            found: "String".to_string(),
                            line,
                            column,
                        });
                        return;
                    }
                };

                if value_type != *typ {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: format!("{}", typ),
                        found: format!("{}", value_type),
                        line,
                        column,
                    });
                    return;
                }

                // Add to symbol table
                let symbol = Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Constant,
                    symbol_type: typ.clone(),
                    value: Some(value.clone()),
                    line,
                    column,
                };
                self.symbol_table.add_symbol(symbol);
            }
            Declaration::VariableWithInit(names, typ, expr) => {
                // First, check the expression
                let expr_type = self.analyze_expression(expr);
                if let Some(expr_type) = expr_type {
                    if expr_type != *typ {
                        let (line, column) = self.get_position(&names[0]);
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: format!("{}", typ),
                            found: format!("{}", expr_type),
                            line,
                            column,
                        });
                    }
                }

                // Now add the variables to the symbol table
                for name in names {
                    let (line, column) = self.get_position(name);

                    // Check for duplicate declaration
                    if self.symbol_table.contains(name) {
                        let existing = self.symbol_table.get(name).unwrap();
                        self.errors.push(SemanticError::DuplicateDeclaration {
                            name: name.clone(),
                            line,
                            column,
                            original_line: existing.line,
                            original_column: existing.column,
                        });
                        continue;
                    }

                    // Add to symbol table
                    let symbol = Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Variable,
                        symbol_type: typ.clone(),
                        value: None, // Value is set at runtime
                        line,
                        column,
                    };
                    self.symbol_table.add_symbol(symbol);
                }
            }
            Declaration::ArrayWithInit(names, typ, size, values) => {
                // Check if number of initializer values matches array size
                if values.len() > *size {
                    self.errors.push(SemanticError::Other(format!(
                        "Too many initializer values for array. Expected {}, got {}",
                        size,
                        values.len()
                    )));
                }

                // Check types of all initializers
                for value in values {
                    let value_type = self.analyze_expression(value);
                    if let Some(value_type) = value_type {
                        if value_type != *typ {
                            let (line, column) = self.get_position(&names[0]);
                            self.errors.push(SemanticError::TypeMismatch {
                                expected: format!("{}", typ),
                                found: format!("{}", value_type),
                                line,
                                column,
                            });
                        }
                    }
                }

                // Now add the arrays to the symbol table
                for name in names {
                    let (line, column) = self.get_position(name);

                    // Check for duplicate declaration
                    if self.symbol_table.contains(name) {
                        let existing = self.symbol_table.get(name).unwrap();
                        self.errors.push(SemanticError::DuplicateDeclaration {
                            name: name.clone(),
                            line,
                            column,
                            original_line: existing.line,
                            original_column: existing.column,
                        });
                        continue;
                    }

                    // Add to symbol table
                    let symbol = Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Array(*size),
                        symbol_type: typ.clone(),
                        value: None, // Array values are set at runtime
                        line,
                        column,
                    };
                    self.symbol_table.add_symbol(symbol);
                }
            }
        }
    }

    /// Analyzes a statement
    fn analyze_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Assignment(lhs, rhs) => {
                // First analyze the left-hand side (target)
                match lhs {
                    Expression::Identifier(name) => {
                        // Check if the identifier exists
                        if !self.symbol_table.contains(name) {
                            let (line, column) = self.get_position(name);
                            self.errors.push(SemanticError::UndeclaredIdentifier {
                                name: name.clone(),
                                line,
                                column,
                            });
                            return;
                        }

                        // Check if it's a constant
                        let symbol = self.symbol_table.get(name).unwrap();
                        if let SymbolKind::Constant = symbol.kind {
                            let (line, column) = self.get_position(name);
                            self.errors.push(SemanticError::ConstantModification {
                                name: name.clone(),
                                line,
                                column,
                            });
                            return;
                        }

                        // Get the type of the identifier
                        let lhs_type = symbol.symbol_type.clone();

                        // Check if the right-hand side expression matches the type
                        let rhs_type = self.analyze_expression(rhs);
                        if let Some(rhs_type) = rhs_type {
                            if rhs_type != lhs_type {
                                let (line, column) = self.get_position(name);
                                self.errors.push(SemanticError::TypeMismatch {
                                    expected: format!("{}", lhs_type),
                                    found: format!("{}", rhs_type),
                                    line,
                                    column,
                                });
                            }
                        }
                    }
                    Expression::ArrayAccess(name, index_expr) => {
                        // Check if the array exists
                        if !self.symbol_table.contains(name) {
                            let (line, column) = self.get_position(name);
                            self.errors.push(SemanticError::UndeclaredIdentifier {
                                name: name.clone(),
                                line,
                                column,
                            });
                            return;
                        }

                        // Check if it's actually an array
                        let symbol = self.symbol_table.get(name).unwrap();
                        match &symbol.kind {
                            SymbolKind::Array(size) => {
                                // Save type and size information before releasing the borrow
                                let array_size = *size;
                                let element_type = symbol.symbol_type.clone();

                                // Check if the index is a constant and within bounds
                                if let Expression::Literal(Literal::Int(idx)) = &**index_expr {
                                    if *idx < 0 || *idx as usize >= array_size {
                                        let (line, column) = self.get_position(name);
                                        self.errors.push(SemanticError::ArrayIndexOutOfBounds {
                                            name: name.clone(),
                                            index: *idx as usize,
                                            size: array_size,
                                            line,
                                            column,
                                        });
                                        return;
                                    }
                                }

                                // Now we're done with the symbol borrow, check index expression type
                                let idx_type = self.analyze_expression(index_expr);
                                if let Some(idx_type) = idx_type {
                                    if idx_type != Type::Int {
                                        let (line, column) = self.get_position(name);
                                        self.errors.push(SemanticError::TypeMismatch {
                                            expected: "Int".to_string(),
                                            found: format!("{}", idx_type),
                                            line,
                                            column,
                                        });
                                        return;
                                    }
                                }

                                // Use our saved element_type instead of accessing symbol.symbol_type
                                let rhs_type = self.analyze_expression(rhs);
                                if let Some(rhs_type) = rhs_type {
                                    if rhs_type != element_type {
                                        let (line, column) = self.get_position(name);
                                        self.errors.push(SemanticError::TypeMismatch {
                                            expected: format!("{}", element_type),
                                            found: format!("{}", rhs_type),
                                            line,
                                            column,
                                        });
                                    }
                                }
                            }
                            _ => {
                                self.errors.push(SemanticError::Other(format!(
                                    "Cannot index non-array variable '{}'",
                                    name
                                )));
                            }
                        }
                    }
                    _ => {
                        self.errors.push(SemanticError::Other(
                            "Invalid assignment target".to_string(),
                        ));
                    }
                }
            }
            Statement::IfThen(condition, then_block) => {
                // Check that the condition expression is a boolean expression
                self.analyze_expression(condition);

                // Analyze the statements in the then block
                for stmt in then_block {
                    self.analyze_statement(stmt);
                }
            }
            Statement::IfThenElse(condition, then_block, else_block) => {
                // Check that the condition expression is a boolean expression
                self.analyze_expression(condition);

                // Analyze the statements in the then block
                for stmt in then_block {
                    self.analyze_statement(stmt);
                }

                // Analyze the statements in the else block
                for stmt in else_block {
                    self.analyze_statement(stmt);
                }
            }
            Statement::DoWhile(body, condition) => {
                // Check that the condition expression is a boolean expression
                self.analyze_expression(condition);

                // Analyze the statements in the loop body
                for stmt in body {
                    self.analyze_statement(stmt);
                }
            }
            Statement::For(var, from, to, step, body) => {
                // Check if the loop variable exists
                if !self.symbol_table.contains(var) {
                    let (line, column) = self.get_position(var);
                    self.errors.push(SemanticError::UndeclaredIdentifier {
                        name: var.clone(),
                        line,
                        column,
                    });
                } else {
                    // Check if the loop variable is an integer
                    let symbol = self.symbol_table.get(var).unwrap();
                    if symbol.symbol_type != Type::Int {
                        let (line, column) = self.get_position(var);
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: "Int".to_string(),
                            found: format!("{}", symbol.symbol_type),
                            line,
                            column,
                        });
                    }
                }

                // Check that from, to, and step are all numeric expressions
                let from_type = self.analyze_expression(from);
                if let Some(from_type) = from_type {
                    if from_type != Type::Int {
                        let (line, column) = self.get_position(var);
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: "Int".to_string(),
                            found: format!("{}", from_type),
                            line,
                            column,
                        });
                    }
                }

                let to_type = self.analyze_expression(to);
                if let Some(to_type) = to_type {
                    if to_type != Type::Int {
                        let (line, column) = self.get_position(var);
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: "Int".to_string(),
                            found: format!("{}", to_type),
                            line,
                            column,
                        });
                    }
                }

                let step_type = self.analyze_expression(step);
                if let Some(step_type) = step_type {
                    if step_type != Type::Int {
                        let (line, column) = self.get_position(var);
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: "Int".to_string(),
                            found: format!("{}", step_type),
                            line,
                            column,
                        });
                    }
                }

                // Check for division by zero in step
                if let Expression::Literal(Literal::Int(0)) = step {
                    self.errors.push(SemanticError::Other(
                        "Step value cannot be zero".to_string(),
                    ));
                }

                // Analyze the statements in the loop body
                for stmt in body {
                    self.analyze_statement(stmt);
                }
            }
            Statement::Input(var) => {
                // Check if the variable exists and is valid for input
                match var {
                    Expression::Identifier(name) => {
                        if !self.symbol_table.contains(name) {
                            let (line, column) = self.get_position(name);
                            self.errors.push(SemanticError::UndeclaredIdentifier {
                                name: name.clone(),
                                line,
                                column,
                            });
                            return;
                        }

                        // Check if it's a constant
                        let symbol = self.symbol_table.get(name).unwrap();
                        if let SymbolKind::Constant = symbol.kind {
                            let (line, column) = self.get_position(name);
                            self.errors.push(SemanticError::ConstantModification {
                                name: name.clone(),
                                line,
                                column,
                            });
                        }
                    }
                    Expression::ArrayAccess(name, index_expr) => {
                        // Check if the array exists
                        if !self.symbol_table.contains(name) {
                            let (line, column) = self.get_position(name);
                            self.errors.push(SemanticError::UndeclaredIdentifier {
                                name: name.clone(),
                                line,
                                column,
                            });
                            return;
                        }

                        // Check if it's actually an array
                        let symbol = self.symbol_table.get(name).unwrap();
                        match &symbol.kind {
                            SymbolKind::Array(size) => {
                                // Check if the index is a constant and within bounds
                                if let Expression::Literal(Literal::Int(idx)) = &**index_expr {
                                    if *idx < 0 || *idx as usize >= *size {
                                        let (line, column) = self.get_position(name);
                                        self.errors.push(SemanticError::ArrayIndexOutOfBounds {
                                            name: name.clone(),
                                            index: *idx as usize,
                                            size: *size,
                                            line,
                                            column,
                                        });
                                    }
                                }

                                // Check index expression type
                                let idx_type = self.analyze_expression(index_expr);
                                if let Some(idx_type) = idx_type {
                                    if idx_type != Type::Int {
                                        let (line, column) = self.get_position(name);
                                        self.errors.push(SemanticError::TypeMismatch {
                                            expected: "Int".to_string(),
                                            found: format!("{}", idx_type),
                                            line,
                                            column,
                                        });
                                    }
                                }
                            }
                            _ => {
                                self.errors.push(SemanticError::Other(format!(
                                    "Cannot index non-array variable '{}'",
                                    name
                                )));
                            }
                        }
                    }
                    _ => {
                        self.errors
                            .push(SemanticError::Other("Invalid input target".to_string()));
                    }
                }
            }
            Statement::Output(exprs) => {
                // Check each expression
                for expr in exprs {
                    // For output, we just ensure expressions are valid - no specific type required
                    self.analyze_expression(expr);
                }
            }
            Statement::Empty => {
                // Nothing to check for empty statement
            }
        }
    }

    /// Analyzes an expression to determine its type
    /// Returns the type of the expression, or None if there is an error
    fn analyze_expression(&mut self, expr: &Expression) -> Option<Type> {
        match expr {
            Expression::Identifier(name) => {
                // Check if the identifier exists
                if !self.symbol_table.contains(name) {
                    let (line, column) = self.get_position(name);
                    self.errors.push(SemanticError::UndeclaredIdentifier {
                        name: name.clone(),
                        line,
                        column,
                    });
                    return None;
                }

                // Return the identifier's type
                let symbol = self.symbol_table.get(name).unwrap();
                Some(symbol.symbol_type.clone())
            }
            Expression::ArrayAccess(name, index_expr) => {
                // Check if the array exists
                if !self.symbol_table.contains(name) {
                    let (line, column) = self.get_position(name);
                    self.errors.push(SemanticError::UndeclaredIdentifier {
                        name: name.clone(),
                        line,
                        column,
                    });
                    return None;
                }

                // Check if it's actually an array
                let symbol = self.symbol_table.get(name).unwrap();
                match &symbol.kind {
                    SymbolKind::Array(size) => {
                        // Save the symbol type before releasing the borrow
                        let symbol_type = symbol.symbol_type.clone();
                        let array_size = *size;

                        // Check if the index is a constant and within bounds
                        if let Expression::Literal(Literal::Int(idx)) = &**index_expr {
                            if *idx < 0 || *idx as usize >= array_size {
                                let (line, column) = self.get_position(name);
                                self.errors.push(SemanticError::ArrayIndexOutOfBounds {
                                    name: name.clone(),
                                    index: *idx as usize,
                                    size: array_size,
                                    line,
                                    column,
                                });
                                return None;
                            }
                        }

                        // Check index expression type
                        let idx_type = self.analyze_expression(index_expr);
                        if let Some(idx_type) = idx_type {
                            if idx_type != Type::Int {
                                let (line, column) = self.get_position(name);
                                self.errors.push(SemanticError::TypeMismatch {
                                    expected: "Int".to_string(),
                                    found: format!("{}", idx_type),
                                    line,
                                    column,
                                });
                                return None;
                            }
                        } else {
                            return None;
                        }

                        // Return the array element type
                        Some(symbol_type)
                    }
                    _ => {
                        self.errors.push(SemanticError::Other(format!(
                            "Cannot index non-array variable '{}'",
                            name
                        )));
                        None
                    }
                }
            }
            Expression::Literal(lit) => {
                // Check for division by zero in constant literals
                match lit {
                    Literal::Int(_) => Some(Type::Int),
                    Literal::Float(_) => Some(Type::Float),
                    Literal::String(_) => {
                        // String literals are allowed only in output statements
                        // No specific "String" type in MiniSoft language
                        None
                    }
                }
            }
            Expression::BinaryOp(left, op, right) => {
                // Check the types of left and right operands
                let left_type = self.analyze_expression(left);
                let right_type = self.analyze_expression(right);

                if left_type.is_none() || right_type.is_none() {
                    return None;
                }

                let left_type = left_type.unwrap();
                let right_type = right_type.unwrap();

                match op {
                    // Arithmetic operators
                    Operator::Add | Operator::Subtract | Operator::Multiply | Operator::Divide => {
                        // Check for division by zero
                        if *op == Operator::Divide {
                            if let Expression::Literal(Literal::Int(0)) = **right {
                                self.errors.push(SemanticError::DivisionByZero {
                                    line: 0, // Would need position info
                                    column: 0,
                                });
                                return None;
                            } else if let Expression::Literal(Literal::Float(f)) = **right {
                                if f == 0.0 {
                                    self.errors.push(SemanticError::DivisionByZero {
                                        line: 0, // Would need position info
                                        column: 0,
                                    });
                                    return None;
                                }
                            }
                        }

                        // Arithmetic operations require numeric types
                        if left_type != right_type {
                            self.errors.push(SemanticError::TypeMismatch {
                                expected: format!("{}", left_type),
                                found: format!("{}", right_type),
                                line: 0, // Would need position info
                                column: 0,
                            });
                            return None;
                        }

                        // Return the same type for arithmetic operations
                        Some(left_type)
                    }

                    // Comparison operators
                    Operator::GreaterThan
                    | Operator::LessThan
                    | Operator::GreaterEqual
                    | Operator::LessEqual
                    | Operator::Equal
                    | Operator::NotEqual => {
                        // Comparison operations require numeric types
                        if left_type != right_type {
                            self.errors.push(SemanticError::TypeMismatch {
                                expected: format!("{}", left_type),
                                found: format!("{}", right_type),
                                line: 0, // Would need position info
                                column: 0,
                            });
                            return None;
                        }

                        // Comparison operations return boolean (represented as Int)
                        Some(Type::Int)
                    }

                    // Logical operators
                    Operator::And | Operator::Or => {
                        // Logical operations work on boolean values (Int)
                        if left_type != Type::Int || right_type != Type::Int {
                            self.errors.push(SemanticError::TypeMismatch {
                                expected: "Int".to_string(),
                                found: format!("{}, {}", left_type, right_type),
                                line: 0, // Would need position info
                                column: 0,
                            });
                            return None;
                        }

                        // Logical operations return boolean (represented as Int)
                        Some(Type::Int)
                    }
                }
            }
            Expression::UnaryOp(op, expr) => {
                // Check the type of the operand
                let expr_type = self.analyze_expression(expr);
                expr_type.as_ref()?;

                let expr_type = expr_type.unwrap();

                match op {
                    UnaryOperator::Negate => {
                        // Negation requires a numeric type
                        if expr_type != Type::Int && expr_type != Type::Float {
                            self.errors.push(SemanticError::TypeMismatch {
                                expected: "numeric type".to_string(),
                                found: format!("{}", expr_type),
                                line: 0, // Would need position info
                                column: 0,
                            });
                            return None;
                        }

                        // Negation returns the same type
                        Some(expr_type)
                    }
                    UnaryOperator::Not => {
                        // Logical negation requires a boolean value (Int)
                        if expr_type != Type::Int {
                            self.errors.push(SemanticError::TypeMismatch {
                                expected: "Int".to_string(),
                                found: format!("{}", expr_type),
                                line: 0, // Would need position info
                                column: 0,
                            });
                            return None;
                        }

                        // Logical negation returns a boolean (Int)
                        Some(Type::Int)
                    }
                }
            }
        }
    }
}
