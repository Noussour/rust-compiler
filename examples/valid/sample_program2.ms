MainPrgm Factorial;
Var
  let n: Int;
  let result: Float = 4.2;
  let i: Int = (-10);
  let A: [Float; 5] = {1.0, 2.0, 3.0, 4.0, 5.4};
BeginPg
{
  n := 5;
  result := 1.0;

  A[2] := 4.2;
  
  for i from 1 to n step 1 {
    result := result * i;
  }
  
  output(result);  <!- Outputs: 120 -!>
}
EndPg;
