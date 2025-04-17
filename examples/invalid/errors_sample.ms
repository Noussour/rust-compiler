MainPrgm error_demo;
Var
  <!- Variable declarations with errors -!>
  let x,x  : Int = 0;  <!- Error: Double declaration of identifier 'x' -!>
  let y : Int = 5;
  let z : Int = 2000;

BeginPg
{
  x := 5 * 7 + 4 / (5-5);
  y := y / x;
  if (x+5) then {

  }
}
EndPg;