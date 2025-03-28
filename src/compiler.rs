use crate::error_reporter::{ErrorKind, ErrorReporter};
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
        
        // Tokenize the source code and capture lexical errors
        let tokens = self.tokenize();
        
        // Check for lexical errors
        let mut has_lexical_errors = false;
        for token in &tokens {
            if let Token::Error = &token.token {
                self.error_reporter.add_error(
                    ErrorKind::Lexical,
                    &format!("Invalid token: '{}'", token.text),
                    token.position.line,
                    token.position.column
                );
                has_lexical_errors = true;
            }
        }
        
        if has_lexical_errors {
            println!("{}", "Lexical errors detected".red().bold());
            return Err(1);
        }
        
        self.print_tokens(&tokens);
        
        // Parse and analyze, capture errors
        let _ = self.parse_and_analyze(tokens);
        
        // Report any errors that were found
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

    fn parse_and_analyze(&mut self, tokens: Vec<TokenWithPosition>) -> Result<(), ()> {
        // Build a position map for identifiers and other tokens of interest
        let mut position_map = HashMap::new();
        for token in &tokens {
            match &token.token {
                Token::Identifier(name) => {
                    position_map.insert(name.clone(), (token.position.line, token.position.column));
                },
                Token::IntLiteral(val) => {
                    // Track positions of literals for potential division by zero errors
                    if *val == 0 {
                        let key = format!("int_literal_{}", val);
                        position_map.insert(key, (token.position.line, token.position.column));
                    }
                },
                Token::FloatLiteral(val) => {
                    if *val == 0.0 {
                        let key = format!("float_literal_{}", val);
                        position_map.insert(key, (token.position.line, token.position.column));
                    }
                },
                _ => {}
            }
        }

        // Try parsing the tokens and print the result
        println!("\n{}", "Parsing:".bold().underline());
        match parse(tokens) {
            Ok(program) => {
                println!("{}", "AST:".green());
                println!("{:#?}", program);

                // Semantic analysis
                println!("\n{}", "Semantic Analysis:".bold().underline());
                let mut analyzer = SemanticAnalyzer::new_with_positions(position_map);
                analyzer.analyze(&program);

                // Check for semantic errors
                let errors = analyzer.get_errors();
                if !errors.is_empty() {
                    println!("{}", "Semantic Errors Detected".red().bold());
                    for error in errors {
                        // Add each semantic error to our error reporter
                        self.error_reporter.add_semantic_error(error);
                    }
                    return Err(());
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
                    
                    Ok(())
                }
            }
            Err(err) => {
                println!("{}", "Parser Error:".red().bold());
                // Add the parse error to our error reporter
                self.error_reporter.add_parse_error(&err);
                Err(())
            }
        }
    }
}
