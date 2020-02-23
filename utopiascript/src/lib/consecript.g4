programm: function+|statement+;
functionDeclaration: fn identifier formalParameters (functionReturn)? functionBody ;
formalParameter : (' (ParameterDefine)?(,ParameterDefine)+ ')'
ParameterDefine: indentifer:typeType
functionReturn: '->' typeType
functionBody: statementBlock
statementBlock: '{' statement* '}'
statement: statementBlock|conditionBlockStmt|returnStatement|declaration| expressionStatement| assignmentStatement| ;
typeType: :i32|:i64|:f32|:f64|:bool|:string
FunctionCall : IDENTIFIER '(' expressionList? ')'
returnStatement: return (expressionStatement)?;
expressionList : addExpression (',' addExpression)*
conditionBlockStmt: if ConditionExpression statement|statementBlock (else statement|statementBlock)?;
conditionExpression: compareExpression|primarytype(&&|||conditionExpression)*
compareExpression: additiveExpression bop additiveExpression|primarytype::bool bop=(== , !=, >, <, >=, <=)
intDeclaration : 'let' Identifier:typeType ( '=' expressionStmt)? ';'
assignmentStatement : Identifier '=' expressionStmt
expressionStatement : additiveExpression ';'
additiveExpression -> multiplicativeExpress (+ multiplicativeExpress|- multiplicativeExpress)*
multiplicativeExpress -> primary(*multiplicativeExpress|/multiplicativeExpress)
primary -> functionCall|identifier|intLiteral| (additiveExpression)
intLiteral = intLiteral token
