MainPrgm Semanticerror;
Var
  let counter: Int;
  let average: Float = 0.0;
  @define Const Max_value: Int = 100;
  let numbers: [Int; 5] = {1, 2, 3, 4, 5};
  let result: Int;
  

  let counter: Float;
  
BeginPg
{

  total := 10;
  

  average := "not a number";
  
  if (counter + 5) then {
    output("This will cause a type error");
  }
  
  result := average + counter;
  
  Max_value := 200;
  
  result := 10 / 0;
  
  result := numbers[10];
  
  result := (numbers[1] * 2) + (true AND false);
  
  undeclared_var := Max_value / 0 + numbers[10];
  
  if (counter AND average) then {
    output("This will cause another type error");
  }
  
  numbers[0] := average;
  
  result := numbers;
  
  some_array[5] := 10;
}
EndPg;