MainPrgm L3_software;
Var
  <!- Variable declarations -!>
  let counter, sum, average: Int;
  let temperatures: [Float; 10];
  @define Const MAX_TEMP: Float = (+100.0);
  @define Const MIN_TEMP: Float = (-0.0);
  @define Const DEFAULT_TEMP: Float = 25.5;
  {--
    ASK THE PROF :
      - IF AFFECTATION OF A VARIABLE IN THE DECLARATION PART IS ALLOWED
  --}
  let testValue1: Int = (+42);
  let testArray1: [Float; 3] = { (-4.34), (+23.23), 19};

BeginPg
{
  {-- This is a sample program that calculates the average
     of a series of temperature readings --}
  
  sum := 0;
  
  <!- Input phase -!>
  for i from 0 to 9 step 1 {
    output("Enter temperature reading #", i+1, ": ");
    input(temperatures[i]);
    
    <!- Validate temperature range -!>
    if (temperatures[i] > MAX_TEMP) then {
      output("Warning: Temperature exceeds maximum value!");
      temperatures[i] := MAX_TEMP;
    } else {
      if (temperatures[i] < (-10.5)) then {
        output("Warning: Temperature well below minimum value!");
        temperatures[i] := MIN_TEMP;
      }
    }
    
    sum := sum + temperatures[i];
  }
  
  <!- Calculate average -!>
  average := sum / 10;
  
  <!- Output results -!>
  output("Temperature readings:");
  counter := (-1);  <!- Start with -1 and increment first -!>
  do {
    counter := counter + 1;
    output("Reading #", counter+1, ": ", temperatures[counter]);
  } while (counter < (+9));
  
  output("Average temperature: ", average);
  
  <!- Check if average is within safe range -!>
  if ((average >= (+50.0)) AND (average <= (+75.0))) then {
    output("Average temperature is within safe operating range.");
  } else {
    output("Warning: Average temperature is outside safe operating range!");
  }
}
EndPg;
