use crate::lexer::{lexer_core::TokenWithPosition, token::Token, Lexer};
use crate::parser::parse;
use crate::semantics::{SemanticAnalyzer, SymbolKind};
use colored::*;
use std::collections::HashMap;
use std::fs;

pub struct Compiler {
    source_code: String,
    file_path: String,
}

impl Compiler {
    pub fn new(file_path: &str) -> Result<Self, String> {
        match fs::read_to_string(file_path) {
            Ok(content) => Ok(Self {
                source_code: content,
                file_path: file_path.to_string(),
            }),
            Err(e) => Err(format!("Error reading file '{}': {}", file_path, e)),
        }
    }

    pub fn run(&self) {
        println!("Compiling file: {}", self.file_path);
        self.print_source_code();
        let tokens = self.tokenize();
        self.print_tokens(&tokens);
        self.parse_and_analyze(tokens);
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

    fn parse_and_analyze(&self, tokens: Vec<TokenWithPosition>) {
        // Build a position map for identifiers
        let mut position_map = HashMap::new();
        for token in &tokens {
            if let Token::Identifier(name) = &token.token {
                position_map.insert(name.clone(), (token.position.line, token.position.column));
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
                    println!("{}", "Semantic Errors:".red());
                    for error in errors {
                        println!("- {}", error);
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
            Err(err) => {
                println!("{}", "Parser Error:".red());
                println!("{}", err);
            }
        }
    }
}
