mod lexer;

use crate::lexer::Lexer;
use colored::*;

fn main() {
    // Example MiniSoft program
    let source_code = r#"
    MainPrgm example
    Var
        x: Int;
        y: Float;
    BeginPg
        x := 10;
        y := 3.14;
        if (x > 5) then
            output("x is greater than 5");
        else
            output("x is not greater than 5");
        endif;
    EndPg
    "#;

    println!("{}", "Source code:".bold().underline());
    println!("{}\n", source_code);

    // Create lexer and tokenize the source
    let lexer = Lexer::new(source_code);
    let tokens: Vec<_> = lexer.collect();

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
}
