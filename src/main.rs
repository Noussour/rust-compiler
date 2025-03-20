mod lexer;
mod parser;

use crate::lexer::Lexer;
use crate::parser::parse;
use colored::*;
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
        },
        Err(err) => {
            println!("{}", "Parser Error:".red());
            println!("{}", err);
        }
    }
}
