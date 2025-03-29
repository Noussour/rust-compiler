use crate::error_reporter::ErrorReporter;
use crate::lexer::{lexer_core::TokenWithPosition, token::Token, Lexer};
use crate::parser::parse;
use crate::semantics::{SemanticAnalyzer, SymbolKind};
use colored::*;
use std::collections::HashMap;
use std::fs;

pub struct Compiler {
    source_code: String,
    file_path: String,
    error_reporter: ErrorReporter,
}

impl Compiler {
    pub fn new(file_path: &str) -> Result<Self, String> {
        match fs::read_to_string(file_path) {
            Ok(content) => {
                let error_reporter = ErrorReporter::new(&content, file_path);
                Ok(Self {
                    source_code: content,
                    file_path: file_path.to_string(),
                    error_reporter,
                })
            }
            Err(e) => Err(format!("Error reading file '{}': {}", file_path, e)),
        }
    }

    pub fn run(&mut self) -> Result<(), i32> {
        println!("Compiling file: {}", self.file_path);
        self.print_source_code();

        // STEP 1: Lexical Analysis
        // Tokenize the source code and capture lexical errors
        let tokens = self.tokenize();

        // Check for lexical errors
        let mut has_lexical_errors = false;
        for token in &tokens {
            if let Token::Error = &token.token {
                self.error_reporter.add_lexical_error(
                    &token.text,
                    token.position.line,
                    token.position.column,
                );
                has_lexical_errors = true;
            }
        }

        // If lexical errors, report and exit early
        if has_lexical_errors {
            println!("{}", "Lexical errors detected".red().bold());
            self.error_reporter.report_errors();
            return Err(1);
        }

        self.print_tokens(&tokens);

        // STEP 2: Syntax Analysis
        println!("\n{}", "Parsing:".bold().underline());

        // Parse tokens into an AST
        match parse(tokens) {
            Ok(program) => {
                println!("{}", "AST:".green());
                println!("{:#?}", program);

                // STEP 3: Semantic Analysis
                println!("\n{}", "Semantic Analysis:".bold().underline());

                // Build position map for better error reporting
                let position_map = self.build_position_map(&program);

                let mut analyzer = SemanticAnalyzer::new_with_positions(position_map);
                analyzer.analyze(&program);

                // Check for semantic errors
                let semantic_errors = analyzer.get_errors();
                if !semantic_errors.is_empty() {
                    println!("{}", "Semantic Errors Detected".red().bold());
                    for error in semantic_errors {
                        self.error_reporter.add_semantic_error(error);
                    }
                } else {
                    println!("{}", "No semantic errors found".green());

                    // Display symbol table
                    println!("\n{}", "Symbol Table:".bold().underline());
                    let symbol_table = analyzer.get_symbol_table();
                    for symbol in symbol_table.get_all() {
                        let kind = match &symbol.kind {
                            SymbolKind::Variable => "Variable".cyan(),
                            SymbolKind::Constant => "Constant".yellow(),
                            SymbolKind::Array(size) => format!("Array[{}]", size).magenta(),
                        };

                        let value = if let Some(val) = &symbol.value {
                            format!("{:?}", val).green()
                        } else {
                            "<uninitialized>".dimmed()
                        };

                        println!(
                            "{} {} {} = {} (line {}, col {})",
                            kind,
                            symbol.name.white(),
                            format!("({})", symbol.symbol_type).blue(),
                            value,
                            symbol.line,
                            symbol.column
                        );
                    }
                }
            }
            Err(parse_error) => {
                println!("{}", "Parser Error:".red().bold());
                self.error_reporter.add_parse_error(&parse_error);
            }
        }

        // Final error reporting
        if self.error_reporter.has_errors() {
            self.error_reporter.report_errors();
            Err(1) // Return error code
        } else {
            println!("{}", "Compilation successful!".green().bold());
            Ok(()) // Successful compilation
        }
    }

    fn print_source_code(&self) {
        println!("{}", "Source code:".bold().underline());
        println!("{}\n", self.source_code);
    }

    fn tokenize(&self) -> Vec<TokenWithPosition> {
        let lexer = Lexer::new(&self.source_code);
        let tokens: Vec<_> = lexer.collect();
        tokens
    }

    fn print_tokens(&self, tokens: &[TokenWithPosition]) {
        println!("{}", "Tokens:".bold().underline());
        for token_with_pos in tokens {
            let token_name = format!("{:?}", token_with_pos.token).green();
            let token_value = token_with_pos.text.yellow();
            let position = format!(
                "Line {}, Col {}",
                token_with_pos.position.line, token_with_pos.position.column
            )
            .blue();

            println!(
                "{}  →  {}  {}  [span: {}]",
                token_name,
                token_value,
                position,
                format!("{:?}", token_with_pos.span).magenta()
            );
        }
    }

    fn build_position_map(
        &self,
        program: &crate::parser::ast::Program,
    ) -> HashMap<String, (usize, usize)> {
        let mut position_map = HashMap::new();

        // Add program name position if available
        if let Some(line_col) = self.find_identifier_position(&program.name) {
            position_map.insert(program.name.clone(), line_col);
        }

        // Process declarations to get positions of all variables
        for decl in &program.declarations {
            match decl {
                crate::parser::ast::Declaration::Variable(names, _)
                | crate::parser::ast::Declaration::Array(names, _, _)
                | crate::parser::ast::Declaration::VariableWithInit(names, _, _)
                | crate::parser::ast::Declaration::ArrayWithInit(names, _, _, _) => {
                    for name in names {
                        if let Some(line_col) = self.find_identifier_position(name) {
                            position_map.insert(name.clone(), line_col);
                        }
                    }
                }
                crate::parser::ast::Declaration::Constant(name, _, _) => {
                    if let Some(line_col) = self.find_identifier_position(name) {
                        position_map.insert(name.clone(), line_col);
                    }
                }
            }
        }

        // Also collect positions from statements to improve error reporting
        self.collect_statement_positions(&program.statements, &mut position_map);

        position_map
    }

    fn collect_statement_positions(
        &self,
        statements: &[crate::parser::ast::Statement],
        position_map: &mut HashMap<String, (usize, usize)>,
    ) {
        for statement in statements {
            match statement {
                crate::parser::ast::Statement::Assignment(target, expr) => {
                    // Collect identifiers from the target
                    self.collect_expr_positions(target, position_map);
                    // Collect identifiers from the expression
                    self.collect_expr_positions(expr, position_map);
                }
                crate::parser::ast::Statement::IfThen(condition, then_block) => {
                    self.collect_expr_positions(condition, position_map);
                    self.collect_statement_positions(then_block, position_map);
                }
                crate::parser::ast::Statement::IfThenElse(condition, then_block, else_block) => {
                    self.collect_expr_positions(condition, position_map);
                    self.collect_statement_positions(then_block, position_map);
                    self.collect_statement_positions(else_block, position_map);
                }
                crate::parser::ast::Statement::DoWhile(body, condition) => {
                    self.collect_statement_positions(body, position_map);
                    self.collect_expr_positions(condition, position_map);
                }
                crate::parser::ast::Statement::For(var, from, to, step, body) => {
                    // Add position for the loop variable
                    if let Some(line_col) = self.find_identifier_position(var) {
                        position_map.insert(var.clone(), line_col);
                    }
                    self.collect_expr_positions(from, position_map);
                    self.collect_expr_positions(to, position_map);
                    self.collect_expr_positions(step, position_map);
                    self.collect_statement_positions(body, position_map);
                }
                crate::parser::ast::Statement::Input(var) => {
                    self.collect_expr_positions(var, position_map);
                }
                crate::parser::ast::Statement::Output(exprs) => {
                    for expr in exprs {
                        self.collect_expr_positions(expr, position_map);
                    }
                }
                crate::parser::ast::Statement::Empty => {}
            }
        }
    }

    fn collect_expr_positions(
        &self,
        expr: &crate::parser::ast::Expression,
        position_map: &mut HashMap<String, (usize, usize)>,
    ) {
        match expr {
            crate::parser::ast::Expression::Identifier(name) => {
                if let Some(line_col) = self.find_identifier_position(name) {
                    position_map.insert(name.clone(), line_col);
                }
            }
            crate::parser::ast::Expression::ArrayAccess(name, index) => {
                if let Some(line_col) = self.find_identifier_position(name) {
                    position_map.insert(name.clone(), line_col);

                    // Also track array_name[] as a separate key
                    let array_access_key = format!("{}_access", name);
                    position_map.insert(array_access_key, line_col);
                }
                self.collect_expr_positions(index, position_map);
            }
            crate::parser::ast::Expression::BinaryOp(left, _, right) => {
                self.collect_expr_positions(left, position_map);
                self.collect_expr_positions(right, position_map);
            }
            crate::parser::ast::Expression::UnaryOp(_, operand) => {
                self.collect_expr_positions(operand, position_map);
            }
            crate::parser::ast::Expression::Literal(_) => {
                // No identifiers to track in literals
            }
        }
    }

    // Improved identifier position finder with more context
    fn find_identifier_position(&self, name: &str) -> Option<(usize, usize)> {
        // Simple search for identifier in source
        let lines: Vec<&str> = self.source_code.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let mut search_pos = 0;

            // Look for all occurrences of the name in this line
            while let Some(col_idx) = line[search_pos..].find(name) {
                let actual_col_idx = search_pos + col_idx;

                // Verify this is a proper identifier boundary
                let is_valid_start = actual_col_idx == 0
                    || !line
                        .chars()
                        .nth(actual_col_idx - 1)
                        .unwrap_or(' ')
                        .is_alphanumeric();
                let end_idx = actual_col_idx + name.len();
                let is_valid_end = end_idx >= line.len()
                    || !line.chars().nth(end_idx).unwrap_or(' ').is_alphanumeric();

                if is_valid_start && is_valid_end {
                    return Some((line_idx + 1, actual_col_idx + 1)); // 1-based indexing
                }

                // Move past this occurrence
                search_pos = actual_col_idx + 1;

                // Safety check
                if search_pos >= line.len() {
                    break;
                }
            }
        }

        None
    }
}
