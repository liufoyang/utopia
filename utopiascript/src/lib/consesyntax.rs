use super::conselexer::TokenList;
use super::conselexer::Token;
use super::conselexer::Token_Type;
use std::collections::HashMap;


#[derive(Debug)]
#[derive(Clone)]
pub enum AST_Node_Type {
    Programm,           //程序入口，根节点
    Statement,        // 程序语句
    Function,        // 函数
    FormalParameters, //函数参数定义
    FunctionReturn,  // 函数返回定义
    FunctionBody,    // 函数体
    FunctionCall,        // 函数调用
    expressionList,   // 表达式列表
    Declaration,     //变量声明
    ExpressionStmt,     //表达式语句，即表达式后面跟个分号
    AssignmentStmt,     //赋值语句
    ConditionBlockStmt,  // if condition stmt else stmt
    ReturnStmt,  // return expression
    StatementBlock,  // 程序块

    Multiplicative,     //乘法表达式
    Additive,           //加法表达式

    Primary,            //基础表达式

    Identifier,         //标识符
    IntLiteral,          //整型字面量
    DoubleLiteral,          //双浮点型字面量
    StringLiteral,          //整型字面量
    ConditionExpression,   // 条件表达式
    CompareExpression,   // 比较表达式
    ParameterDefine,       // 单个参数定义
    TypeType,               // 类型声明节点

}

impl PartialEq for AST_Node_Type {

    fn eq(&self, other: &Self) -> bool {
        let a_value = self.clone() as u32;
        let b_value = other.clone() as u32;

        return a_value == b_value;
    }

}

#[derive(Debug)]
pub struct AST_Node {

    node_type:AST_Node_Type,
    children:Vec<AST_Node>,
    value:String,
    id: i32,
}

impl AST_Node {
    pub fn new(_node_type:AST_Node_Type, _value:&str) -> AST_Node{
        let node = AST_Node {
            node_type:_node_type,
            children:Vec::new(),
            value:_value.to_string(),
            id:1
        };

        return node;
    }

    pub fn addChild(&mut self, _node:AST_Node) {

        self.children.push(_node);
    }

    pub fn get_type(&self) -> AST_Node_Type {
        return self.node_type.clone();
    }

    pub fn getChildren(&self) -> &Vec<AST_Node> {
         return &self.children;
    }

    pub fn get_id(&self) -> i32{
        return self.id.clone();
    }

    pub fn get_value(&self) -> &str{
        return self.value.as_str();
    }

    fn set_id(&mut self, id:i32) {
        self.id = id;
    }

}

/// new a promgram AST tree EBNF is  programm: function+|statement+;
/// create by liufoyang
pub fn syntaxParse(tokens:&mut TokenList) -> Option<AST_Node> {

    let mut node = AST_Node::new(AST_Node_Type::Programm, "conse_ast_root");

    let mut childNodeResult = funtions(tokens);
    while childNodeResult.is_some() {
        node.addChild(childNodeResult.unwrap());

        childNodeResult = funtions(tokens);
    }

    if node.getChildren().len() ==0 {
        childNodeResult = statement(tokens);
        while childNodeResult.is_some() {
            node.addChild(childNodeResult.unwrap());

            childNodeResult = statement(tokens);
        }
    }

    // set id for node;
    let mut  node_id = 1;
    init_node_id(&mut node, &mut node_id);
    return Some(node);
}

fn init_node_id(node:&mut AST_Node, node_id:&mut i32) {
    node.set_id(node_id.clone());
    *node_id +=1;

    if node.getChildren().len() == 0 {
        return;
    }

    let leng = node.getChildren().len();
    let index = 0;
    for i in 0..leng{
        let mut child_node = node.children.get_mut(i).unwrap();
        init_node_id(child_node, node_id);
    }
}

/// new a promgram AST tree EBNF is  functionDeclaration: fn identifier formalParameters (functionReturn)? functionBody ;
/// create by liufoyang
pub fn funtions(tokens:&mut TokenList) -> Option<AST_Node> {
    let position = tokens.getPosition();
    let fnTokenResult = tokens.next();

    if fnTokenResult.is_none() {
        tokens.setPosition(position);
        return None;
    }

    let fnToken = fnTokenResult.unwrap();
    // not function define
    if fnToken.get_type() != Token_Type::Function {
        println!("not function token {}", fnToken.to_string());
        tokens.setPosition(position);
        return None;
    }

    let funtionIdentiferResult = identifier(tokens);

    if funtionIdentiferResult.is_none() {
        tokens.setPosition(position);
        panic!("functionDeclaration error at {}, it must fn identifier formalParameters (functionReturn)? functionBody", fnToken.to_string());
    }

    let funtionIdentiferNode = funtionIdentiferResult.unwrap();

    let mut functionNode = AST_Node::new(AST_Node_Type::Function, funtionIdentiferNode.value.as_str());

    // formalParameters
    let formalParametersResult = formalParameters(tokens);
    if formalParametersResult.is_none() {
        tokens.setPosition(position);
        panic!("functionDeclaration error at {}, it must fn identifier formalParameters (functionReturn)? functionBody, but formalParameters not found", fnToken.to_string());
    }
    functionNode.addChild(formalParametersResult.unwrap());

    let functionReturnResult = functionReturn(tokens);
    if functionReturnResult.is_some() {
        println!("found function return");
        functionNode.addChild(functionReturnResult.unwrap());
    }

    let functionBodyResult = functionBody(tokens);
    if functionBodyResult.is_none() {
        tokens.setPosition(position);
        panic!("functionDeclaration error at {}, it must fn identifier formalParameters (functionReturn)? functionBody, but functionBodyResult not found", fnToken.to_string());
    }
    functionNode.addChild(functionBodyResult.unwrap());

    return Some(functionNode);

}

/// new a promgram AST tree EBNF is  formalParameter : (' (ParameterDefine)?(,ParameterDefine)+ ')'
/// create by liufoyang
pub fn formalParameters(tokens:&mut TokenList) -> Option<AST_Node> {
    let position = tokens.getPosition();
    let leftTokenResult = tokens.next();

    if(leftTokenResult.is_none()) {
        tokens.setPosition(position);
        return None;
    }

    let leftToken = leftTokenResult.unwrap();

    if leftToken.get_type() != Token_Type::LeftParen{
        tokens.setPosition(position);
        return None;
    }

    let mut formalNode = AST_Node::new(AST_Node_Type::FormalParameters, "()");
    let mut paramDefineResult = parameterDefine(tokens);
    while paramDefineResult.is_some() {
        formalNode.addChild(paramDefineResult.unwrap());

        let commaTokenResult = tokens.preRead();
        if(commaTokenResult.is_some()) {
            let commaToken = commaTokenResult.unwrap();

            if commaToken.get_type() == Token_Type::Comma {
                tokens.next();
            } else {
                break;
            }
        } else {
            break;
        }

        paramDefineResult = parameterDefine(tokens);
        if paramDefineResult.is_none() {
            tokens.setPosition(position);
            panic!("formalParameter must (ParameterDefine)?(,ParameterDefine)+ but");
        }
    }

    let rightTokenResult = tokens.next();
    if rightTokenResult.is_none() {
        tokens.setPosition(position);
        panic!("formalParameter must (ParameterDefine)?(,ParameterDefine)+ but lack rightparen");
    }

    let rightToken = rightTokenResult.unwrap();

    if rightToken.get_type() != Token_Type::RightParen{
        tokens.setPosition(position);
        return None;
    }


    println!("formalParameters {:?}", formalNode);
    return Some(formalNode);

}

/// new a promgram AST tree EBNF is  ParameterDefine: indentifer:typeType
/// create by liufoyang
pub fn parameterDefine(tokens:&mut TokenList) -> Option<AST_Node> {
    let position = tokens.getPosition();

    let tokenResult = tokens.preRead();
    if tokenResult.is_none() {
        return None;
    }

    let token = tokenResult.unwrap();
    let identifierNodeResult = identifier(tokens);
    if(identifierNodeResult.is_none()) {
        tokens.setPosition(position);
        return None;
    }

    let mut paramDefindNode = AST_Node::new(AST_Node_Type::ParameterDefine, "");
    paramDefindNode.addChild(identifierNodeResult.unwrap());

    let typeTypeNodeResult = typeType(tokens);

    if typeTypeNodeResult.is_none() {
        panic!("ParameterDefine error, type not found at {}", token.to_string());
    }

    paramDefindNode.addChild(typeTypeNodeResult.unwrap());

    return Some(paramDefindNode);
}

/// new a functionReturn AST tree EBNF is  functionReturn: '->' typeType
/// create by liufoyang
pub fn functionReturn(tokens:&mut TokenList) -> Option<AST_Node> {

    let position = tokens.getPosition();
    let returnBeginToken = tokens.next();

    if returnBeginToken.is_none() {
        tokens.setPosition(position);
        return None;
    }

    let returnToken = returnBeginToken.unwrap();
    if returnToken.get_type()!= Token_Type::FuncationReturn {
        tokens.setPosition(position);
        return None;
    }

    let typeTypeNodeResult = typeType(tokens);
    if typeTypeNodeResult.is_none() {
        tokens.setPosition(position);
        panic!("functionReturn error, type not found at {}", returnToken.to_string())
    }


    let mut functionReturnNode = AST_Node::new(AST_Node_Type::FunctionReturn, "->");
    functionReturnNode.addChild(typeTypeNodeResult.unwrap());

    return Some(functionReturnNode);
}

/// new a functionBody AST tree EBNF is  functionBody: statementBlock
/// create by liufoyang
pub fn functionBody(tokens:&mut TokenList ) -> Option<AST_Node> {

    let statementBlockResult = statementBlock(tokens);

    if statementBlockResult.is_none() {
        return None;
    }

    let mut functionBodyNode = AST_Node::new(AST_Node_Type::FunctionBody,"");
    functionBodyNode.addChild(statementBlockResult.unwrap());

    return Some(functionBodyNode);
}

/// new a statementBlock AST tree EBNF is  statementBlock: '{' statement* '}'
/// create by liufoyang
pub fn statementBlock (tokens:&mut TokenList ) -> Option<AST_Node> {
    let position = tokens.getPosition();
    let leftbraceTokenResult = tokens.next();

    if leftbraceTokenResult.is_none() {
        tokens.setPosition(position);
        return None;
    }

    let letBraceToken = leftbraceTokenResult.unwrap();

    if letBraceToken.get_type()!=Token_Type::LeftBrace {
        tokens.setPosition(position);
        return None;
    }

    let mut blockStatmentNode = AST_Node::new(AST_Node_Type::StatementBlock, "block");

    let mut statmentResult = statement(tokens);
    while statmentResult.is_some() {
        //println!("add stmt to block");
        blockStatmentNode.addChild(statmentResult.unwrap());

        let nextToken = tokens.preRead();
        if(nextToken.is_some() && nextToken.unwrap().get_type() == Token_Type::RightBrace) {
            break;
        }
        statmentResult = statement(tokens);
    }

    println!("new block stmt {:?}", blockStatmentNode);

    let rightbraceTokenResult = tokens.next();

    if rightbraceTokenResult.is_none() {
        tokens.setPosition(position);
        panic!("statementBlock error, statementBlock must LeftParen statement rightParen, but rightParen lack at {}",position);
    }
    let rightBraceToken = rightbraceTokenResult.unwrap();

    if rightBraceToken.get_type()!=Token_Type::RightBrace {
        tokens.setPosition(position);
        panic!("statementBlock error, statementBlock must LeftParen statement rightParen, but lack at {}", rightBraceToken.to_string());
    }

    return Some(blockStatmentNode);

}

/// new a typeType AST tree EBNF is  typeType: :i32|:i64|:f32|:f64|:bool|:string
/// create by liufoyang
pub fn typeType (tokens:&mut TokenList ) -> Option<AST_Node> {
    let position = tokens.getPosition();

    let primaryTypeResult = tokens.next();
    if primaryTypeResult.is_none() {
        tokens.setPosition(position);
        return None;
    }

    let primaryType = primaryTypeResult.unwrap();

    let mut typeNode = AST_Node::new(AST_Node_Type::TypeType, "");
    if primaryType.get_type() == Token_Type::i32 {
        typeNode.value = "i32".to_string();
    } else if primaryType.get_type() == Token_Type::i64 {
        typeNode.value = "i64".to_string();
    } else if primaryType.get_type() == Token_Type::f32 {
        typeNode.value = "f32".to_string();
    } else if primaryType.get_type() == Token_Type::f64 {
        typeNode.value = "f64".to_string();
    } else if primaryType.get_type() == Token_Type::bool {
        typeNode.value = "bool".to_string();
    } else if primaryType.get_type() == Token_Type::string {
        typeNode.value = "string".to_string();
    } else {
        tokens.setPosition(position);
        return None;
    }

    return Some(typeNode);

}

/// new a promgram AST tree EBNF is  statement: statementBlock|conditionBlockStmt|returnStatement|declaration| expressionStatement| assignmentStatement| ;
/// create by liufoyang
fn statement(tokens:&mut TokenList) -> Option<AST_Node> {
    if tokens.preRead().is_none()  {
        return None;
    }

    let statementBlock = statementBlock(tokens);
    if statementBlock.is_some() {
        return statementBlock;
    }

    let conditonStmt = condtionBlockStmt(tokens);
    if conditonStmt.is_some() {
        return conditonStmt;
    }

    let returnstatment = returnStatement(tokens);
    if returnstatment.is_some() {
        return returnstatment;
    }

//    let funcationCallStmt = functionCall(tokens);
//    if funcationCallStmt.is_some()  {
//        return funcationCallStmt;
//    }

    let declarationNode = declaration(tokens);
    if declarationNode.is_some() {
        return declarationNode;
    }

    let assignmentStmtNode = assignmentStmt(tokens);
    if assignmentStmtNode.is_some() {
        return assignmentStmtNode;
    }

    let expressionNode = expressionStmt(tokens);
    if expressionNode.is_some() {
        return expressionNode;
    }


    return None;

}

/// new a promgram AST tree EBNF is  FunctionCall : IDENTIFIER '(' expressionList? ')'
/// create by liufoyang
fn functionCall (tokens:&mut TokenList) -> Option<AST_Node>{

    let position = tokens.getPosition();

    let identiferNode = identifier(tokens);

    if identiferNode.is_none() {
        return None;
    }

    let leftParenToken = tokens.next();
    if leftParenToken.is_none() || leftParenToken.unwrap().get_type()!=Token_Type::LeftParen {
        tokens.setPosition(position);
        return None;
    }

    let mut funcationCallNode = AST_Node::new(AST_Node_Type::FunctionCall, "");
    funcationCallNode.addChild(identiferNode.unwrap());

    let expressListNode = expressionList(tokens);

    if expressListNode.is_some() {
        funcationCallNode.addChild(expressListNode.unwrap());
    }

    let rightParenToken = tokens.next();
    if rightParenToken.is_none() || rightParenToken.unwrap().get_type()!=Token_Type::RightParen {
        tokens.setPosition(position);
        return None;
    }

    return Some(funcationCallNode);

}


/// new a promgram AST tree EBNF is  expressionList : addExpression (',' addExpression)*
/// create by liufoyang
fn expressionList (tokens:&mut TokenList) -> Option<AST_Node>{
    let position = tokens.getPosition();

    let expressionNode = additive(tokens);

    if expressionNode.is_none() {
        return None;
    }

    let mut expressListNode = AST_Node::new(AST_Node_Type::expressionList, "");
    expressListNode.addChild(expressionNode.unwrap());

    let mut commsToken = tokens.preRead();


    while (commsToken.is_some()) {
        let comms = commsToken.unwrap();
        if comms.get_type() == Token_Type::Comma {
            tokens.next();
            let expressionNode = additive(tokens);

            if expressionNode.is_none() {
                tokens.setPosition(position);
                panic!("expressionList lack expression after , at {}", comms.to_string());
            }

            expressListNode.addChild(expressionNode.unwrap());
        } else {
            break;
        }
        commsToken = tokens.preRead();
    }

    return Some(expressListNode);

}

/// new a promgram AST tree EBNF is  conditionBlockStmt: if ConditionExpression statement|statementBlock (else statement|statementBlock)?;
/// create by liufoyang
fn condtionBlockStmt (tokens:&mut TokenList) -> Option<AST_Node>{
    let position = tokens.getPosition();
    let ifToken = tokens.next().unwrap();

    if ifToken.get_type() == Token_Type::KeyWord{
        if ifToken.get_text() == "if" {
            //println!("has if token");
            let mut stmtNode = AST_Node::new(AST_Node_Type::ConditionBlockStmt, "if");

            let conditionNode = conditionExpression(tokens);

            if conditionNode.is_none() {
                tokens.setPosition(position);
                panic!("stmt  error, condition stmt must if conditionExpression at token {}", ifToken.to_string());
                return None;
            }
            //println!("has condition expression");

            stmtNode.addChild(conditionNode.unwrap());

            let ifBlockStatm = statement(tokens);
            if ifBlockStatm.is_none() {
                tokens.setPosition(position);
                panic!("stmt  error, condition stmt must if conditionExpression lack stmt");
                return None;
            }

            //println!("has stmt expression");
            stmtNode.addChild(ifBlockStatm.unwrap());

            let elseTokenResult = tokens.preRead();
            if elseTokenResult.is_none() {
                return Some(stmtNode);
            }

            let elseToken = tokens.next().unwrap();
            if elseToken.get_type() != Token_Type::KeyWord || elseToken.get_text() != "else"{
                return Some(stmtNode);
            }
            //println!("has else token");
            tokens.next();

            let elseBlockStmt = statement(tokens);
            if elseBlockStmt.is_none() {
                tokens.setPosition(position);
                panic!("stmt  error, condition stmt must  else lack stmt at token {}", elseToken.to_string());
                return None;
            }
            //println!("has else stmt");
            stmtNode.addChild(elseBlockStmt.unwrap());
            return Some(stmtNode);
        }

    }

    //println!("no condition block");
    tokens.setPosition(position);
    return None;

}

/// new a promgram AST tree EBNF is  returnStatement: return (expressionStatement)?;
/// create by liufoyang
fn returnStatement (tokens:&mut TokenList) -> Option<AST_Node>{
    let position = tokens.getPosition();
    let returnToken = tokens.next().unwrap();

    if returnToken.get_type() == Token_Type::KeyWord{
        if returnToken.get_text() == "return" {

            let mut returnStmtNode = AST_Node::new(AST_Node_Type::ReturnStmt, "return");

            // pre
            let mut nextToken = tokens.preRead().unwrap();

            if(nextToken.get_type() == Token_Type::SemiColon) {
                // no expressionStatement;
                tokens.next();
            } else {
                let expressionNode = expressionStmt(tokens);

                if expressionNode.is_none() {
                    tokens.setPosition(position);
                    panic!("returnStatement  error, returnStatement  must  return (expressionStatement)?");
                }

                returnStmtNode.addChild(expressionNode.unwrap());
            }

            nextToken = tokens.preRead().unwrap();
//
//            if(nextToken.get_type() == Token_Type::SemiColon ||nextToken.get_type() == Token_Type::Enter) {
//                // no expressionStatement;
//                tokens.next();
//            } else {
//                panic!("returnStatement  error, returnStatement  must  return (expressionStatement)? but at {}", nextToken.to_string());
//            }
            println!("end return {}", nextToken.to_string());
            return Some(returnStmtNode);
        }
    }

    tokens.setPosition(position);
    return None;
}

/// new a promgram AST tree EBNF is  conditionExpression: compareExpression|primarytype(&&|||conditionExpression)
/// create by liufoyang
fn conditionExpression(tokens:&mut TokenList) -> Option<AST_Node> {

    let position = tokens.getPosition();


    let mut conditionNode = AST_Node::new(AST_Node_Type::ConditionExpression, "");

    let expressNodeResult = compareExpression(tokens);
    if expressNodeResult.is_some() {
        conditionNode.addChild(expressNodeResult.unwrap());
        return Some(conditionNode);
    }

    let primaryNodeResult = primary(tokens);
    if primaryNodeResult.is_some() {
        conditionNode.addChild(primaryNodeResult.unwrap());
        return Some(conditionNode);
    }

    return None;

}

/// new a promgram AST tree EBNF is  compareExpression: additiveExpression bop additiveExpression|primarytype::bool bop=(== , !=, >, <, >=, <=)
/// create by liufoyang
fn compareExpression(tokens:&mut TokenList) -> Option<AST_Node> {

    let position = tokens.getPosition();

    let expressNodeResult1 = additive(tokens);

    if expressNodeResult1.is_none() {
        tokens.setPosition(position);
        panic!("compareExpression  error, compareExpression  must  additiveExpression bop additiveExpression bop=(== , !=, >, <, >=, <=)");
        return None;
    }

    let conditionTokenResult = tokens.preRead();
    if conditionTokenResult.is_none() {
        tokens.setPosition(position);
        panic!("compareExpression  error, compareExpression  must  contain == or !=");
        return None;
    }

    let conditionToken = conditionTokenResult.unwrap();

    if conditionToken.get_type()!= Token_Type::NOTEQ && conditionToken.get_type()!= Token_Type::EQ && conditionToken.get_type()!= Token_Type::GE
        && conditionToken.get_type()!= Token_Type::GT&& conditionToken.get_type()!= Token_Type::LT && conditionToken.get_type()!= Token_Type::LE{
        tokens.setPosition(position);
        panic!("compareExpression  error, compareExpression  must  contain == or !=");
        return None;
    }

    tokens.next();
    let expressNodeResult2 = additive(tokens);

    if expressNodeResult2.is_none() {
        tokens.setPosition(position);
        panic!("compareExpression  error, additiveExpression bop additiveExpression bop=(== , !=, >, <, >=, <=)");
        return None;
    }

    let expressNode1 = expressNodeResult1.unwrap();
    let expressNode2 = expressNodeResult2.unwrap();

    let mut conditionNode = AST_Node::new(AST_Node_Type::CompareExpression, conditionToken.get_text());

    conditionNode.addChild(expressNode1);
    conditionNode.addChild(expressNode2);

    return Some(conditionNode);

}

/// new a promgram AST tree EBNF is  intDeclaration : 'let' Identifier:typeType ( '=' expressionStmt)? ';';
/// create by liufoyang
fn declaration(tokens:&mut TokenList) -> Option<AST_Node> {

    let position = tokens.getPosition();
    let tokenResult = tokens.next();

    if tokenResult.is_none() {
        return None;
    }

    let token = tokenResult.unwrap();
    if token.get_type() == Token_Type::KeyWord {
        if token.get_text() == "let" {
            //let mut node = AST_Node::new(AST_Node_Type::Declaration, "let");
            let identifierNode = identifier(tokens);

            if identifierNode.is_none() {
                tokens.setPosition(position);
                panic!("declaration var error, declaration must be let identifier");
                return None;
            }

            let typeTypeNode = typeType(tokens);
            if typeTypeNode.is_none() {
                tokens.setPosition(position);
                panic!("declaration var error, declaration must be let identifier:type");
                return None;
            }

            let mut decareNode = AST_Node::new(AST_Node_Type::Declaration, "let");
            decareNode.addChild(identifierNode.unwrap());
            decareNode.addChild(typeTypeNode.unwrap());

            let mut assignTokenResult = tokens.next();
            if assignTokenResult.is_none() {
                tokens.setPosition(position);
                panic!("declaration var error, declaration must be end with ;");
                return None;
            }

            let assignToken = assignTokenResult.unwrap();
            if assignToken.get_type() != Token_Type::Assignment{
                // end with ;
                if assignToken.get_type() != Token_Type::SemiColon && assignToken.get_type() != Token_Type::Enter {
                    tokens.setPosition(position);
                    panic!("declaration var error, declaration must be end with ; or newline");
                    return None;
                }
                return Some(decareNode);
            }

            let expressNode = expressionStmt(tokens);

            if expressNode.is_some() {

                decareNode.addChild(expressNode.unwrap());
            } else {
                tokens.setPosition(position);
                panic!("declaration var error, declaration must be let identifier=expression");
                return None;
            }

            return Some(decareNode);

        } else {
            tokens.setPosition(position);
            return None;
        }
    }

    tokens.setPosition(position);
    return None;
}

/// new a promgram AST tree EBNF is assignmentStatement : Identifier '=' additiveExpression | stringLiteral';';
/// create by liufoyang
pub fn assignmentStmt(tokens:&mut TokenList) -> Option<AST_Node> {
    let position = tokens.getPosition();

    let indentiferTokenResult = tokens.next();
    if indentiferTokenResult.is_none() {
        return None;
    }

    let indentiferToken = indentiferTokenResult.unwrap();
    if indentiferToken.get_type() != Token_Type::Identifier {
        return None;
    }

    let assignTokenResult = tokens.next();
    if assignTokenResult.is_none(){
        tokens.setPosition(position);
        return None;
    }

    let assignToken = assignTokenResult.unwrap();
    if assignToken.get_type() != Token_Type::Assignment {
        tokens.setPosition(position);
        return None;
    }

    let mut node = AST_Node::new(AST_Node_Type::AssignmentStmt, indentiferToken.get_text());

    let additiveNode = additive(tokens);

    let mut has_value = false;
    if additiveNode.is_some() {
        node.addChild(additiveNode.unwrap());
        has_value = true;
    }

    let strIntValueNode = stringLiteral(tokens);
    if strIntValueNode.is_some() {
        node.addChild(strIntValueNode.unwrap());
        has_value = true;
    }

    if !has_value {
        tokens.setPosition(position);
        panic!("assignmentStatement error, assignmentStatement must Identifier = expression; {}",position);
        return None;
    }


    let simTokenResult = tokens.next();

    if simTokenResult.is_none() {
        return Some(node);
    }
    let endToken =  simTokenResult.unwrap();
    if endToken.get_type() != Token_Type::SemiColon && endToken.get_type() != Token_Type::Enter{
        tokens.setPosition(position);
        panic!("assignmentStatement error, assignmentStatement must end with ; or newline but {}", &endToken.to_string());
        return None;
    }

    return Some(node);


}

/// new a promgram AST tree EBNF is expressionStatement : additiveExpression ';';
/// create by liufoyang
fn expressionStmt(tokens:&mut TokenList) -> Option<AST_Node> {
    let position = tokens.getPosition();
    let additiveNode = additive(tokens);

    if additiveNode.is_none() {
        return None;
    }

    let mut endTokenResult = tokens.next();
    if endTokenResult.is_none() {
        tokens.setPosition(position);
        panic!("expressionStatement error, expressionStatement must be end with ;");
        return None;
    }

    let endToken= endTokenResult.unwrap();
    if endToken.get_type() != Token_Type::SemiColon
        && endToken.get_type() != Token_Type::Enter
        && endToken.get_type() != Token_Type::Comma{
        tokens.setPosition(position);
        panic!("expressionStatement error, expressionStatement must be end with ; or , or newline but with {} {}", endToken.to_string(), position);
        return None;
    }

    return additiveNode;


}

/// new a promgram AST tree the EBNF is additiveExpression -> multiplicativeExpress (+ multiplicativeExpress|- multiplicativeExpress)*
/// create by liufoyang
fn additive(tokens:&mut TokenList) -> Option<AST_Node> {

    let position = tokens.getPosition();

    let multipNode = multiplicative(tokens);

    if multipNode.is_none() {
        return None;
    }

    let mut node = multipNode.unwrap();

    while (true) {
        let nextTokenResult = tokens.preRead();
        if nextTokenResult.is_none() {

            break;
        }

        let plusToken = nextTokenResult.unwrap();
        if plusToken.get_type() != Token_Type::Plus && plusToken.get_type() != Token_Type::Minus {
            break;
        }

        let mut plusNode = AST_Node::new(AST_Node_Type::Additive, plusToken.get_text());

        tokens.next();

        let nextMultipNode = multiplicative(tokens);
        if nextMultipNode.is_none() {
            tokens.setPosition(position);
            panic!("additiveExpression error, additiveExpression must +/- with multiplicativeExpress ;");
            return None;
        }

        plusNode.addChild(node);
        plusNode.addChild(nextMultipNode.unwrap());
        node = plusNode;
    }

    return Some(node);

}

/// new a promgram AST tree EBNF multiplicativeExpress -> primary(*multiplicativeExpress|/multiplicativeExpress)
/// create by liufoyang
fn multiplicative(tokens:&mut TokenList) -> Option<AST_Node> {
    let position = tokens.getPosition();

    let primaryNode = primary(tokens);

    if primaryNode.is_none() {
        return None;
    }

    let mut node = primaryNode.unwrap();
    let nextTokenResult = tokens.preRead();
    if nextTokenResult.is_none() {

        return Some(node);
    }

    let mutiToken = nextTokenResult.unwrap();
    if mutiToken.get_type() != Token_Type::Star && mutiToken.get_type() != Token_Type::Slash {
        return Some(node);
    }

    tokens.next();
    let mut multi_node = AST_Node::new(AST_Node_Type::Multiplicative, mutiToken.get_text());

    multi_node.addChild(node);

    let next_muti_node = multiplicative(tokens);

    if next_muti_node.is_none() {
        tokens.setPosition(position);
        panic!("multiplicativeExpress error, multiplicativeExpress must * or / with multiplicativeExpress ;");
        return None;
    }

    multi_node.addChild(next_muti_node.unwrap());

    return Some(multi_node);
}

/// new a promgram AST tree EBNF primary -> functionCall|identifier|intLiteral| (additiveExpression)
/// create by liufoyang
fn primary(tokens:&mut TokenList) -> Option<AST_Node> {
    let position = tokens.getPosition();

    let intLiteralNode = intLiteral(tokens);
    if intLiteralNode.is_some() {
        return intLiteralNode;
    }

    let functionCallNode = functionCall(tokens);
    if functionCallNode.is_some() {
        return functionCallNode;
    }

    let identifierNode = identifier(tokens);
    if identifierNode.is_some() {
        return identifierNode;
    }

    let leftParenTokenResult = tokens.next();
    if leftParenTokenResult.is_none() {
        tokens.setPosition(position);
        return None;
    }

    let leftParenToken = leftParenTokenResult.unwrap();
    if leftParenToken.get_type() != Token_Type::LeftParen {
        tokens.setPosition(position);
        return None;
    }

    let expressResult = additive(tokens);
    if expressResult.is_none() {
        tokens.setPosition(position);
        return None;
    }

    let priNode = expressResult.unwrap();

    let rightParenTokenResult = tokens.next();
    if rightParenTokenResult.is_none() {
        tokens.setPosition(position);
        return None;
    }

    let rightParenToken = rightParenTokenResult.unwrap();
    if rightParenToken.get_type() != Token_Type::RightParen{
        tokens.setPosition(position);
        return None;
    }

    return Some(priNode);
}

/// new a promgram AST tree EBNF is identifier = identifier token
/// create by liufoyang
fn identifier(tokens:&mut TokenList) -> Option<AST_Node> {
    let nextTokenResult = tokens.preRead();

    if nextTokenResult.is_none() {
        return None;
    }

    let token = nextTokenResult.unwrap();
    if token.get_type()!=Token_Type::Identifier {
        return None;
    }

    tokens.next();
    let node = AST_Node::new(AST_Node_Type::Identifier, token.get_text());
    return Some(node);
}

/// new a promgram AST tree
/// create by liufoyang  EBNF is intLiteral = intLiteral token
fn intLiteral(tokens:&mut TokenList) -> Option<AST_Node> {
    let nextTokenResult = tokens.preRead();

    if nextTokenResult.is_none() {
        return None;
    }

    let token = nextTokenResult.unwrap();
    if token.get_type()!=Token_Type::IntLiteral {
        return None;
    }
    tokens.next();
    let node = AST_Node::new(AST_Node_Type::IntLiteral, token.get_text());
    return Some(node);
}

/// new a promgram AST tree EBNF is stringLiteral = stringLiteral token
/// create by liufoyang
fn stringLiteral(tokens:&mut TokenList) -> Option<AST_Node> {
    let nextTokenResult = tokens.preRead();

    if nextTokenResult.is_none() {
        return None;
    }

    let token = nextTokenResult.unwrap();
    if token.get_type()!=Token_Type::StringLiteral {
        return None;
    }
    tokens.next();
    let node = AST_Node::new(AST_Node_Type::StringLiteral, token.to_string().as_str());
    return Some(node);
}

pub fn executeSripte(node:&mut AST_Node, varMap:&mut HashMap<String, i32>) {

    if node.children.len() == 0 {
        return;
    }

    for child in &mut node.children {
        let value = executeNode(child,  varMap);
        println!("{}", value);
    }

    // return context params
    //return varMap;
}

fn executeNode (node:&mut AST_Node, paramMap:&mut HashMap<String, i32>) -> i32 {
    let mut result = 0;
    match node.node_type {
        AST_Node_Type::Declaration => {
            let varName = node.value.clone();
            let mut value = 0;
            if node.children.len() >0 {
                value = executeNode(node.children.get_mut(0).unwrap(), paramMap);
            }
            result = value.clone();
            paramMap.insert(varName, value);
        },
        AST_Node_Type::ConditionBlockStmt => {
            //
            if node.children.len() < 2 {
                panic!("condition stmt must have 2 child node");
            }

            // condition
            let mut child1 = node.children.get_mut(0).unwrap();
            let condition_result = executeConditionNode(child1, paramMap);

            if condition_result {
                let mut child2 = node.children.get_mut(1).unwrap();
                result = executeNode(child2, paramMap);
            } else if node.children.len()>2 {
                let mut child3 = node.children.get_mut(2).unwrap();
                result = executeNode(child3, paramMap);
            }
        },

        AST_Node_Type::AssignmentStmt =>{
            let varName = node.value.clone();
            let child = node.children.get_mut(0).unwrap();
            let mut value = 0;
            value = executeNode(child, paramMap);
            result = value.clone();
            paramMap.insert(varName, value);
        },
        AST_Node_Type::ExpressionStmt =>{
            // Not need to execute this node;
            result = 0;
        },

        AST_Node_Type::Additive => {
            if node.children.len() == 1 {
                let child = node.children.get_mut(0).unwrap();
                result = executeNode(child, paramMap);
            }

            if node.children.len() == 2 {
                let child1 = node.children.get_mut(0).unwrap();
                let value1 = executeNode(child1, paramMap);

                let child2 = node.children.get_mut(1).unwrap();
                let value2 = executeNode(child2, paramMap);

                let addTokenValue = node.value.as_str();
                if addTokenValue == "+" {
                    result = value1+value2;
                }

                if addTokenValue == "-" {
                    result = value1-value2;
                }
            }
        }

        AST_Node_Type::Multiplicative => {
            if node.children.len() == 1 {
                let child = node.children.get_mut(0).unwrap();
                result = executeNode(child, paramMap);
            }

            if node.children.len() == 2 {
                let child1 = node.children.get_mut(0).unwrap();
                let value1 = executeNode(child1, paramMap);

                let child2 = node.children.get_mut(1).unwrap();
                let value2 = executeNode(child2, paramMap);

                let addTokenValue = node.value.as_str();
                if addTokenValue == "*" {
                    result = value1*value2;
                }

                if addTokenValue == "/" {
                    result = value1/value2;
                }
            }
        },

        AST_Node_Type::Identifier => {
            let identifierName = node.value.as_str();
            if paramMap.contains_key(&node.value) {
                result = paramMap.get(&node.value).unwrap().clone();
                //println!("param get {}", result);
            }
        },

        AST_Node_Type::IntLiteral => {
            let intValue = node.value.clone();
            let value = intValue.trim().parse::<>().unwrap();

            result = value;

        },
        _ => {
            result = 0;
        }

    }
    //println!("resut value {}", result);
    return result;
}

fn executeConditionNode (node:&mut AST_Node, paramMap:&mut HashMap<String, i32>) -> bool {
    //println!("{:?} condition express {}", node.node_type, node.value);
    if node.node_type != AST_Node_Type::ConditionExpression {
        panic!("not condition expression");
    }

    if node.children.len()!=2 {
        panic!("condition must have two child expression");
    }

    let mut child1 = node.children.get_mut(0).unwrap();
    let value1 = executeNode(child1, paramMap);

    let mut child2 = node.children.get_mut(1).unwrap();
    let value2 = executeNode(child2, paramMap);

    if node.value.as_str() == "==" {
        return value1 == value2;
    }

    if node.value.as_str() == "!=" {
        return value1 != value2;
    }

    if node.value.as_str() == ">" {
        return value1 > value2;
    }

    if node.value.as_str() == "<" {
        return value1 < value2;
    }

    if node.value.as_str() == ">=" {
        //println!("{} >= {}", value1, value2);
        return value1 >= value2;
    }

    if node.value.as_str() == "<=" {
        return value1 <= value2;
    }

    return false;

}


#[cfg(test)]
mod tests {
    use crate::lib::conselexer::TokenList;
    use crate::lib::conselexer::Token;
    use crate::lib::conselexer::Token_Type;
    use crate::lib::conselexer;
    use super::AST_Node;
    use super::AST_Node_Type;
    use std::collections::HashMap;


    //#[test]
    fn test_cal_AST_node() {
        let code = String::from("let a = 100; a=(a-1)*15*(a+1);a+10*3;");

        let mut tokens = conselexer::lexerParse(code.as_str());

        //assert_eq!(19, tokens.len());

        let ast_node = super::syntaxParse(&mut tokens);
        assert!(ast_node.is_some());
        let mut node = ast_node.unwrap();
        //println!("{:?}", node);

        let mut varMap = HashMap::new();
        super::executeSripte(&mut node, &mut varMap);
    }


    #[test]
    fn test_cal_AST_node_condition() {
        let code = String::from("let a:i32 = 100; let b:i32=101; if a>=100 a=a+1; else b=b+10; a+b*3;");

        let mut tokens = conselexer::lexerParse(code.as_str());

        //assert_eq!(19, tokens.len());

        let ast_node = super::syntaxParse(&mut tokens);
        assert!(ast_node.is_some());
        let mut node = ast_node.unwrap();
        println!("{:?}", node);

        let mut varMap = HashMap::new();
        super::executeSripte(&mut node, &mut varMap);
    }


    #[test]
    fn test_cal_AST_node_function() {
        let code = String::from("
        fn add(a:i32, b:i32) -> i32 {
	       return a+b;
        }

        fn main() {
	       let a:i32 = 1;
	       let b:i32 = 2;

	       add(a, b);
        }");

        let mut tokens = conselexer::lexerParse(code.as_str());

        //assert_eq!(19, tokens.len());

        let ast_node = super::syntaxParse(&mut tokens);
        assert!(ast_node.is_some());
        let mut node = ast_node.unwrap();
        println!("{:?}", node);

        let mut varMap = HashMap::new();
        super::executeSripte(&mut node, &mut varMap);
    }
}

