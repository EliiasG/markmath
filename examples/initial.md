### This is an initial example of mm code 
In mm expressions are rendered as "expr = result", i.e ^5\*3^ becomes 5\*3 = 15, all expressions have a result unit - in the prior case it was simply the 'None' unit.  

There 4 types of units:  
- The special 'None' unit '(expr)None'
  - This means that the unit of the value is irrelevant.  
- Implicit units 
  - This is unit is the result of applying operators to defined units.  
  - The compiler asks for and saves the result unit of applying operators to expressions of "a{op}b", where a and b are expressions with result unit of defined or implicit, and {op} is an operator.
- Literal units '(expr)"Unit Name"'
  - This a unit that is given a name "on the spot".  
- Defined units '(expr)unit_name'
  - This is a unit that is defined by a short name, and should be given a display name when compiling.  

The resulting unit of a expression is the following:
- Any of the expressions shown above can override the resulting unit of an expression.  
- All functions result in the 'None' unit.  
- When both operands of an operator are of unit 'None' or literal, the result unit of the operator will be 'None'
- When one operand of an operator is of unit 'None', the result unit of the operation will be equal to the unit of the other operand.  
- When one operand is of an operator is a literal unit, and the other is defined or undefined, the result unit of the operation will be equal to the unit of the other operand.  
- When 

In mm variables are defined by expressions, and variable assignments are "invisible" when expressions are rendered. Formally, the expression "vname=exp" is rendered as "exp".  

Normally when variables are referenced they will render as a constant with the result value and unit, however if a ! is added before the variable name it will be rendered as an expression.  

An expression can be rendered as 


Speed of the snail: ^ speed = (7mm/100)m/14sec^  
