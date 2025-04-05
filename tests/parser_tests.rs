#[cfg(test)]
mod parser_tests {
    use rust_compiler::parser::ast::{
        Declaration, Expression, Literal, Operator, Program, Statement, Type,
    };
    use rust_compiler::parser::parse_source;

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

        match &program.declarations[0] {
            Declaration::Variable(names, typ) => {
                assert_eq!(names.len(), 1);
                assert_eq!(names[0], "x");
                assert!(matches!(typ, Type::Int));
            }
            _ => panic!("Expected variable declaration"),
        }

        match &program.declarations[1] {
            Declaration::Variable(names, typ) => {
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

        match &program.declarations[0] {
            Declaration::Array(names, typ, size) => {
                assert_eq!(names.len(), 1);
                assert_eq!(names[0], "arr");
                assert!(matches!(typ, Type::Int));
                assert_eq!(*size, 10);
            }
            _ => panic!("Expected array declaration"),
        }

        match &program.declarations[1] {
            Declaration::Array(names, typ, size) => {
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
        match &program.declarations[0] {
            Declaration::VariableWithInit(names, typ, value) => {
                assert_eq!(names.len(), 1);
                assert_eq!(names[0], "x");
                assert!(matches!(typ, Type::Int));
                assert!(matches!(value, Expression::Literal(Literal::Int(10))));
            }
            _ => panic!("Expected variable declaration with initialization"),
        }

        // Test multiple variables initialization
        match &program.declarations[1] {
            Declaration::VariableWithInit(names, typ, value) => {
                assert_eq!(names.len(), 2);
                assert_eq!(names[0], "y");
                assert_eq!(names[1], "z");
                assert!(matches!(typ, Type::Float));
                assert!(matches!(value, Expression::Literal(Literal::Float(v)) if *v == 3.14));
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
        match &program.declarations[0] {
            Declaration::ArrayWithInit(names, typ, size, values) => {
                assert_eq!(names.len(), 1);
                assert_eq!(names[0], "arr");
                assert!(matches!(typ, Type::Int));
                assert_eq!(*size, 3);
                assert_eq!(values.len(), 3);

                assert!(matches!(&values[0], Expression::Literal(Literal::Int(1))));
                assert!(matches!(&values[1], Expression::Literal(Literal::Int(2))));
                assert!(matches!(&values[2], Expression::Literal(Literal::Int(3))));
            }
            _ => panic!("Expected array declaration with initialization"),
        }

        // Test multiple arrays initialization
        match &program.declarations[1] {
            Declaration::ArrayWithInit(names, typ, size, values) => {
                assert_eq!(names.len(), 2);
                assert_eq!(names[0], "matrix1");
                assert_eq!(names[1], "matrix2");
                assert!(matches!(typ, Type::Float));
                assert_eq!(*size, 2);
                assert_eq!(values.len(), 2);

                assert!(matches!(&values[0], Expression::Literal(Literal::Float(v)) if *v == 1.1));
                assert!(matches!(&values[1], Expression::Literal(Literal::Float(v)) if *v == 2.2));
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

        match &program.declarations[0] {
            Declaration::Constant(name, typ, value) => {
                assert_eq!(name, "Pi");
                assert!(matches!(typ, Type::Float));
                assert!(matches!(value, Literal::Float(v) if *v == 3.14));
            }
            _ => panic!("Expected constant declaration"),
        }

        match &program.declarations[1] {
            Declaration::Constant(name, typ, value) => {
                assert_eq!(name, "Max");
                assert!(matches!(typ, Type::Int));
                assert!(matches!(value, Literal::Int(v) if *v == 100));
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
            assert!(matches!(stmt, Statement::Assignment(_, _)));
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

        match &program.statements[0] {
            Statement::IfThen(condition, then_block) => {
                // Check condition is x > 10
                if let Expression::BinaryOp(left, op, right) = condition {
                    assert!(matches!(**left, Expression::Identifier(ref id) if id == "x"));
                    assert!(matches!(op, Operator::GreaterThan));
                    assert!(matches!(**right, Expression::Literal(Literal::Int(10))));
                } else {
                    panic!("Expected binary operation as condition");
                }

                // Check then block has one assignment
                assert_eq!(then_block.len(), 1);
                assert!(matches!(&then_block[0], Statement::Assignment(_, _)));
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

        match &program.statements[0] {
            Statement::IfThenElse(condition, then_block, else_block) => {
                // Check condition is x > 10
                if let Expression::BinaryOp(left, op, right) = condition {
                    assert!(matches!(**left, Expression::Identifier(ref id) if id == "x"));
                    assert!(matches!(op, Operator::GreaterThan));
                    assert!(matches!(**right, Expression::Literal(Literal::Int(10))));
                } else {
                    panic!("Expected binary operation as condition");
                }

                // Check then block has one assignment
                assert_eq!(then_block.len(), 1);
                assert!(matches!(&then_block[0], Statement::Assignment(_, _)));

                // Check else block has one assignment
                assert_eq!(else_block.len(), 1);
                assert!(matches!(&else_block[0], Statement::Assignment(_, _)));
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

        match &program.statements[0] {
            Statement::DoWhile(body, condition) => {
                // Check body has one assignment
                assert_eq!(body.len(), 1);
                assert!(matches!(&body[0], Statement::Assignment(_, _)));

                // Check condition is i < 10
                if let Expression::BinaryOp(left, op, right) = condition {
                    assert!(matches!(**left, Expression::Identifier(ref id) if id == "i"));
                    assert!(matches!(op, Operator::LessThan));
                    assert!(matches!(**right, Expression::Literal(Literal::Int(10))));
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

        match &program.statements[0] {
            Statement::For(var, from, to, step, body) => {
                // Check loop variable
                assert_eq!(var, "i");

                // Check from is 1
                assert!(matches!(from, Expression::Literal(Literal::Int(1))));

                // Check to is 100
                assert!(matches!(to, Expression::Literal(Literal::Int(100))));

                // Check step is 1
                assert!(matches!(step, Expression::Literal(Literal::Int(1))));

                // Check body has one assignment
                assert_eq!(body.len(), 1);
                assert!(matches!(&body[0], Statement::Assignment(_, _)));
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

        match &program.statements[0] {
            Statement::Input(var) => {
                assert!(matches!(var, Expression::Identifier(id) if id == "name"));
            }
            _ => panic!("Expected input statement"),
        }

        match &program.statements[1] {
            Statement::Output(exprs) => {
                assert_eq!(exprs.len(), 2);
                assert!(
                    matches!(&exprs[0], Expression::Literal(Literal::String(s)) if s == "Value is: ")
                );
                assert!(matches!(&exprs[1], Expression::Identifier(id) if id == "name"));
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
            assert!(matches!(stmt, Statement::Assignment(_, _)));
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
}
