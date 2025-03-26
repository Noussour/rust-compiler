mod lexer;
mod parser;
mod semantics;

use crate::lexer::lexer_core::TokenWithPosition;
use crate::lexer::Lexer;
use crate::parser::parse;
use crate::semantics::SemanticAnalyzer;
use colored::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::process;

fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // Parse arguments and read file content
    let source_code = if args.len() > 1 {
        match fs::read_to_string(&args[1]) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading file '{}': {}", args[1], e);
                process::exit(1);
            }
        }
    } else {
        eprintln!("Usage: {} <file_path>", args[0]);
        process::exit(1);
    };

    println!("{}", "Source code:".bold().underline());
    println!("{}\n", source_code);

    // Create lexer and tokenize the source
    let lexer = Lexer::new(&source_code);
    let tokens: Vec<_> = lexer.collect();

    // Build a position map for identifiers
    let mut position_map = HashMap::new();
    for token in &tokens {
        if let crate::lexer::token::Token::Identifier(name) = &token.token {
            position_map.insert(name.clone(), (token.position.line, token.position.column));
        }
    }

    // Display tokens
    println!("{}", "Tokens:".bold().underline());
    for token_with_pos in &tokens {
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
                        semantics::SymbolKind::Variable => "Variable".cyan(),
                        semantics::SymbolKind::Constant => "Constant".yellow(),
                        semantics::SymbolKind::Array(size) => format!("Array[{}]", size).magenta(),
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
