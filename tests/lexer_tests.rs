#[cfg(test)]
mod lexer_tests {
    use logos::Logos;
    use rust_compiler::lexer::token::Token;

    #[test]
    fn test_basic_tokens() {
        let mut lexer = Token::lexer("MainPrgm Var BeginPg EndPg");
        assert_eq!(lexer.next(), Some(Ok(Token::MainPrgm)));
        assert_eq!(lexer.next(), Some(Ok(Token::Var)));
        assert_eq!(lexer.next(), Some(Ok(Token::BeginPg)));
        assert_eq!(lexer.next(), Some(Ok(Token::EndPg)));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_literals() {
        let mut lexer = Token::lexer("123 (+456) (-789) 45.67 (+12.34) (-56.78) \"hello world\"");
        assert_eq!(lexer.next(), Some(Ok(Token::IntLiteral(123))));
        assert_eq!(lexer.next(), Some(Ok(Token::IntLiteral(456))));
        assert_eq!(lexer.next(), Some(Ok(Token::IntLiteral(-789))));
        assert_eq!(lexer.next(), Some(Ok(Token::FloatLiteral(45.67))));
        assert_eq!(lexer.next(), Some(Ok(Token::FloatLiteral(12.34))));
        assert_eq!(lexer.next(), Some(Ok(Token::FloatLiteral(-56.78))));
        assert_eq!(
            lexer.next(),
            Some(Ok(Token::StringLiteral("hello world".to_string())))
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Token::lexer("variable x123 test_var");
        assert_eq!(
            lexer.next(),
            Some(Ok(Token::Identifier("variable".to_string())))
        );
        assert_eq!(
            lexer.next(),
            Some(Ok(Token::Identifier("x123".to_string())))
        );
        assert_eq!(
            lexer.next(),
            Some(Ok(Token::Identifier("test_var".to_string())))
        );
    }

    #[test]
    fn test_operators() {
        let mut lexer = Token::lexer("+ - * / > < >= <= == != := = AND OR !");
        assert_eq!(lexer.next(), Some(Ok(Token::Plus)));
        assert_eq!(lexer.next(), Some(Ok(Token::Minus)));
        assert_eq!(lexer.next(), Some(Ok(Token::Multiply)));
        assert_eq!(lexer.next(), Some(Ok(Token::Divide)));
        assert_eq!(lexer.next(), Some(Ok(Token::GreaterThan)));
        assert_eq!(lexer.next(), Some(Ok(Token::LessThan)));
        assert_eq!(lexer.next(), Some(Ok(Token::GreaterEqual)));
        assert_eq!(lexer.next(), Some(Ok(Token::LessEqual)));
        assert_eq!(lexer.next(), Some(Ok(Token::Equal)));
        assert_eq!(lexer.next(), Some(Ok(Token::NotEqual)));
        assert_eq!(lexer.next(), Some(Ok(Token::Assign)));
        assert_eq!(lexer.next(), Some(Ok(Token::Equals)));
        assert_eq!(lexer.next(), Some(Ok(Token::And)));
        assert_eq!(lexer.next(), Some(Ok(Token::Or)));
        assert_eq!(lexer.next(), Some(Ok(Token::Not)));
    }

    #[test]
    fn test_comments() {
        let mut lexer = Token::lexer("a <!- commented text -!> b {-- another comment --} c");
        assert_eq!(lexer.next(), Some(Ok(Token::Identifier("a".to_string()))));
        assert_eq!(lexer.next(), Some(Ok(Token::Identifier("b".to_string()))));
        assert_eq!(lexer.next(), Some(Ok(Token::Identifier("c".to_string()))));
    }

    #[test]
    fn test_invalid_identifiers() {
        // Test identifiers exceeding 14 chars
        let mut lexer = Token::lexer("thisidentifieristoolong");
        assert_eq!(lexer.next(), Some(Err(())));

        // Test consecutive underscores
        let mut lexer = Token::lexer("invalid__id");
        assert_eq!(lexer.next(), Some(Err(())));

        // Test finishing with an underscore
        let mut lexer = Token::lexer("invalid_");
        assert_eq!(lexer.next(), Some(Err(())));

        // Test integer literal out of range
        let mut lexer = Token::lexer("32768");
        assert_eq!(lexer.next(), Some(Err(())));
        let mut lexer = Token::lexer("(-32769)");
        assert_eq!(lexer.next(), Some(Err(())));
    }

    #[test]
    fn test_complete_program() {
        use std::fs;

        let test_file_path = "examples/valid/sample_program.ms";
        let input = fs::read_to_string(test_file_path).expect("Failed to read test file");

        let lexer = Token::lexer(&input);
        let tokens: Vec<_> = lexer.collect();

        assert!(tokens.iter().all(|t| t.is_ok()));
    }

    #[test]
    fn test_empty_input() {
        let mut lexer = Token::lexer("");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_only_comments() {
        let mut lexer = Token::lexer("{-- comment --} <!- another -!>");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_mixed_valid_and_invalid() {
        let mut lexer = Token::lexer("valid1 invalid__id 12345");
        assert_eq!(
            lexer.next(),
            Some(Ok(Token::Identifier("valid1".to_string())))
        );
        assert_eq!(lexer.next(), Some(Err(()))); // invalid__id
        assert_eq!(lexer.next(), Some(Ok(Token::IntLiteral(12345))));
        assert_eq!(lexer.next(), None);
    }
}
