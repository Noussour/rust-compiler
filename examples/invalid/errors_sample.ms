MainPrgm error_demo;
Var
  <!- Variable declarations with errors -!>
  let x, x: Int;  <!- Error: Double declaration of identifier 'x' -!>
  let arr: [Int; -5];  <!- Error: Array size must be positive -!>
  @define Const PI: Float = 3.14159;

BeginPg
{
  {-- This program demonstrates various semantic errors --}
  
  <!- Undeclared identifier error -!>
  y := 10;  <!- Error: 'y' is not declared -!>
  
  <!- Type incompatibility error -!>
  x := "hello";  <!- Error: Type mismatch - cannot assign string to Int -!>
  
  <!- Division by zero error -!>
  x := 100 / 0;  <!- Error: Division by zero -!>
  
  <!- Modifying a constant error -!>
  PI := 3.14;  <!- Error: Cannot modify a constant -!>
  
  <!- Array index out of bounds -!>
  arr[10] := 5;  <!- Error: Index out of bounds -!>
  
  <!- Mixed type expressions -!>
  x := 5 + 3.14;  <!- Error: Type mismatch in expression -!>
  
  <!- Incorrect argument count -!>
  input();  <!- Error: Missing argument for input -!>
  
  <!- Invalid conditions -!>
  if (10) then {  <!- Error: Condition must be a boolean expression -!>
    x := 5;
  }
  
  <!- Lexical errors -!>
  let invalid__id: Int;  <!- Error: Identifier contains consecutive underscores -!>
  let very_long_identifier_name: Int;  <!- Error: Identifier exceeds 14 character limit -!>
  
  <!- Missing semicolons -!>
  x := 10  <!- Error: Missing semicolon -!>
  
  <!- Improves syntax errors -!>
  if (x > 5) {  <!- Error: Missing 'then' keyword -!>
    x := 1;
  }
  
  <!- Unclosed string literal -!>
  output("This string is not closed);  <!- Error: Unclosed string literal -!>
}
EndPg;
