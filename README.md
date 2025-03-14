# MiniSoft Compiler Project

A compiler implementation for the MiniSoft language using [Lalrpop](https://github.com/lalrpop/lalrpop) and [Logos](https://github.com/maciejhirsz/logos) in Rust.

## Overview

This project implements a complete compiler for the MiniSoft language, including lexical analysis, syntax analysis, semantic analysis, and intermediate code generation. The compiler also includes robust error handling and a symbol table management system.

## MiniSoft Language

MiniSoft is a simple programming language with the following features:

- Variable and constant declarations
- Integer and float data types
- Arrays
- Control structures (conditionals and loops)
- Input/output operations
- Arithmetic, logical, and comparison operators

## Project Structure

```
minisoft-compiler/
├── Cargo.toml
├── README.md
├── examples/
│   ├── valid/
│   │   └── sample_program.ms
│   └── invalid/
│       └── errors_sample.ms
├── src/
│   ├── main.rs             # Entry point
│   ├── compiler.rs         # Compiler orchestration
│   ├── lexer/
│   │   ├── mod.rs          # Lexer module exports
│   │   ├── token.rs        # Token definitions
│   │   └── lexer.rs        # Logos lexer implementation
│   ├── parser/
│   │   ├── mod.rs          # Parser module exports
│   │   ├── ast.rs          # Abstract Syntax Tree definitions
│   │   ├── grammar.lalrpop # LALRPOP grammar definition
│   │   └── error.rs        # Parser error handling
│   ├── semantics/
│   │   ├── mod.rs          # Semantics module exports
│   │   ├── analyzer.rs     # Semantic analyzer
│   │   ├── symbol_table.rs # Symbol table implementation
│   │   └── error.rs        # Semantic error handling
│   ├── codegen/
│   │   ├── mod.rs          # Code generation module exports
│   │   ├── quadruple.rs    # Quadruple representation
│   │   └── generator.rs    # Intermediate code generator
│   └── error/
│       ├── mod.rs          # Error module exports
│       └── reporter.rs     # Error reporting utilities
└── tests/
    ├── lexer_tests.rs
    ├── parser_tests.rs
    ├── semantic_tests.rs
    └── integration_tests.rs
```

## Features

- **Lexical Analysis**: Tokenizes MiniSoft source code using Logos
- **Syntax Analysis**: Parses tokens into an Abstract Syntax Tree (AST) using LALRPOP
- **Semantic Analysis**: Performs type checking and other semantic validations
- **Symbol Table**: Manages identifiers and their associated information
- **Error Handling**: Reports errors with line and column information
- **Intermediate Code Generation**: Generates quadruples for the input program

## Semantic Error Checking

The compiler detects the following semantic errors:

- Undeclared identifiers
- Double-declared identifiers
- Type incompatibilities
- Division by zero (for constant expressions)
- Attempts to modify constant values
- Array index out of bounds

## Building and Running

```bash
# Build the project
cargo build

# Run on a sample file
cargo run -- examples/valid/sample_program.ms

# Run tests
cargo test
```
