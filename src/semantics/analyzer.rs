use crate::parser::ast::{
    Declaration, Expression, Literal, Operator, Program, Statement, Type, UnaryOperator,
};
use crate::semantics::error::SemanticError;
use crate::semantics::symbol_table::{Symbol, SymbolKind, SymbolTable};
use std::collections::{HashMap, HashSet};

// Enhanced position tracking for expressions
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
    // Track expression positions
    expression_positions: HashMap<String, (usize, usize)>,
    // Keep track of division by zero literals for better error reporting
    zero_literals: Vec<(usize, usize)>,
    // Keep track of reported error keys to avoid duplicates
    reported_errors: HashSet<String>,
}

impl SemanticAnalyzer {
    /// Creates a new semantic analyzer with position information
    pub fn new_with_positions(positions: HashMap<String, (usize, usize)>) -> Self {
        SemanticAnalyzer {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            positions,
            current_expr_pos: None,
            expression_positions: HashMap::new(),
            zero_literals: Vec::new(),
            reported_errors: HashSet::new(),
        }
    }

    /// Gets position information for an identifier
    fn get_position(&self, name: &str) -> (usize, usize) {
        // First try identifier's known position from the map
        if let Some(pos) = self.positions.get(name) {
            return *pos;
        }

        // Try the expression positions map (for computed expressions)
        if let Some(pos) = self.expression_positions.get(name) {
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

    /// Track a position for a specific expression key
    fn track_expression_pos(&mut self, key: String, line: usize, column: usize) {
        self.expression_positions.insert(key, (line, column));
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

        // Clear any accumulated positions to start fresh for statements
        self.clear_current_expr_pos();

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

    /// Extracts a literal value from an expression if possible
    fn extract_literal(&self, expr: &Expression) -> Option<Literal> {
        match expr {
            Expression::Literal(lit) => Some(lit.clone()),
            Expression::UnaryOp(UnaryOperator::Negate, inner) => {
                if let Some(inner_lit) = self.extract_literal(inner) {
                    match inner_lit {
                        Literal::Int(val) => Some(Literal::Int(-val)),
                        Literal::Float(val) => Some(Literal::Float(-val)),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            // Only handle simple literals, not complex expressions
            _ => None,
        }
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

                // Check for division by zero in constants
                if let Literal::Int(0) = value {
                    self.zero_literals.push((line, column));
                } else if let Literal::Float(f) = value {
                    if *f == 0.0 {
                        self.zero_literals.push((line, column));
                    }
                }

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
                
                // Try to extract the literal value
                let literal_value = self.extract_literal(expr);
                
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

                    // Add to symbol table with value if it's a literal
                    let symbol = Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Variable,
                        symbol_type: typ.clone(),
                        value: literal_value.clone(), // Store the value if it's a literal
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

                // Try to extract all literal values
                let mut literal_values = Vec::new();
                let mut all_literals = true;
                
                for value in values {
                    if let Some(lit) = self.extract_literal(value) {
                        literal_values.push(lit);
                    } else {
                        all_literals = false;
                        break;
                    }
                    
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

                    // Create an array initializer literal if we have all literals
                    let array_value = if all_literals && !literal_values.is_empty() {
                        Some(Literal::String(format!("{:?}", literal_values))) // Use String type as a temp container
                    } else {
                        None
                    };

                    // Add to symbol table
                    let symbol = Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Array(*size),
                        symbol_type: typ.clone(),
                        value: array_value, // Store array literal values if all are literals
                        line,
                        column,
                    };
                    self.symbol_table.add_symbol(symbol);
                }
            }
        }
    }

    /// Checks if two types are compatible for operations
    fn are_types_compatible(&self, type1: &Type, type2: &Type) -> bool {
        // Same types are always compatible
        if type1 == type2 {
            return true;
        }

        // For numeric operations, Int and Float can work together
        matches!(
            (type1, type2),
            (Type::Int, Type::Float) | (Type::Float, Type::Int)
        )
    }

    /// Determines the resulting type when operating on two types
    fn resulting_type(&self, type1: &Type, type2: &Type) -> Type {
        if type1 == type2 {
            return type1.clone();
        }

        // If either type is Float, the result is Float
        if *type1 == Type::Float || *type2 == Type::Float {
            return Type::Float;
        }

        // Default to the first type
        type1.clone()
    }

    /// Adds an error if it hasn't been reported yet
    fn add_error(&mut self, error: SemanticError) {
        // Create a unique key for this error to avoid duplicates
        let error_key = match &error {
            SemanticError::TypeMismatch {
                expected,
                found,
                line,
                column,
            } => {
                format!("type_mismatch:{}:{}:{}:{}", expected, found, line, column)
            }
            SemanticError::UndeclaredIdentifier { name, line, column } => {
                format!("undeclared:{}:{}:{}", name, line, column)
            }
            // Add other error types as needed
            _ => format!("{:?}", error),
        };

        // Only add the error if we haven't reported it yet
        if !self.reported_errors.contains(&error_key) {
            self.errors.push(error);
            self.reported_errors.insert(error_key);
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
                            self.add_error(SemanticError::UndeclaredIdentifier {
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
                            self.add_error(SemanticError::ConstantModification {
                                name: name.clone(),
                                line,
                                column,
                            });
                            return;
                        }

                        // Get the type and position of the identifier
                        let lhs_type = symbol.symbol_type.clone();
                        let (line, column) = (symbol.line, symbol.column);

                        // Store the position for later reference
                        let expr_key = format!("assign_{}", name);
                        self.track_expression_pos(expr_key, line, column);

                        // Check if the right-hand side expression matches the type
                        let rhs_type = self.analyze_expression(rhs);
                        if let Some(rhs_type) = rhs_type {
                            // Allow automatic conversion from Float to Int for assignment
                            if rhs_type == Type::Float && lhs_type == Type::Int {
                                // This is a valid implicit conversion (with potential data loss)
                                // Could add a warning here if desired
                            }
                            // For other type mismatches, report an error
                            else if rhs_type != lhs_type {
                                self.add_error(SemanticError::TypeMismatch {
                                    expected: format!("{}", lhs_type),
                                    found: format!("{}", rhs_type),
                                    line,
                                    column,
                                });
                            }
                        }
                    }
                    Expression::ArrayAccess(name, index_expr) => {
                        // Track where the array access occurs
                        let (line, column) = self.get_position(name);
                        let expr_key = format!("array_access_{}", name);
                        self.track_expression_pos(expr_key, line, column);

                        // Check if the array exists
                        if !self.symbol_table.contains(name) {
                            self.add_error(SemanticError::UndeclaredIdentifier {
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
                                // Save size, type, and position information
                                let array_size = *size;
                                let element_type = symbol.symbol_type.clone();
                                let (array_line, array_col) = (symbol.line, symbol.column);

                                // Check if the index is a constant and within bounds
                                if let Expression::Literal(Literal::Int(idx)) = &**index_expr {
                                    if *idx < 0 || *idx as usize >= array_size {
                                        self.add_error(SemanticError::ArrayIndexOutOfBounds {
                                            name: name.clone(),
                                            index: *idx as usize,
                                            size: array_size,
                                            line,
                                            column,
                                        });
                                        return;
                                    }
                                }

                                // Check index expression type
                                let idx_type = self.analyze_expression(index_expr);
                                if let Some(idx_type) = idx_type {
                                    if idx_type != Type::Int {
                                        // Use the position of the index expression
                                        let (idx_line, idx_col) =
                                            if let Some(pos) = &self.current_expr_pos {
                                                (pos.line, pos.column)
                                            } else {
                                                (line, column + name.len() + 1) // Estimate index position
                                            };

                                        self.add_error(SemanticError::TypeMismatch {
                                            expected: "Int".to_string(),
                                            found: format!("{}", idx_type),
                                            line: idx_line,
                                            column: idx_col,
                                        });
                                        return;
                                    }
                                }

                                // Check right-hand side type against element type
                                let rhs_type = self.analyze_expression(rhs);
                                if let Some(rhs_type) = rhs_type {
                                    if rhs_type != element_type {
                                        self.add_error(SemanticError::TypeMismatch {
                                            expected: format!("{}", element_type),
                                            found: format!("{}", rhs_type),
                                            line: array_line,
                                            column: array_col,
                                        });
                                    }
                                }
                            }
                            _ => {
                                self.add_error(SemanticError::Other(format!(
                                    "Cannot index non-array variable '{}'",
                                    name
                                )));
                            }
                        }
                    }
                    _ => {
                        self.add_error(SemanticError::Other(
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
                // Get position of the loop variable
                let (var_line, var_col) = self.get_position(var);

                // Check if the loop variable exists
                if !self.symbol_table.contains(var) {
                    self.add_error(SemanticError::UndeclaredIdentifier {
                        name: var.clone(),
                        line: var_line,
                        column: var_col,
                    });
                } else {
                    // Check if the loop variable is an integer
                    let symbol = self.symbol_table.get(var).unwrap();
                    if symbol.symbol_type != Type::Int {
                        self.add_error(SemanticError::TypeMismatch {
                            expected: "Int".to_string(),
                            found: format!("{}", symbol.symbol_type),
                            line: var_line,
                            column: var_col,
                        });
                    }
                }

                // Store current position for better error reporting
                self.set_current_expr_pos(var_line, var_col);

                // Check that from, to, and step are all numeric expressions
                let from_type = self.analyze_expression(from);
                if let Some(from_type) = from_type {
                    if from_type != Type::Int {
                        // Use better position for "from" expression
                        let (from_line, from_col) = if let Some(pos) = &self.current_expr_pos {
                            (pos.line, pos.column + 5) // Estimate after "from" keyword
                        } else {
                            (var_line, var_col)
                        };

                        self.add_error(SemanticError::TypeMismatch {
                            expected: "Int".to_string(),
                            found: format!("{}", from_type),
                            line: from_line,
                            column: from_col,
                        });
                    }
                }

                let to_type = self.analyze_expression(to);
                if let Some(to_type) = to_type {
                    if to_type != Type::Int {
                        // Use better position for "to" expression
                        let (to_line, to_col) = if let Some(pos) = &self.current_expr_pos {
                            (pos.line, pos.column + 3) // Estimate after "to" keyword
                        } else {
                            (var_line, var_col)
                        };

                        self.add_error(SemanticError::TypeMismatch {
                            expected: "Int".to_string(),
                            found: format!("{}", to_type),
                            line: to_line,
                            column: to_col,
                        });
                    }
                }

                let step_type = self.analyze_expression(step);
                if let Some(step_type) = step_type {
                    if step_type != Type::Int {
                        // Use better position for "step" expression
                        let (step_line, step_col) = if let Some(pos) = &self.current_expr_pos {
                            (pos.line, pos.column + 5) // Estimate after "step" keyword
                        } else {
                            (var_line, var_col)
                        };

                        self.add_error(SemanticError::TypeMismatch {
                            expected: "Int".to_string(),
                            found: format!("{}", step_type),
                            line: step_line,
                            column: step_col,
                        });
                    }
                }

                // Check for division by zero in step
                if let Expression::Literal(Literal::Int(0)) = step {
                    // Use specific position for step
                    let (step_line, step_col) = if let Some(pos) = &self.current_expr_pos {
                        (pos.line, pos.column + 5)
                    } else {
                        (var_line, var_col)
                    };

                    self.add_error(SemanticError::DivisionByZero {
                        line: step_line,
                        column: step_col,
                    });
                }

                // Analyze the statements in the loop body
                self.clear_current_expr_pos(); // Clear before entering the body
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
                            self.add_error(SemanticError::UndeclaredIdentifier {
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
                            self.add_error(SemanticError::ConstantModification {
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
                            self.add_error(SemanticError::UndeclaredIdentifier {
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
                                        self.add_error(SemanticError::ArrayIndexOutOfBounds {
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
                                        self.add_error(SemanticError::TypeMismatch {
                                            expected: "Int".to_string(),
                                            found: format!("{}", idx_type),
                                            line,
                                            column,
                                        });
                                    }
                                }
                            }
                            _ => {
                                self.add_error(SemanticError::Other(format!(
                                    "Cannot index non-array variable '{}'",
                                    name
                                )));
                            }
                        }
                    }
                    _ => {
                        self.add_error(SemanticError::Other("Invalid input target".to_string()));
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
                    self.add_error(SemanticError::UndeclaredIdentifier {
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
                    self.add_error(SemanticError::UndeclaredIdentifier {
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
                                self.add_error(SemanticError::ArrayIndexOutOfBounds {
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
                                self.add_error(SemanticError::TypeMismatch {
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
                        self.add_error(SemanticError::Other(format!(
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
                // Save the current position before recursion
                let saved_pos = self.current_expr_pos.clone();

                // Check the types of left and right operands
                let left_type = self.analyze_expression(left);
                let right_type = self.analyze_expression(right);

                // Restore the position after recursion
                self.current_expr_pos = saved_pos;

                if left_type.is_none() || right_type.is_none() {
                    return None;
                }

                let left_type = left_type.unwrap();
                let right_type = right_type.unwrap();

                // Get the current position for error reporting
                let (line, column) = if let Some(pos) = &self.current_expr_pos {
                    (pos.line, pos.column)
                } else {
                    (1, 1) // Default
                };

                match op {
                    // Arithmetic operators
                    Operator::Add | Operator::Subtract | Operator::Multiply | Operator::Divide => {
                        // Check for division by zero
                        if *op == Operator::Divide {
                            if let Expression::Literal(Literal::Int(0)) = **right {
                                // Get the position of the zero literal if possible
                                let (zero_line, zero_col) = if !self.zero_literals.is_empty() {
                                    self.zero_literals[0]
                                } else {
                                    (line, column + 1) // Estimate position of the divisor
                                };

                                self.add_error(SemanticError::DivisionByZero {
                                    line: zero_line,
                                    column: zero_col,
                                });
                                return None;
                            } else if let Expression::Literal(Literal::Float(f)) = **right {
                                if f == 0.0 {
                                    // Get the position of the zero literal if possible
                                    let (zero_line, zero_col) = if !self.zero_literals.is_empty() {
                                        self.zero_literals[0]
                                    } else {
                                        (line, column + 1) // Estimate position of the divisor
                                    };

                                    self.add_error(SemanticError::DivisionByZero {
                                        line: zero_line,
                                        column: zero_col,
                                    });
                                    return None;
                                }
                            }
                        }

                        // For arithmetic operations, allow mixed numeric types
                        if self.are_types_compatible(&left_type, &right_type) {
                            // Return the resulting type (Float if mixing Int and Float)
                            Some(self.resulting_type(&left_type, &right_type))
                        } else {
                            self.add_error(SemanticError::TypeMismatch {
                                expected: format!("{}", left_type),
                                found: format!("{}", right_type),
                                line,
                                column,
                            });
                            None
                        }
                    }

                    // Comparison operators
                    Operator::GreaterThan
                    | Operator::LessThan
                    | Operator::GreaterEqual
                    | Operator::LessEqual
                    | Operator::Equal
                    | Operator::NotEqual => {
                        // For comparison operations, allow mixed numeric types
                        if self.are_types_compatible(&left_type, &right_type) {
                            // Comparison operations return boolean (represented as Int)
                            Some(Type::Int)
                        } else {
                            self.add_error(SemanticError::TypeMismatch {
                                expected: format!("{}", left_type),
                                found: format!("{}", right_type),
                                line,
                                column,
                            });
                            None
                        }
                    }

                    // Logical operators
                    Operator::And | Operator::Or => {
                        // Logical operations work on boolean values (Int)
                        if left_type != Type::Int || right_type != Type::Int {
                            self.add_error(SemanticError::TypeMismatch {
                                expected: "Int".to_string(),
                                found: format!("{}, {}", left_type, right_type),
                                line,
                                column,
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
                            self.add_error(SemanticError::TypeMismatch {
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
                            self.add_error(SemanticError::TypeMismatch {
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
