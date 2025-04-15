use crate::error_reporter::ErrorReportFormatter;
use crate::lexer::lexer_core::{TokenWithMetaData, tokenize};
use crate::parser::ast::Program;
use crate::parser::parser_core::parse;
use crate::semantics::{SemanticAnalyzer, symbol_table::SymbolKind};
use colored::*;
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

    pub fn run(&mut self) -> Result<(), i32> {
        println!("Compiling file: {}", self.file_path);
        // self.print_source_code();

        // Step 1: Lexical Analysis
        let tokens = match self.perform_lexical_analysis() {
            Ok(tokens) => tokens,
            Err(exit_code) => return Err(exit_code),
        };

        // Step 2: Syntax Analysis
        let ast = match self.perform_syntax_analysis(tokens) {
            Ok(ast) => ast,
            Err(exit_code) => return Err(exit_code),
        };

        // Step 3: Semantic Analysis
        // if let Err(exit_code) = self.perform_semantic_analysis(&ast) {
        //     return Err(exit_code);
        // }
        Ok(())
    }

    fn perform_lexical_analysis(&mut self) -> Result<Vec<TokenWithMetaData>, i32> {
        println!("{}: ", "Lexical Analysis".bold().underline());
        // Tokenize the source code and capture lexical errors
        let (valid_tokens, errors) = tokenize(&self.source_code);

        // Check for lexical errors
        if !errors.is_empty() {
            println!("{}", "Lexical Errors Detected:".red().bold());
            ErrorReportFormatter::print_errors(&errors, Some(&self.source_code));
            return Err(1);
        }

        // self.print_tokens(&valid_tokens);
        println!(
            "{}",
            "Lexical analysis completed successfully.".green().bold()
        );
        Ok(valid_tokens)
    }

    fn perform_syntax_analysis(
        &mut self,
        tokens: Vec<TokenWithMetaData>,
    ) -> Result<crate::parser::ast::Program, i32> {
        println!("\n{} :", "Syntax Analysis".bold().underline());
        println!("{} :", "Parsing".bold().underline());

        // Parse tokens into an AST
        match parse(tokens, &self.source_code) {
            Ok(program) => {
                self.print_ast(&program);
                println!("{}", "Parsing completed successfully.".green().bold());
                Ok(program)
            }
            Err(parse_error) => {
                println!("{}", "Parser Error Detected:".red().bold());
                ErrorReportFormatter::print_errors(&[parse_error], Some(&self.source_code));
                return Err(1);
            }
        }
    }

    fn perform_semantic_analysis(
        &mut self,
        program: &crate::parser::ast::Program,
    ) -> Result<(), i32> {
        println!("\n{}", "Semantic Analysis:".bold().underline());

        // Create analyzer with source code for span-to-line/column conversion
        let mut analyzer = SemanticAnalyzer::new_with_source_code(self.source_code.clone());
        analyzer.analyze(program);

        // Check for semantic errors
        let semantic_errors = analyzer.get_errors();
        if !semantic_errors.is_empty() {
            println!("{}", "Semantic Errors Detected:".red().bold());
            ErrorReportFormatter::print_errors(&semantic_errors, Some(&self.source_code));
            Err(1)
        } else {
            println!("{}", "analysis completed successfully.".green());
            self.print_symbol_table(&analyzer);
            Ok(())
        }
    }

    
    fn print_source_code(&self) {
        println!("{}", "Source code:".bold().underline());
        println!("{}\n", self.source_code);
    }
    
    fn print_tokens(&self, tokens: &[TokenWithMetaData]) {
        println!("{}", "Tokens:".bold().underline());
        for token_with_pos in tokens {
            let token_name = format!("{:?}", token_with_pos.kind).green();
            let token_value = token_with_pos.value.yellow();
            let position = format!(
                "Line {}, Col {}",
                token_with_pos.line, token_with_pos.column
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

    fn print_ast(&self, ast: &Program) {
        println!("{}", "AST:".green());
        println!("{:#?}", ast);
    }

    fn print_symbol_table(&self, analyzer: &SemanticAnalyzer) {
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
