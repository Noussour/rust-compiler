MainPrgm SemanticErrorDemo;
Var
  let counter: Int;
  let average: Float = 0.0;
  @define Const Max_value: Int = 100;
  let numbers: [Int; 5] = {1, 2, 3, 4, 5};
  let result: Int;
  
  // Duplicate declaration - SemanticError::DuplicateDeclaration
  let counter: Float;
  
BeginPg
{
  // Undeclared identifier - SemanticError::UndeclaredIdentifier
  total := 10;
  
  // Type mismatch in assignment - SemanticError::TypeMismatch
  average := "not a number";
  
  // Type mismatch in condition - SemanticError::TypeMismatch
  if (counter + 5) then {
    output("This will cause a type error");
  }
  
  // Type mismatch in arithmetic - SemanticError::TypeMismatch
  result := average + counter;
  
  // Attempt to modify constant - SemanticError::ConstantModification
  Max_value := 200;
  
  // Division by zero - SemanticError::DivisionByZero
  result := 10 / 0;
  
  // Array index out of bounds - SemanticError::ArrayIndexOutOfBounds
  result := numbers[10];
  
  // Type mismatch in complex expression
  result := (numbers[1] * 2) + (true AND false);
  
  // Multiple errors in one line
  undeclared_var := Max_value / 0 + numbers[10];
  
  // Incompatible types in logical operation
  if (counter AND average) then {
    output("This will cause another type error");
  }
  
  // Writing to an array index with wrong type
  numbers[0] := average;
  
  // Using array name without index
  result := numbers;
  
  // Assigning to undeclared array index
  someArray[5] := 10;
}
EndPg;