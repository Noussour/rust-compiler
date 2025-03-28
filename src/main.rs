use rust_compiler::compiler::Compiler;
use std::env;
use std::process;

fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // Check if a file path was provided
    if args.len() <= 1 {
        eprintln!("Usage: {} <file_path>", args[0]);
        process::exit(1);
    }

    // Create a compiler instance with the provided file path
    let file_path = &args[1];
    match Compiler::new(file_path) {
        Ok(compiler) => {
            // Run the compiler
            compiler.run();
        }
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1);
        }
    }
}
