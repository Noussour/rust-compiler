MainPrgm L3_software ;  
Var  
    let x : Int ;                <!- Error: Missing '::' in declaration -!>  
    let A : [Int ; 5] ;        <!- Error: Missing array size -!>  
    let B : [Float ; 10] ;     <!- Error: Missing closing bracket -!>  
BeginPg  
{  
    if (x > 5) then {              <!- Error: Missing 'then' keyword -!>  
        x := x + 1 ;         <!- Error: Missing semicolon after assignment -!>  
    }  
    for i from 1 to 10 step 1 {      <!- Error: Missing 'step' keyword -!>  
        output(i) ;  
    }  
    y := (5 + 3 * x);        <!- Error: Incomplete expression -!>  
}  
EndPg;               <!- Error: Missing semicolon after EndPg -!>  
Extratoken