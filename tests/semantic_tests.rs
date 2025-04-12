#[cfg(test)]
mod semantic_tests {
    use rust_compiler::parser::parser_core::parse_source;
    use rust_compiler::semantics::analyzer_core::SemanticAnalyzer;
    use rust_compiler::semantics::error::SemanticError;
    use std::collections::HashMap;

    /// Helper function to analyze code semantically and return errors
    fn analyze_test(source: &str) -> Vec<SemanticError> {
        // First parse the code to get an AST
        let program = match parse_source(source) {
            Ok(program) => program,
            Err(e) => panic!("Parse error: {}", e),
        };

        // Create a semantic analyzer with empty position info
        let mut analyzer = SemanticAnalyzer::new_with_positions(HashMap::new());

        // Analyze the program
        analyzer.analyze(&program);

        // Return the errors detected
        analyzer.get_errors().to_vec()
    }

    /// Helper to check if errors match expected patterns
    fn contains_error_of_type(errors: &[SemanticError], error_type: &str) -> bool {
        errors.iter().any(|e| {
            let error_str = format!("{:?}", e);
            error_str.contains(error_type)
        })
    }

    #[test]
    fn test_valid_program() {
        let source = r#"
            MainPrgm test;
            Var
            let x, y: Int;
            let z: Float;
            
            BeginPg
            {
                x := (+10);
                y := x + (+5);
                z := 3.14;
            }
            EndPg;
        "#;

        let errors = analyze_test(source);
        assert!(
            errors.is_empty(),
            "Expected no errors, but found: {:?}",
            errors
        );
    }

    #[test]
    fn test_undeclared_identifier() {
        let source = r#"
            MainPrgm test;
            Var
            let x: Int;
            
            BeginPg
            {
                y := (+10); <!- y is not declared -!>
            }
            EndPg;
        "#;

        let errors = analyze_test(source);
        assert!(!errors.is_empty(), "Expected errors, but found none");
        assert!(
            contains_error_of_type(&errors, "UndeclaredIdentifier"),
            "Expected undeclared identifier error, but found: {:?}",
            errors
        );
    }

    #[test]
    fn test_duplicate_declaration() {
        let source = r#"
            MainPrgm test;
            Var
            let x: Int;
            let x: Float; <!- Duplicate declaration -!>
            
            BeginPg
            {
                x := (+10);
            }
            EndPg;
        "#;

        let errors = analyze_test(source);
        assert!(!errors.is_empty(), "Expected errors, but found none");
        assert!(
            contains_error_of_type(&errors, "DuplicateDeclaration"),
            "Expected duplicate declaration error, but found: {:?}",
            errors
        );
    }

    #[test]
    fn test_type_mismatch() {
        let source = r#"
            MainPrgm test;
            Var
            let x: Int;
            let y: Float;
            
            BeginPg
            {
                x := y; <!- Type mismatch: Int := Float -!>
            }
            EndPg;
        "#;

        let errors = analyze_test(source);
        assert!(!errors.is_empty(), "Expected errors, but found none");
        assert!(
            contains_error_of_type(&errors, "TypeMismatch"),
            "Expected type mismatch error, but found: {:?}",
            errors
        );
    }

    #[test]
    fn test_constant_modification() {
        let source = r#"
            MainPrgm test;
            Var
            @define Const Pi: Float = 3.14;
            
            BeginPg
            {
                Pi := 3.14159; <!- Cannot modify constant -!>
            }
            EndPg;
        "#;

        let errors = analyze_test(source);
        assert!(!errors.is_empty(), "Expected errors, but found none");
        assert!(
            contains_error_of_type(&errors, "ConstantModification"),
            "Expected constant modification error, but found: {:?}",
            errors
        );
    }

    #[test]
    fn test_array_index_out_of_bounds() {
        let source = r#"
            MainPrgm test;
            Var
            let arr: [Int; 5];
            
            BeginPg
            {
                arr[5] := (+10); <!- Index out of bounds (valid indices are 0-4) -!>
            }
            EndPg;
        "#;

        let errors = analyze_test(source);
        assert!(!errors.is_empty(), "Expected errors, but found none");
        assert!(
            contains_error_of_type(&errors, "ArrayIndexOutOfBounds"),
            "Expected array index out of bounds error, but found: {:?}",
            errors
        );
    }

    #[test]
    fn test_division_by_zero() {
        let source = r#"
            MainPrgm test;
            Var
            let x: Int;
            
            BeginPg
            {
                x := (+10) / 0; <!- Division by zero -!>
            }
            EndPg;
        "#;

        let errors = analyze_test(source);
        assert!(!errors.is_empty(), "Expected errors, but found none");
        assert!(
            contains_error_of_type(&errors, "DivisionByZero"),
            "Expected division by zero error, but found: {:?}",
            errors
        );
    }

    #[test]
    fn test_non_array_indexing() {
        let source = r#"
            MainPrgm test;
            Var
            let x: Int;
            
            BeginPg
            {
                x[0] := (+10); <!- x is not an array -!>
            }
            EndPg;
        "#;

        let errors = analyze_test(source);
        assert!(!errors.is_empty(), "Expected errors, but found none");
        assert!(
            contains_error_of_type(&errors, "Cannot index non-array"),
            "Expected non-array indexing error, but found: {:?}",
            errors
        );
    }

    #[test]
    fn test_invalid_array_index_type() {
        let source = r#"
            MainPrgm test;
            Var
            let arr: [Int; 5];
            let idx: Float;
            
            BeginPg
            {
                idx := 1.5;
                arr[idx] := (+10); <!- Array index must be an integer -!>
            }
            EndPg;
        "#;

        let errors = analyze_test(source);
        assert!(!errors.is_empty(), "Expected errors, but found none");
        assert!(
            contains_error_of_type(&errors, "TypeMismatch"),
            "Expected type mismatch error for array index, but found: {:?}",
            errors
        );
    }

    #[test]
    fn test_invalid_if_condition() {
        let source = r#"
            MainPrgm test;
            Var
            let x: Float;
            
            BeginPg
            {
                x := 3.14;
                if (x) then { <!- x is not a boolean expression -!>
                    x := 2.71;
                }
            }
            EndPg;
        "#;

        let errors = analyze_test(source);
        assert!(!errors.is_empty(), "Expected errors, but found none");
        // This should cause a type error since Float is not valid as a boolean condition
    }

    #[test]
    fn test_invalid_for_loop_variable() {
        let source = r#"
            MainPrgm test;
            Var
            let i: Float; <!- Loop variable should be Int -!>
            
            BeginPg
            {
                for i from 1 to 10 step 1 {
                    i := i + 0.5; <!- Invalid: modifying loop variable with wrong type -!>
                }
            }
            EndPg;
        "#;

        let errors = analyze_test(source);
        assert!(!errors.is_empty(), "Expected errors, but found none");
        assert!(
            contains_error_of_type(&errors, "TypeMismatch"),
            "Expected type mismatch error for for-loop variable, but found: {:?}",
            errors
        );
    }

    #[test]
    fn test_mixed_type_operation() {
        let source = r#"
            MainPrgm test;
            Var
            let x: Int;
            let y: Float;
            let z: Int;
            
            BeginPg
            {
                {-- This should be allowed with implicit conversion --}
                z := x + y; <!- Int + Float should result in Float, can't assign to Int -!>
            }
            EndPg;
        "#;

        let errors = analyze_test(source);
        assert!(!errors.is_empty(), "Expected errors, but found none");
        assert!(
            contains_error_of_type(&errors, "TypeMismatch"),
            "Expected type mismatch error for mixed type operation, but found: {:?}",
            errors
        );
    }

    #[test]
    fn test_valid_array_initialization() {
        let source = r#"
            MainPrgm test;
            Var
            let arr: [Int; 3] = {1, 2, 3};
            
            BeginPg
            {
                arr[1] := (+42);
            }
            EndPg;
        "#;

        let errors = analyze_test(source);
        assert!(
            errors.is_empty(),
            "Expected no errors, but found: {:?}",
            errors
        );
    }

    #[test]
    fn test_incompatible_array_initialization() {
        let source = r#"
            MainPrgm test;
            Var
            let arr: [Int; 3] = {1.1, 2.2, 3.3}; <!- Float values for Int array -!>
            
            BeginPg
            {
                arr[1] := (+42);
            }
            EndPg;
        "#;

        let errors = analyze_test(source);
        assert!(!errors.is_empty(), "Expected errors, but found none");
        assert!(
            contains_error_of_type(&errors, "TypeMismatch"),
            "Expected type mismatch error for array initialization, but found: {:?}",
            errors
        );
    }

    #[test]
    fn test_multiple_errors() {
        let source = r#"
            MainPrgm test;
            Var
            let x: Int;
            let x: Float; <!- Error 1: Duplicate declaration -!>
            
            BeginPg
            {
                y := (+10); <!- Error 2: Undeclared identifier -!>
            }
            EndPg;
        "#;

        let errors = analyze_test(source);
        assert!(
            errors.len() >= 2,
            "Expected at least 2 errors, but found: {:?}",
            errors
        );
    }

    #[test]
    fn test_valid_complex_expressions() {
        let source = r#"
            MainPrgm test;
            Var
            let a, b, c: Int;
            let d, e: Float;
            
            BeginPg
            {
                a := (+5);
                b := (+10);
                c := (a + b) * (+2);
                d := 3.14;
                e := d * d + 2.0;
                
                if (a < b AND b > 0) then {
                    c := c + (+1);
                } else {
                    c := c - (+1);
                }
                
                for a from 1 to 10 step 2 {
                    b := b + a;
                }
            }
            EndPg;
        "#;

        let errors = analyze_test(source);
        assert!(
            errors.is_empty(),
            "Expected no errors, but found: {:?}",
            errors
        );
    }

    #[test]
    fn test_do_while_condition_error() {
        let source = r#"
            MainPrgm test;
            Var
            let x: Float;
            
            BeginPg
            {
                do {
                    x := x + 1.0;
                } while (x); <!- x should be in a boolean expression -!>
            }
            EndPg;
        "#;

        let errors = analyze_test(source);
        assert!(!errors.is_empty(), "Expected errors, but found none");
        // This should cause a type error for the condition
    }
}
