use colored::*;
use rust_compiler::compiler::Compiler;
use std::env;
use std::process;

fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // Check if a file path was provided
    if args.len() <= 1 {
        eprintln!("{}: Usage: {} <file_path>", "Error".red().bold(), args[0]);
        process::exit(1);
    }

    // Create a compiler instance with the provided file path
    let file_path = &args[1];
    match Compiler::new(file_path) {
        Ok(mut compiler) => {
            // Run the compiler and process the result
            match compiler.run() {
                Ok(_) => {
                    println!("{}", "✓ Compilation successful!".green().bold());
                    process::exit(0);
                }
                Err(exit_code) => {
                    eprintln!("{}", "✗ Compilation failed".red().bold());
                    process::exit(exit_code);
                }
            }
        }
        Err(error) => {
            eprintln!("{}: {}", "Error".red().bold(), error);
            process::exit(1);
        }
    }
}
