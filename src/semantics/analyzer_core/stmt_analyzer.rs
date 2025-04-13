use crate::parser::ast::{Expression, Literal, Statement, Type};
use crate::semantics::analyzer_core::SemanticAnalyzer;
use crate::semantics::error::SemanticError;
use crate::semantics::symbol_table::SymbolKind;

impl SemanticAnalyzer {
    /// Analyzes a statement
    pub(crate) fn analyze_statement(&mut self, statement: &Statement) {
        match &statement.node {
            Statement::Block(statements) => {
                // Analyze each statement in the block
                for stmt in statements {
                    self.analyze_statement(stmt);
                }
            }
            Statement::Assignment(lhs, rhs) => {
                // First analyze the left-hand side (target)
                match &lhs.node {
                    Expression::Identifier(name) => {
                        // Check if the identifier exists
                        if !self.symbol_table.contains(name) {
                            let (line, column) = self.get_span_location(&lhs.span);
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
                            let (line, column) = self.get_span_location(&lhs.span);
                            self.add_error(SemanticError::ConstantModification {
                                name: name.clone(),
                                line,
                                column,
                            });
                            return;
                        }

                        // Get the type of the identifier
                        let lhs_type = symbol.symbol_type.clone();
                        let (line, column) = self.get_span_location(&lhs.span);

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
                                    context: None,
                                });
                            }
                        }
                    }
                    Expression::ArrayAccess(name, index_expr) => {
                        // Get the span location of array access
                        let (line, column) = self.get_span_location(&lhs.span);

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
                                let array_size = *size;
                                let element_type = symbol.symbol_type.clone();

                                // Check if the index is a constant and within bounds
                                if let Expression::Literal(Literal::Int(idx)) = &index_expr.node {
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
                                        // Use the index expression's span location
                                        let (idx_line, idx_col) =
                                            self.get_span_location(&index_expr.span);
                                        self.add_error(SemanticError::TypeMismatch {
                                            expected: "Int".to_string(),
                                            found: format!("{}", idx_type),
                                            line: idx_line,
                                            column: idx_col,
                                            context: None,
                                        });
                                        return;
                                    }
                                }

                                // Check right-hand side type against element type
                                let rhs_type = self.analyze_expression(rhs);
                                if let Some(rhs_type) = rhs_type {
                                    if rhs_type != element_type {
                                        // Report error with span of right-hand side
                                        let (rhs_line, rhs_col) = self.get_span_location(&rhs.span);
                                        self.add_error(SemanticError::TypeMismatch {
                                            expected: format!("{}", element_type),
                                            found: format!("{}", rhs_type),
                                            line: rhs_line,
                                            column: rhs_col,
                                            context: None,
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
                // Process the for loop variable and expressions
                self.analyze_for_loop(var, from, to, step, body);
            }
            Statement::Input(var) => {
                // Check if the variable exists and is valid for input
                self.analyze_input_target(var);
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

    // Helper method to analyze for loops (updated to use span locations)
    fn analyze_for_loop(
        &mut self,
        var: &str,
        from: &Expression,
        to: &Expression,
        step: &Expression,
        body: &[Statement],
    ) {
        // Use span locations from expressions
        let (var_line, var_col) = self.get_span_location(&from.span); // Use from's span as approximate location

        // Check if the loop variable exists
        if !self.symbol_table.contains(var) {
            self.add_error(SemanticError::UndeclaredIdentifier {
                name: var.to_string(),
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
                    context: None,
                });
            }
        }

        // Check that from, to, and step are all numeric expressions
        let from_type = self.analyze_expression(from);
        if let Some(from_type) = from_type {
            if from_type != Type::Int {
                let (from_line, from_col) = self.get_span_location(&from.span);
                self.add_error(SemanticError::TypeMismatch {
                    expected: "Int".to_string(),
                    found: format!("{}", from_type),
                    line: from_line,
                    column: from_col,
                    context: Some("from expression".to_string()),
                });
            }
        }

        let to_type = self.analyze_expression(to);
        if let Some(to_type) = to_type {
            if to_type != Type::Int {
                let (to_line, to_col) = self.get_span_location(&to.span);
                self.add_error(SemanticError::TypeMismatch {
                    expected: "Int".to_string(),
                    found: format!("{}", to_type),
                    line: to_line,
                    column: to_col,
                    context: Some("to expression".to_string()),
                });
            }
        }

        let step_type = self.analyze_expression(step);
        if let Some(step_type) = step_type {
            if step_type != Type::Int {
                let (step_line, step_col) = self.get_span_location(&step.span);
                self.add_error(SemanticError::TypeMismatch {
                    expected: "Int".to_string(),
                    found: format!("{}", step_type),
                    line: step_line,
                    column: step_col,
                    context: Some("step expression".to_string()),
                });
            }
        }

        // Check for division by zero in step
        if let Expression::Literal(Literal::Int(0)) = step.node {
            let (step_line, step_col) = self.get_span_location(&step.span);
            self.add_error(SemanticError::DivisionByZero {
                line: step_line,
                column: step_col,
            });
        }

        // Analyze the statements in the loop body
        for stmt in body {
            self.analyze_statement(stmt);
        }
    }

    // Helper method to analyze input targets (updated to use span locations)
    fn analyze_input_target(&mut self, var: &Expression) {
        match &var.node {
            Expression::Identifier(name) => {
                if !self.symbol_table.contains(name) {
                    let (line, column) = self.get_span_location(&var.span);
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
                    let (line, column) = self.get_span_location(&var.span);
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
                    let (line, column) = self.get_span_location(&var.span);
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
                        if let Expression::Literal(Literal::Int(idx)) = &index_expr.node {
                            if *idx < 0 || *idx as usize >= *size {
                                let (line, column) = self.get_span_location(&var.span);
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
                                let (line, column) = self.get_span_location(&index_expr.span);
                                self.add_error(SemanticError::TypeMismatch {
                                    expected: "Int".to_string(),
                                    found: format!("{}", idx_type),
                                    line,
                                    column,
                                    context: Some("array index".to_string()),
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
}
