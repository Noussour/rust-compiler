use colored::*;
use rust_compiler::compiler::Compiler;
use clap::{Arg, Command};
use std::process;

fn main() {
    let matches = Command::new("rust-compiler")
        .version("1.0")
        .author("Your Name")
        .about("Compiles MiniSoft programming language")
        .arg(
            Arg::new("file")
                .help("Input file to compile")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let file_path = matches.get_one::<String>("file").unwrap();
    let verbose = matches.get_flag("verbose");

    match Compiler::new(file_path) {
        Ok(mut compiler) => {
            if verbose {
                println!("{}", "Verbose mode enabled".yellow().bold());
            }
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