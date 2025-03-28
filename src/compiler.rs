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
                            symbol.name.white().bold(),
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

        position_map
    }

    fn find_identifier_position(&self, name: &str) -> Option<(usize, usize)> {
        // Simple search for identifier in source
        let lines: Vec<&str> = self.source_code.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            if let Some(col_idx) = line.find(name) {
                // Verify this is a proper identifier boundary
                let is_valid_start = col_idx == 0
                    || !line
                        .chars()
                        .nth(col_idx - 1)
                        .unwrap_or(' ')
                        .is_alphanumeric();
                let end_idx = col_idx + name.len();
                let is_valid_end = end_idx >= line.len()
                    || !line.chars().nth(end_idx).unwrap_or(' ').is_alphanumeric();

                if is_valid_start && is_valid_end {
                    return Some((line_idx + 1, col_idx + 1)); // 1-based indexing
                }
            }
        }

        None
    }
}
