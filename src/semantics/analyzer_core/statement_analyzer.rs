use crate::parser::ast::{Expression, ExpressionKind, Statement, StatementKind, Type};
use crate::semantics::analyzer_core::SemanticAnalyzer;
use crate::semantics::error::SemanticError;

impl SemanticAnalyzer {
    pub fn analyze_statement(&mut self, stmt: &Statement) {
        // Implementation of statement analysis
        match &stmt.node {
            StatementKind::Assignment(left, right) => {
                // Analyze both sides of the assignment
                self.analyze_expression(left);
                self.analyze_expression(right);

                // Check if left side is assignable
                if let ExpressionKind::Identifier(name) = &left.node {
                    if let Some(symbol) = self.symbol_table.get(name) {
                        if symbol.is_constant {
                            self.errors.push(SemanticError::ConstantModification {
                                name: name.clone(),
                                line: self.get_line(left.span.start),
                                column: self.get_column(left.span.start),
                            });
                        }
                    }
                }

                // Check type compatibility
                let left_type = self.get_expression_type(left);
                let right_type = self.get_expression_type(right);

                if let (Some(left_t), Some(right_t)) = (left_type, right_type) {
                    if left_t != right_t {
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: left_t.to_string(),
                            found: right_t.to_string(),
                            line: self.get_line(right.span.start),
                            column: self.get_column(right.span.start),
                            context: Some("assignment".to_string()),
                        });
                    }
                }
            }

            StatementKind::IfThen(condition, then_block) => {
                // Analyze condition
                self.analyze_expression(condition);

                // Ensure condition is boolean
                if let Some(cond_type) = self.get_expression_type(condition) {
                    if cond_type != Type::Bool {
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: Type::Bool.to_string(),
                            found: cond_type.to_string(),
                            line: self.get_line(condition.span.start),
                            column: self.get_column(condition.span.start),
                            context: Some("condition".to_string()),
                        });
                    }
                }

                // Analyze statements in then block
                for stmt in then_block {
                    self.analyze_statement(stmt);
                }
            }

            StatementKind::IfThenElse(condition, then_block, else_block) => {
                // Analyze condition
                self.analyze_expression(condition);

                // Ensure condition is boolean
                if let Some(cond_type) = self.get_expression_type(condition) {
                    if cond_type != Type::Bool {
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: Type::Bool.to_string(),
                            found: cond_type.to_string(),
                            line: self.get_line(condition.span.start),
                            column: self.get_column(condition.span.start),
                            context: Some("condition".to_string()),
                        });
                    }
                }

                // Analyze statements in then block
                for stmt in then_block {
                    self.analyze_statement(stmt);
                }

                // Analyze statements in else block
                for stmt in else_block {
                    self.analyze_statement(stmt);
                }
            }

            StatementKind::DoWhile(body, condition) => {
                // Analyze loop body
                for stmt in body {
                    self.analyze_statement(stmt);
                }

                // Analyze condition
                self.analyze_expression(condition);

                // Ensure condition is boolean
                if let Some(cond_type) = self.get_expression_type(condition) {
                    if cond_type != Type::Bool {
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: Type::Bool.to_string(),
                            found: cond_type.to_string(),
                            line: self.get_line(condition.span.start),
                            column: self.get_column(condition.span.start),
                            context: Some("condition".to_string()),
                        });
                    }
                }
            }

            StatementKind::For(iterator, init, condition, update, body) => {
                // Analyze initialization
                self.analyze_expression(init);

                // Analyze condition
                self.analyze_expression(condition);

                // Ensure condition is boolean
                if let Some(cond_type) = self.get_expression_type(condition) {
                    if cond_type != Type::Bool {
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: Type::Bool.to_string(),
                            found: cond_type.to_string(),
                            line: self.get_line(condition.span.start),
                            column: self.get_column(condition.span.start),
                            context: Some("condition".to_string()),
                        });
                    }
                }

                // Analyze update
                self.analyze_expression(update);

                // Analyze loop body
                for stmt in body {
                    self.analyze_statement(stmt);
                }
            }

            StatementKind::Input(target) => {
                // Analyze the target expression
                self.analyze_expression(target);

                // Check if target is assignable
                if let ExpressionKind::Identifier(name) = &target.node {
                    if let Some(symbol) = self.symbol_table.get(name) {
                        if symbol.is_constant {
                            self.errors.push(SemanticError::ConstantModification {
                                name: name.clone(),
                                line: self.get_line(target.span.start),
                                column: self.get_column(target.span.start),
                            });
                        }
                    }
                }
            }

            StatementKind::Output(expressions) => {
                // Analyze all expressions in the output list
                for expr in expressions {
                    self.analyze_expression(expr);
                }
            }

            StatementKind::Scope(statements) => {
                // Analyze all statements in the block
                for stmt in statements {
                    self.analyze_statement(stmt);
                }
            }

            StatementKind::Empty => {
                // No-op for empty statements
            }
        }
    }

    fn get_expression_type(&self, expr: &Expression) -> Option<Type> {
        // Implementation to determine expression type would go here
        None
    }

    fn get_line(&self, pos: usize) -> usize {
        // Implementation to convert position to line number
        1
    }

    fn get_column(&self, pos: usize) -> usize {
        // Implementation to convert position to column number
        1
    }
}
