#[cfg(test)]
mod parser_tests {
    use rust_compiler::parser::ast::{
        DeclarationKind, ExpressionKind, LiteralKind, Operator, Program, StatementKind, Type,
    };
    use rust_compiler::parser::parser_core::parse_source;

    /// Helper function to parse a source string and return the AST
    fn parse_test(source: &str) -> Program {
        match parse_source(source) {
            Ok(program) => program,
            Err(e) => panic!("Parse error: {}", e),
        }
    }

    #[test]
    fn test_minimal_program() {
        let source = "MainPrgm test ; Var BeginPg { } EndPg ;";
        let program = parse_test(source);

        assert_eq!(program.name, "test");
        assert!(program.declarations.is_empty());
        assert!(program.statements.is_empty());
    }

    #[test]
    fn test_variable_declarations() {
        let source = "
            MainPrgm test ;
            Var
            let x : Int ;
            let y, z : Float ;
            BeginPg { } EndPg ;
        ";

        let program = parse_test(source);

        assert_eq!(program.declarations.len(), 2);

        match &program.declarations[0].node {
            DeclarationKind::Variable(names, typ) => {
                assert_eq!(names.len(), 1);
                assert_eq!(names[0], "x");
                assert!(matches!(typ, Type::Int));
            }
            _ => panic!("Expected variable declaration"),
        }

        match &program.declarations[1].node {
            DeclarationKind::Variable(names, typ) => {
                assert_eq!(names.len(), 2);
                assert_eq!(names[0], "y");
                assert_eq!(names[1], "z");
                assert!(matches!(typ, Type::Float));
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_array_declarations() {
        let source = "
            MainPrgm test ;
            Var
            let arr : [Int; 10] ;
            let matrix1, matrix2 : [Float; 100] ;
            BeginPg { } EndPg ;
        ";

        let program = parse_test(source);

        assert_eq!(program.declarations.len(), 2);

        match &program.declarations[0].node {
            DeclarationKind::Array(names, typ, size) => {
                assert_eq!(names.len(), 1);
                assert_eq!(names[0], "arr");
                assert!(matches!(typ, Type::Int));
                assert_eq!(*size, 10);
            }
            _ => panic!("Expected array declaration"),
        }

        match &program.declarations[1].node {
            DeclarationKind::Array(names, typ, size) => {
                assert_eq!(names.len(), 2);
                assert_eq!(names[0], "matrix1");
                assert_eq!(names[1], "matrix2");
                assert!(matches!(typ, Type::Float));
                assert_eq!(*size, 100);
            }
            _ => panic!("Expected array declaration"),
        }
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_variable_with_initialization() {
        let source = "
        MainPrgm test ;
        Var
        let x : Int = 10 ;
        let y, z : Float = 3.14 ;
        BeginPg { } EndPg ;
    ";

        let program = parse_test(source);

        assert_eq!(program.declarations.len(), 2);

        // Test single variable initialization
        match &program.declarations[0].node {
            DeclarationKind::VariableWithInit(names, typ, value) => {
                assert_eq!(names.len(), 1);
                assert_eq!(names[0], "x");
                assert!(matches!(typ, Type::Int));
                assert!(
                    matches!(&value.node, ExpressionKind::Literal(lit) if matches!(&lit.node, LiteralKind::Int(10)))
                );
            }
            _ => panic!("Expected variable declaration with initialization"),
        }

        // Test multiple variables initialization
        match &program.declarations[1].node {
            DeclarationKind::VariableWithInit(names, typ, value) => {
                assert_eq!(names.len(), 2);
                assert_eq!(names[0], "y");
                assert_eq!(names[1], "z");
                assert!(matches!(typ, Type::Float));
                assert!(
                    matches!(&value.node, ExpressionKind::Literal(lit) if matches!(&lit.node, LiteralKind::Float(v) if (*v - 3.14).abs() < 0.0001))
                );
            }
            _ => panic!("Expected variable declaration with initialization"),
        }
    }

    #[test]
    fn test_array_with_initialization() {
        let source = "
        MainPrgm test ;
        Var
        let arr : [Int; 3] = {1, 2, 3} ;
        let matrix1, matrix2 : [Float; 2] = {1.1, 2.2} ;
        BeginPg { } EndPg ;
    ";

        let program = parse_test(source);

        assert_eq!(program.declarations.len(), 2);

        // Test array initialization
        match &program.declarations[0].node {
            DeclarationKind::ArrayWithInit(names, typ, size, values) => {
                assert_eq!(names.len(), 1);
                assert_eq!(names[0], "arr");
                assert!(matches!(typ, Type::Int));
                assert_eq!(*size, 3);
                assert_eq!(values.len(), 3);

                assert!(
                    matches!(&values[0].node, ExpressionKind::Literal(lit) if matches!(&lit.node, LiteralKind::Int(1)))
                );
                assert!(
                    matches!(&values[1].node, ExpressionKind::Literal(lit) if matches!(&lit.node, LiteralKind::Int(2)))
                );
                assert!(
                    matches!(&values[2].node, ExpressionKind::Literal(lit) if matches!(&lit.node, LiteralKind::Int(3)))
                );
            }
            _ => panic!("Expected array declaration with initialization"),
        }

        // Test multiple arrays initialization
        match &program.declarations[1].node {
            DeclarationKind::ArrayWithInit(names, typ, size, values) => {
                assert_eq!(names.len(), 2);
                assert_eq!(names[0], "matrix1");
                assert_eq!(names[1], "matrix2");
                assert!(matches!(typ, Type::Float));
                assert_eq!(*size, 2);
                assert_eq!(values.len(), 2);

                assert!(
                    matches!(&values[0].node, ExpressionKind::Literal(lit) if matches!(&lit.node, LiteralKind::Float(v) if (*v - 1.1).abs() < 0.0001))
                );
                assert!(
                    matches!(&values[1].node, ExpressionKind::Literal(lit) if matches!(&lit.node, LiteralKind::Float(v) if (*v - 2.2).abs() < 0.0001))
                );
            }
            _ => panic!("Expected array declaration with initialization"),
        }
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_constant_declarations() {
        let source = "
            MainPrgm test ;
            Var
            @define Const Pi : Float = 3.14 ;
            @define Const Max : Int = 100 ;
            BeginPg { } EndPg ;
        ";

        let program = parse_test(source);

        assert_eq!(program.declarations.len(), 2);

        match &program.declarations[0].node {
            DeclarationKind::Constant(name, typ, value) => {
                assert_eq!(name, "Pi");
                assert!(matches!(typ, Type::Float));
                assert!(matches!(&value.node, LiteralKind::Float(v) if *v == 3.14));
            }
            _ => panic!("Expected constant declaration"),
        }

        match &program.declarations[1].node {
            DeclarationKind::Constant(name, typ, value) => {
                assert_eq!(name, "Max");
                assert!(matches!(typ, Type::Int));
                assert!(matches!(&value.node, LiteralKind::Int(v) if *v == 100));
            }
            _ => panic!("Expected constant declaration"),
        }
    }

    #[test]
    fn test_assignment_statements() {
        let source = "
            MainPrgm test ;
            Var
            let x, y : Int ;
            let arr : [Int; 5] ;
            BeginPg {
                x := 10 ;
                y := x + 5 ;
                arr[0] := 42 ;
            } EndPg ;
        ";

        let program = parse_test(source);

        assert_eq!(program.statements.len(), 3);

        // Check that they're all assignment statements
        for stmt in &program.statements {
            assert!(matches!(&stmt.node, StatementKind::Assignment(_, _)));
        }
    }

    #[test]
    fn test_if_statement() {
        let source = "
            MainPrgm test ;
            Var
            let x, max : Int ;
            BeginPg {
                if (x > 10) then {
                    max := x ;
                }
            } EndPg ;
        ";

        let program = parse_test(source);

        assert_eq!(program.statements.len(), 1);

        match &program.statements[0].node {
            StatementKind::IfThen(condition, then_block) => {
                // Check condition is x > 10
                if let ExpressionKind::BinaryOp(left, op, right) = &condition.node {
                    assert!(matches!(&left.node, ExpressionKind::Identifier(id) if id == "x"));
                    assert!(matches!(op, Operator::GreaterThan));
                    assert!(
                        matches!(&right.node, ExpressionKind::Literal(lit) if matches!(&lit.node, LiteralKind::Int(10)))
                    );
                } else {
                    panic!("Expected binary operation as condition");
                }

                // Check then block has one assignment
                assert_eq!(then_block.len(), 1);
                assert!(matches!(
                    &then_block[0].node,
                    StatementKind::Assignment(_, _)
                ));
            }
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_if_else_statement() {
        let source = "
            MainPrgm test ;
            Var
            let x, max : Int ;
            BeginPg {
                if (x > 10) then {
                    max := x ;
                } else {
                    max := 10 ;
                }
            } EndPg ;
        ";

        let program = parse_test(source);

        assert_eq!(program.statements.len(), 1);

        match &program.statements[0].node {
            StatementKind::IfThenElse(condition, then_block, else_block) => {
                // Check condition is x > 10
                if let ExpressionKind::BinaryOp(left, op, right) = &condition.node {
                    assert!(matches!(&left.node, ExpressionKind::Identifier(id) if id == "x"));
                    assert!(matches!(op, Operator::GreaterThan));
                    assert!(
                        matches!(&right.node, ExpressionKind::Literal(lit) if matches!(&lit.node, LiteralKind::Int(10)))
                    );
                } else {
                    panic!("Expected binary operation as condition");
                }

                // Check then block has one assignment
                assert_eq!(then_block.len(), 1);
                assert!(matches!(
                    &then_block[0].node,
                    StatementKind::Assignment(_, _)
                ));

                // Check else block has one assignment
                assert_eq!(else_block.len(), 1);
                assert!(matches!(
                    &else_block[0].node,
                    StatementKind::Assignment(_, _)
                ));
            }
            _ => panic!("Expected if-else statement"),
        }
    }

    #[test]
    fn test_do_while_loop() {
        let source = "
            MainPrgm test ;
            Var
            let i : Int ;
            BeginPg {
                do {
                    i := i + 1 ;
                } while (i < 10) ;
            } EndPg ;
        ";

        let program = parse_test(source);

        assert_eq!(program.statements.len(), 1);

        match &program.statements[0].node {
            StatementKind::DoWhile(body, condition) => {
                // Check body has one assignment
                assert_eq!(body.len(), 1);
                assert!(matches!(&body[0].node, StatementKind::Assignment(_, _)));

                // Check condition is i < 10
                if let ExpressionKind::BinaryOp(left, op, right) = &condition.node {
                    assert!(matches!(&left.node, ExpressionKind::Identifier(id) if id == "i"));
                    assert!(matches!(op, Operator::LessThan));
                    assert!(
                        matches!(&right.node, ExpressionKind::Literal(lit) if matches!(&lit.node, LiteralKind::Int(10)))
                    );
                } else {
                    panic!("Expected binary operation as condition");
                }
            }
            _ => panic!("Expected do-while statement"),
        }
    }

    #[test]
    fn test_for_loop() {
        let source = "
            MainPrgm test ;
            Var
            let sum, i : Int ;
            BeginPg {
                for i from 1 to 100 step 1 {
                    sum := sum + i ;
                }
            } EndPg ;
        ";

        let program = parse_test(source);

        assert_eq!(program.statements.len(), 1);

        match &program.statements[0].node {
            StatementKind::For(var, from, to, step, body) => {
                // Check loop variable: extract identifier from Located<ExpressionKind>
                if let ExpressionKind::Identifier(ident) = &var.node {
                    assert_eq!(ident, "i");
                } else {
                    panic!("Expected identifier for loop variable");
                }
                assert!(matches!(&from.node, ExpressionKind::Literal(lit) if
                    matches!(&lit.node, LiteralKind::Int(1))
                ));
                assert!(matches!(&to.node, ExpressionKind::Literal(lit) if
                    matches!(&lit.node, LiteralKind::Int(100))
                ));
                assert!(matches!(&step.node, ExpressionKind::Literal(lit) if
                    matches!(&lit.node, LiteralKind::Int(1))
                ));

                assert_eq!(body.len(), 1);
                assert!(matches!(&body[0].node, StatementKind::Assignment(_, _)));
            }
            _ => panic!("Expected for statement"),
        }
    }

    #[test]
    fn test_input_output() {
        let source = "
            MainPrgm test ;
            Var
            let name : Int ;
            BeginPg {
                input(name) ;
                output(\"Value is: \", name) ;
            } EndPg ;
        ";

        let program = parse_test(source);

        assert_eq!(program.statements.len(), 2);

        match &program.statements[0].node {
            StatementKind::Input(var) => {
                assert!(matches!(&var.node, ExpressionKind::Identifier(id) if id == "name"));
            }
            _ => panic!("Expected input statement"),
        }

        match &program.statements[1].node {
            StatementKind::Output(exprs) => {
                assert_eq!(exprs.len(), 2);
                assert!(
                    matches!(&exprs[0].node, ExpressionKind::Literal(lit) if matches!(&lit.node, LiteralKind::String(s) if s == "Value is: "))
                );
                assert!(matches!(&exprs[1].node, ExpressionKind::Identifier(id) if id == "name"));
            }
            _ => panic!("Expected output statement"),
        }
    }

    #[test]
    fn test_complex_expressions() {
        let source = "
            MainPrgm test ;
            Var
            let x, y, z : Int ;
            BeginPg {
                z := x + y * 2 ;
                z := (x + y) * 2 ;
                z := x > y AND y < 10 ;
                z := (x > y) OR (y != 5) ;
            } EndPg ;
        ";

        let program = parse_test(source);

        assert_eq!(program.statements.len(), 4);

        // All should be assignment statements
        for stmt in &program.statements {
            assert!(matches!(&stmt.node, StatementKind::Assignment(_, _)));
        }
    }

    #[test]
    #[should_panic(expected = "Parse error")]
    fn test_missing_semicolon() {
        let source = "
            MainPrgm test ;
            Var
            let x : Int
            BeginPg { } EndPg ;
        ";

        parse_test(source);
    }

    #[test]
    #[should_panic(expected = "Parse error")]
    fn test_missing_then() {
        let source = "
            MainPrgm test ;
            Var
            BeginPg {
                if (x > 10) {
                    x := 20 ;
                } else {
                    x := 10 ;
                }
            } EndPg ;
        ";

        parse_test(source);
    }

    #[test]
    #[should_panic(expected = "Parse error")]
    fn test_wrong_program_structure() {
        let source = "
            BeginPg
            let x : Int ;
            MainPrgm test ;
            { } 
            EndPg ;
        ";

        parse_test(source);
    }

    #[test]
    fn test_large_program() {
        use std::fs;

        let test_file_path = "examples/valid/sample_program.ms";
        let input = fs::read_to_string(test_file_path).expect("Failed to read test file");

        // Just check if parsing succeeds
        let program = parse_test(&input);
        assert_eq!(program.name, "L3_software");
    }

    #[test]
    fn test_empty_program() {
        let source = "";
        let result = parse_source(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_unexpected_token() {
        let source = "MainPrgm test ; Var let x : Int ; BeginPg { x := ; } EndPg ;";
        let result = parse_source(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_minimal_program_structure_valid() {
        let source = "
            MainPrgm L3_software ;
            Var
            BeginPg
            {
            }
            EndPg ;
        ";
        let program = parse_test(source);
        assert_eq!(program.name, "L3_software");
    }

    #[test]
    fn test_variable_declaration_simple_valid() {
        let source = "MainPrgm t; Var let x : Int ; BeginPg { } EndPg ;";
        let program = parse_test(source);
        assert!(program.declarations.iter().any(|d| matches!(&d.node, DeclarationKind::Variable(names, typ) if names.contains(&"x".to_string()) && matches!(typ, Type::Int))));
    }

    #[test]
    fn test_variable_declaration_array_valid() {
        let source = "MainPrgm t; Var let tableau : [Float ; 5] ; BeginPg { } EndPg ;";
        let program = parse_test(source);
        assert!(program.declarations.iter().any(|d| matches!(&d.node, DeclarationKind::Array(names, typ, size) if names.contains(&"tableau".to_string()) && matches!(typ, Type::Float) && *size == 5)));
    }

    #[test]
    fn test_variable_declaration_multiple_valid() {
        let source =
            "MainPrgm t; Var let a, b, c : Int ; let tab1, tab2 : [Int ; 10] ; BeginPg { } EndPg ;";
        let program = parse_test(source);
        assert!(program.declarations.iter().any(|d| matches!(&d.node, DeclarationKind::Variable(names, typ) if names.contains(&"a".to_string()) && matches!(typ, Type::Int))));
        assert!(program.declarations.iter().any(|d| matches!(&d.node, DeclarationKind::Array(names, typ, size) if names.contains(&"tab1".to_string()) && matches!(typ, Type::Int) && *size == 10)));
    }

    #[test]
    fn test_variable_declaration_missing_colon_invalid() {
        let source = "MainPrgm t; Var let erreur Int ; BeginPg { } EndPg ;";
        let result = parse_source(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_array_declaration_invalid_sizes() {
        let source = "MainPrgm t; Var let tab_erreur : [Int ; -1] ; let tab_erreur2 : [Float ; 0] ; let tab_erreur3 : [Int ; 2.5] ; BeginPg { } EndPg ;";
        let program = parse_test(source);
        // Semantic error, not parser error, so just check parsing succeeds
        assert_eq!(program.declarations.len(), 3);
    }

    #[test]
    fn test_identifier_invalid_lexical() {
        let sources = [
            "MainPrgm t; Var let 1abc : Int ; BeginPg { } EndPg ;",
            "MainPrgm t; Var let var- : Float ; BeginPg { } EndPg ;",
            "MainPrgm t; Var let var_ : Int ; BeginPg { } EndPg ;",
            "MainPrgm t; Var let var__valide : Float ; BeginPg { } EndPg ;",
            "MainPrgm t; Var let un_nom_de_variable_trop_long_depassant_14_caracteres : Int ; BeginPg { } EndPg ;",
        ];
        for src in sources {
            let result = parse_source(src);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_constant_declaration_valid() {
        let source = "MainPrgm t; Var @define Const PI : Float = 3.14 ; @define Const MAX_SIZE : Int = 100 ; BeginPg { } EndPg ;";
        let program = parse_test(source);
        assert!(program.declarations.iter().any(|d| matches!(&d.node, DeclarationKind::Constant(name, typ, _) if name == "PI" && matches!(typ, Type::Float))));
        assert!(program.declarations.iter().any(|d| matches!(&d.node, DeclarationKind::Constant(name, typ, _) if name == "MAX_SIZE" && matches!(typ, Type::Int))));
    }

    #[test]
    fn test_constant_declaration_missing_equal_invalid() {
        let source = "MainPrgm t; Var @define Const EULER : Float 2.718 ; BeginPg { } EndPg ;";
        let result = parse_source(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_constant_declaration_non_constant_value_invalid() {
        let source = "MainPrgm t; Var let variable : Int ; @define Const ERREUR : Int = variable ; BeginPg { } EndPg ;";
        let program = parse_test(source);
        // Semantic error, not parser error, so just check parsing succeeds
        assert!(program.declarations.len() >= 2);
    }

    #[test]
    fn test_type_usage_valid() {
        let source = "MainPrgm t; Var let entier : Int ; let reel : Float ; @define Const ZERO_INT : Int = 0 ; @define Const ZERO_FLOAT : Float = 0.0 ; BeginPg { } EndPg ;";
        let program = parse_test(source);
        assert!(program.declarations.len() >= 4);
    }

    #[test]
    fn test_type_unknown_invalid() {
        let source = "MainPrgm t; Var let inconnu : String ; BeginPg { } EndPg ;";
        let program = parse_test(source);
        // Semantic error, not parser error, so just check parsing succeeds
        assert!(program.declarations.len() == 1);
    }

    #[test]
    fn test_int_constants_valid() {
        let source = "MainPrgm t; Var let pos : Int = 123 ; let neg : Int = (-456) ; let zero : Int = 0 ; BeginPg { } EndPg ;";
        let program = parse_test(source);
        assert!(program.declarations.len() == 3);
    }

    #[test]
    fn test_int_constants_out_of_bounds_invalid() {
        let source = "MainPrgm t; Var let trop_grand : Int = 32768 ; let trop_petit : Int = (-32769) ; BeginPg { } EndPg ;";
        let program = parse_test(source);
        assert!(program.declarations.len() == 2);
    }

    #[test]
    fn test_float_constants_valid() {
        let source = "MainPrgm t; Var let pi_valide : Float = 3.14159 ; let neg_reel : Float = (-2.718) ; let pos_reel : Float = (+1.618) ; BeginPg { } EndPg ;";
        let program = parse_test(source);
        assert!(program.declarations.len() == 3);
    }

    #[test]
    fn test_float_constants_invalid_format() {
        let sources = [
            "MainPrgm t; Var let reel_erreur : Float = 3. ; BeginPg { } EndPg ;",
            "MainPrgm t; Var let reel_erreur2 : Float = .14 ; BeginPg { } EndPg ;",
            "MainPrgm t; Var let reel_erreur3 : Float = 314. ; BeginPg { } EndPg ;",
        ];
        for src in sources {
            let result = parse_source(src);
            assert!(result.is_err());
        }
    }
}
