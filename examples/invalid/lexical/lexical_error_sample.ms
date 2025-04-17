MainPrgm L3_software ;  
Var  
    <!- Partie déclaration -!>  
    let 2var :: Int ;          <!- Error: Identifier starts with a digit -!>  
    let my__var :: Int ;       <!- Error: Consecutive underscores -!>  
    let var_ :: Float ;        <!- Error: Trailing underscore -!>  
    let this_is_too_long_identifier :: Int ; < !- Error: Exceeds 14 characters -!>  
    let my$var :: Int ;        <!- Error: Invalid character '$' -!>  
    let large_num :: Int = 32768 ; <!- Error: Integer out of range (-32768 to 32767) -!>  
    let bad_float :: Float = 5.6.7 ; <!- Error: Multiple decimal points -!>  
    <!- Unterminated comment    <!- Error: Missing closing -!> -!>  
BeginPg  
{  
    x := 5 ^ 2 ;              <!- Error: Invalid operator '^' -!>  
}  
EndPg ;  