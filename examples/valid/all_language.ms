MainPrgm Factorial;
Var
  let n, result: Int;
  let numbers: [Int; 5] = {1, 2, 3, 4, 5};
  let pi: Float = (-3.14);
  let initialized: [Float; 3] = {1.1, 2.2, 3.3};
  @define Const Max_value: Int = (+100);

  let i: Int = 0;
BeginPg
{
  output("Factorial Calculator");
  output("Enter a number: ");
  input(n);
  
  if (n > Max_value) then {
    output("Number too large!");
  } else {
    result := 1;
    
    do {
      result := result * n;
      n := n - 1;
    } while (n > 0);
    
    output("Factorial result: ", result);
  }
  
  for numbers[0] from 1 to 10 step 2 {
    output("Counter: ", numbers[0]);
  }
  
  if (pi == 3.14) then {
    output("Pi is approximately ", pi);
  }
  
  do {
    numbers[i] := numbers[i] * 2;
    i := i + 1;
  } while (i < 5);
  
  if ((numbers[0] > 0) AND (numbers[1] > 0)) then {
    output("First two numbers are positive");
  }
  
  if (!((numbers[0] + numbers[1]) <= 0) OR (pi >= 3.0)) then {
    output("Complex condition met");
  }
}
EndPg;
