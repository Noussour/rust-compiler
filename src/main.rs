mod lexer;

use crate::lexer::Lexer;

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

    println!("Source code:");
    println!("{}\n", source_code);

    // Create lexer and tokenize the source
    let lexer = Lexer::new(source_code);

    println!("Tokens:");
    for token_with_pos in lexer {
        println!(
            "{:?} (value: '{}') at Line {}, Column {} (span: {:?})",
            token_with_pos.token,
            token_with_pos.text, // Assuming you've added a 'text' field to TokenWithPosition
            token_with_pos.position.line,
            token_with_pos.position.column,
            token_with_pos.span
        );
    }
}
