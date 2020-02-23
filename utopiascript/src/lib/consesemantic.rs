use super::consesyntax::{AST_Node, AST_Node_Type};
use std::collections::HashMap;

#[derive(Clone)]
#[derive(Debug)]
enum Symbol_Type {
    i32,
    i64,
    f32,
    f64,
    string,
    funtion,
    bool,
    void,
}

impl Symbol_Type {
    pub fn fromStr(name:&str) -> Symbol_Type{
        if (name == "i32") {
            return Symbol_Type::i32;
        }

        if (name == "i64") {
            return Symbol_Type::i64;
        }

        if (name == "f64") {
            return Symbol_Type::f64;
        }

        if (name == "string") {
            return Symbol_Type::string;
        }

        if (name == "funtion") {
            return Symbol_Type::funtion;
        }

        if (name == "f32") {
            return Symbol_Type::f32;
        }

        return Symbol_Type::void;
    }
}

impl PartialEq for Symbol_Type {

    fn eq(&self, other: &Self) -> bool {
        let a_value = self.clone() as u32;
        let b_value = other.clone() as u32;

        return a_value == b_value;
    }

}

#[derive(Debug)]
#[derive(Clone)]
struct Symbol {
    name:String,
    stype:Symbol_Type,
    scope_index: usize,
}

impl Symbol {
    pub fn new(_name:&str, _stype:Symbol_Type, _scope_index:usize) -> Symbol {
        let symbol = Symbol{
            name:_name.to_string(),
            stype:_stype,
            scope_index: _scope_index,
        };

        return symbol;
    }

    pub fn get_stype(&self) -> Symbol_Type {
        return self.stype.clone();
    }
}



struct Block_Scope {
    symbol_index_list:Vec<usize>,
    parent_index:Option<usize>,
}

impl Block_Scope {
    pub fn new(_parent_index:Option<usize>) -> Block_Scope {
        let scope = Block_Scope {
            symbol_index_list:Vec::new(),
            parent_index:_parent_index,
        };
        return scope;
    }

    pub fn get_parent_index(&self) -> Option<usize> {
        return self.parent_index.clone();
    }

    pub fn containCurrentScopeSymbol(&self, name:&str, semantic_context:&Semantic_Context) -> bool {
        for symbol_index in &self.symbol_index_list {
            let symbol = semantic_context.symbol_list.get(symbol_index.clone()).unwrap();
            if symbol.name.as_str() == name  {
                return true;
            }
        }

        return false;
    }

    pub fn findSymbol(&self, name:&str, semantic_context:&Semantic_Context) -> Option<Symbol_Type> {
        for symbol in &semantic_context.symbol_list {
            if symbol.name.as_str() == name  {
                return Some(symbol.stype.clone());
            }
        }

        if self.parent_index.is_some() {
            let parent_index_value = self.parent_index.unwrap().clone();
            let parent_scope = semantic_context.scope_list.get(parent_index_value).unwrap();

            return parent_scope.findSymbol(name, semantic_context);
        }

        return None;

    }
}

pub struct Semantic_Context {
    symbol_list:Vec<Symbol>,
    scope_list:Vec<Block_Scope>,
    node_scope_map:HashMap<i32, usize>,
    node_symbol_map:HashMap<i32, usize>,
    node_primary_type_map:HashMap<i32, Symbol_Type>,
}

impl Semantic_Context {
    pub fn addSymbolToScope (&mut self, name:&str, stype:Symbol_Type, scope_index:usize) -> usize{

        let scope = self.scope_list.get(scope_index).unwrap();
        if (scope.containCurrentScopeSymbol(name, self)) {
            panic!("repeate symbol defined for {}", name);
        }

        let symbol = Symbol {
            name:name.to_string(),
            stype:stype.clone(),
            scope_index: scope_index,
        };

        self.symbol_list.push(symbol);

        let symbol_index = self.scope_list.len() -1;


        return symbol_index;
    }
}

fn findSymbolFromAvaScope (semantic_context: &Semantic_Context, scope_list:&Vec<Block_Scope>, _name:&str, scope_index:usize) ->Option<Symbol_Type> {
    let currunt_scope = scope_list.get(scope_index).unwrap();

    let mut symbol_type_result = currunt_scope.findSymbol(_name.clone(), semantic_context);

    if symbol_type_result.is_some() {
        return symbol_type_result;
    }

    let mut parent_index_result = currunt_scope.get_parent_index();
    while parent_index_result.is_some() {
        let scope = scope_list.get(scope_index).unwrap();
        symbol_type_result = scope.findSymbol(_name.clone(), semantic_context);

        if symbol_type_result.is_some() {
            break;
        }
    }

    return symbol_type_result;

}

pub trait AST_Tree_Processor {
    fn process_enter_node(&mut self, node: & AST_Node, semantic_context:&mut Semantic_Context);

    fn process_exit_node(&mut self, node: & AST_Node, semantic_context:&mut Semantic_Context);
}

pub fn walk_AST_tree(root: & AST_Node, tree_process: &mut AST_Tree_Processor, semantic_context:&mut Semantic_Context) {

    walk_sigle_AST_node(root, tree_process, semantic_context);
}

pub fn walk_sigle_AST_node(node: & AST_Node, tree_process: &mut AST_Tree_Processor, semantic_context:&mut Semantic_Context) {
    tree_process.process_enter_node(node, semantic_context);

    for childNode in node.getChildren() {
        walk_sigle_AST_node(childNode, tree_process, semantic_context);
    }

    tree_process.process_exit_node(node, semantic_context);
}

///作用域的解析
///
struct Scope_Resolver{
    scope_index_stack:Vec<usize>,
    current_index:usize
}

impl Scope_Resolver {
    pub fn new(semantic_context:&mut Semantic_Context) -> Scope_Resolver{
        let mut resolver = Scope_Resolver{
            scope_index_stack:Vec::new(),
            current_index:0
        };

        let gload_scope = Block_Scope::new(None);
        semantic_context.scope_list.push(gload_scope);
        resolver.scope_index_stack.push(semantic_context.scope_list.len() -1);
        return resolver;
    }

    pub fn pushScope(&mut self, scope:Block_Scope, semantic_context:&mut Semantic_Context) {
        semantic_context.scope_list.push(scope);

        self.scope_index_stack.push(semantic_context.scope_list.len()-1);
        self.current_index = semantic_context.scope_list.len()-1;
    }

    pub fn popScope(&mut self) -> Option<usize> {

        if(self.scope_index_stack.len() > 1) {
            let index = self.scope_index_stack.pop().unwrap();
            self.current_index = self.scope_index_stack.get(self.scope_index_stack.len() -1).unwrap().clone();
            return Some(index);
        } else {
            return None;
        }
    }

}

impl AST_Tree_Processor for Scope_Resolver {
    fn process_enter_node(&mut self, node: &AST_Node, semantic_context: &mut Semantic_Context) {

        // not new scope
        if node.get_type() != AST_Node_Type::Function
            && node.get_type() != AST_Node_Type::ConditionBlockStmt
            && node.get_type() != AST_Node_Type::StatementBlock{
            semantic_context.node_scope_map.insert(node.get_id(), self.current_index);
            return;
        }
        let mut scope = Block_Scope::new(Some(self.current_index.clone()));

        self.pushScope(scope, semantic_context);
        println!("add scope for node {}, current index {}", node.get_id(), self.current_index);

        semantic_context.node_scope_map.insert(node.get_id(), self.current_index);

    }

    fn process_exit_node(&mut self, node: &AST_Node, semantic_context: &mut Semantic_Context) {

        if (node.get_id() == 0) {
            return;
        }
        // not new scope
        if node.get_type() != AST_Node_Type::Function
            && node.get_type() != AST_Node_Type::ConditionBlockStmt
            && node.get_type() != AST_Node_Type::StatementBlock{

            return;
        }

        self.popScope();
        println!("exist scope for node {}, current index {}", node.get_id(), self.current_index);
    }
}

///作用域的解析
///
struct Variable_Resolver{
}

impl Variable_Resolver {
    pub fn new() -> Variable_Resolver{
        let variable_resolver = Variable_Resolver{
        };

        return variable_resolver;
    }

    pub fn addSymbol(&mut self, node_id:i32, name:&str, symbol_type:Symbol_Type, semantic_context: &mut Semantic_Context) -> usize {
        //semantic_context.symbol_list.push(symbol);

        let scope_index = semantic_context.node_scope_map.get(&node_id).unwrap().clone();
        println!("add symbol {} {:?} at scope {}", name,symbol_type, scope_index);

        let symbol_index = semantic_context.addSymbolToScope(name,symbol_type.clone(), scope_index);

        semantic_context.node_symbol_map.insert(node_id.clone(), symbol_index.clone());
        semantic_context.node_primary_type_map.insert(node_id, symbol_type);

        return symbol_index;
    }
}

impl AST_Tree_Processor for Variable_Resolver {
    fn process_enter_node(&mut self, node: &AST_Node, semantic_context: &mut Semantic_Context) {
        // not new scope
        if node.get_type() == AST_Node_Type::Declaration{
            let type_node = node.getChildren().get(1).unwrap();

            let identifer_node = node.getChildren().get(0).unwrap();
            let symbol_name = identifer_node.get_value().to_string();

            let mut symbol_type = Symbol_Type::void;
            if type_node.get_value().len()> 0 {
                symbol_type = Symbol_Type::fromStr(type_node.get_value());
            }

            self.addSymbol(node.get_id(), symbol_name.as_str(), symbol_type, semantic_context);
        }

        if node.get_type() == AST_Node_Type::Function{

            let mut symbol_name = node.get_value().to_string();

            let symbol_type = Symbol_Type::funtion;

            self.addSymbol(node.get_id(), symbol_name.as_str(), symbol_type, semantic_context);

            symbol_name.push_str("_call");
            let mut fun_call_type = Symbol_Type::void;

            let child_node = node.getChildren().get(1).unwrap();
            if child_node.get_type() == AST_Node_Type::FunctionReturn {
                let type_node = child_node.getChildren().get(0).unwrap();

                fun_call_type = Symbol_Type::fromStr(type_node.get_value());
            }

            self.addSymbol(node.get_id(), symbol_name.as_str(), fun_call_type, semantic_context);


        }

        if node.get_type() == AST_Node_Type::ParameterDefine {
            let type_node = node.getChildren().get(1).unwrap();

            let identifer_node = node.getChildren().get(0).unwrap();

            let symbol_name = identifer_node.get_value().to_string();

            let mut symbol_type = Symbol_Type::void;
            if type_node.get_value().len() > 0 {
                symbol_type = Symbol_Type::fromStr(type_node.get_value());
            }

            self.addSymbol(node.get_id(), symbol_name.as_str(), symbol_type, semantic_context);
        }

    }

    fn process_exit_node(&mut self, node: &AST_Node, semantic_context: &mut Semantic_Context) {
        return;
    }
}
///作用域的解析
///
struct PrimaryType_Resolver{

}

impl AST_Tree_Processor for PrimaryType_Resolver {
    fn process_enter_node(&mut self, node: &AST_Node, semantic_context: &mut Semantic_Context) {

        let mut symbol_type = Symbol_Type::void;

        if node.get_type() == AST_Node_Type::Primary{
            let type_node = node.getChildren().get(0).unwrap();

            symbol_type = Symbol_Type::fromStr(type_node.get_value());
        }

        // identifier 就取变量的定义类型
        if node.get_type() == AST_Node_Type::Identifier || node.get_type() == AST_Node_Type::AssignmentStmt{
            let scope_index:usize = semantic_context.node_scope_map.get(&node.get_id()).unwrap().clone();
            let scope = semantic_context.scope_list.get(scope_index).unwrap();

            let symbol_result = scope.findSymbol(node.get_value(), semantic_context);

            if symbol_result.is_some() {
                symbol_type = symbol_result.unwrap().clone();
            }
        }

        // function call 函数调用
        if node.get_type() == AST_Node_Type::FunctionCall {
            let scope_index:usize = semantic_context.node_scope_map.get(&node.get_id()).unwrap().clone();
            let scope = semantic_context.scope_list.get(scope_index).unwrap();

            let identifierNode = node.getChildren().get(0).unwrap();
            let mut fun_name = identifierNode.get_value().to_string();
            fun_name.push_str("_call");
            let symbol_result = scope.findSymbol(fun_name.as_str(), semantic_context);

            if symbol_result.is_some() {
                symbol_type = symbol_result.unwrap().clone();
            }
        }


        if node.get_type() == AST_Node_Type::StringLiteral {
            symbol_type = Symbol_Type::string;
        }

        if node.get_type() == AST_Node_Type::DoubleLiteral {
            symbol_type = Symbol_Type::f64;
        }

        if node.get_type() == AST_Node_Type::IntLiteral {
            symbol_type = Symbol_Type::i32;
        }

        if node.get_type() == AST_Node_Type::ConditionExpression{
            symbol_type = Symbol_Type::bool;
        }

        if node.get_type() == AST_Node_Type::CompareExpression{
            symbol_type = Symbol_Type::bool;
        }

        if node.get_type() == AST_Node_Type::FunctionReturn{
            let type_node = node.getChildren().get(0).unwrap();

            symbol_type = Symbol_Type::fromStr(type_node.get_value());
        }

        if node.get_type() == AST_Node_Type::TypeType{
            symbol_type = Symbol_Type::fromStr(node.get_value());
        }



        if symbol_type != Symbol_Type::void {
            semantic_context.node_primary_type_map.insert(node.get_id(), symbol_type);
        }



    }

    fn process_exit_node(&mut self, node: &AST_Node, semantic_context: &mut Semantic_Context) {
        let mut symbol_type = Symbol_Type::void;

        // main函数固定为void
        if node.get_type() == AST_Node_Type::Programm{
            symbol_type = Symbol_Type::void;
        }

        if node.get_type() == AST_Node_Type::FunctionBody{
            let child_nodes = node.getChildren();

            if(child_nodes.len()>0) {
                let chilid_node = child_nodes.get(0).unwrap();
                symbol_type = semantic_context.node_primary_type_map.get(&chilid_node.get_id()).unwrap().clone();

            }
        }

        if node.get_type() == AST_Node_Type::Declaration{
            let child_nodes = node.getChildren();
            let chilid_node = child_nodes.get(1).unwrap();
            symbol_type = semantic_context.node_primary_type_map.get(&chilid_node.get_id()).unwrap().clone();
        }


        // if 不一定有确定类型，因为if块和else块的最后语句不要求统一类型， 认为定： 由第二个子节点觉得类型， if conditon stament else statment
        if node.get_type() == AST_Node_Type::ConditionBlockStmt{
            let child_nodes = node.getChildren();
            let chilid_node = child_nodes.get(1).unwrap();
            symbol_type = semantic_context.node_primary_type_map.get(&chilid_node.get_id()).unwrap().clone();
        }

        if node.get_type() == AST_Node_Type::ReturnStmt {

            let statement_nodes = node.getChildren();
            if(statement_nodes.len()>0) {

                // 如果return带expressStatment
                let len = statement_nodes.len();
                let last_node = statement_nodes.get(len - 1).unwrap();
                symbol_type = semantic_context.node_primary_type_map.get(&last_node.get_id()).unwrap().clone();
            }

        }


        // 程序块的属性，就取return或者最后一句语句的类型
        if node.get_type() == AST_Node_Type::StatementBlock {
            let statement_nodes = node.getChildren();

            if(statement_nodes.len()>0) {
                // TODO 检查所有return类型。

                let len = statement_nodes.len();
                let mut node_type = Symbol_Type::void;
                let mut has_return = false;

                for child_node in statement_nodes {
                    if child_node.get_type()==AST_Node_Type::ReturnStmt {
                        let return_node_type = semantic_context.node_primary_type_map.get(&child_node.get_id()).unwrap().clone();

                        if !has_return {
                            has_return = true;
                            node_type = return_node_type;
                        } else {
                            if return_node_type!= node_type {
                                panic!("return value type must same but return {:?} and other return {:?}", node_type, return_node_type);
                            }
                        }
                    }
                }

                symbol_type = node_type;

            } else {
                symbol_type = Symbol_Type::void;
            }
        }

        if node.get_type() == AST_Node_Type::Multiplicative || node.get_type() == AST_Node_Type::Additive {
            let left_node = node.getChildren().get(0).unwrap();
            let left_type = semantic_context.node_primary_type_map.get(&left_node.get_id()).unwrap().clone();
            if left_type!= Symbol_Type::i32 && left_type!= Symbol_Type::f64 {
                panic!("error the mul value {:?} at node {}", left_type, node.get_id());
            }

            let right_node = node.getChildren().get(1).unwrap();
            let right_type = semantic_context.node_primary_type_map.get(&right_node.get_id()).unwrap().clone();

            if right_type!= Symbol_Type::i32 && right_type!= Symbol_Type::f64 {
                panic!("error the mul value {:?} at node {}", right_type, node.get_id());
            }

            if right_type == Symbol_Type::f64 ||left_type == Symbol_Type::f64 {
                symbol_type = Symbol_Type::f64;
            } else {
                symbol_type = Symbol_Type::i32;
            }
        }

        // 比较的表达式类型，肯定是bool，这里顺便检查两边类型是否一致
        if node.get_type() == AST_Node_Type::CompareExpression {

            let child_nodes = node.getChildren();
            let left_node = child_nodes.get(0).unwrap();
            let right_node = child_nodes.get(1).unwrap();

            let right_type = semantic_context.node_primary_type_map.get(&right_node.get_id()).unwrap().clone();
            let left_type = semantic_context.node_primary_type_map.get(&left_node.get_id()).unwrap().clone();

            if right_type != left_type {
                panic!("can not compare two value between {:?} and {:?}", right_type, left_type);
            }
        }

        // 条件语句肯定是bool类型，顺便看看其子节点是否都是bool
        if node.get_type() == AST_Node_Type::ConditionExpression {

            let child_nodes = node.getChildren();

            for child in child_nodes {
                let child_type = semantic_context.node_primary_type_map.get(&child.get_id()).unwrap().clone();
                if child_type != Symbol_Type::bool {
                    panic!("conditoin must be bool type");
                }

            }
        }

        if !semantic_context.node_primary_type_map.contains_key(&node.get_id()) {
            semantic_context.node_primary_type_map.insert(node.get_id(), symbol_type);
        }

    }
}

///
/// 类型检查，匹配各种运作的类型，或者做类型转换
struct PrimaryType_Checker {

}

impl AST_Tree_Processor for PrimaryType_Checker {
    fn process_enter_node(&mut self, node: &AST_Node, semantic_context: &mut Semantic_Context) {

        return;
    }

    fn process_exit_node(&mut self, node: &AST_Node, semantic_context: &mut Semantic_Context) {
        if node.get_type() == AST_Node_Type::Multiplicative  || node.get_type() == AST_Node_Type::Additive {
            let left_node = node.getChildren().get(0).unwrap();
            let left_type = semantic_context.node_primary_type_map.get(&left_node.get_id()).unwrap().clone();
            if left_type!= Symbol_Type::i32 && left_type!= Symbol_Type::f64 {
                panic!("error the mul value {:?}", left_type);
            }

            let right_node = node.getChildren().get(1).unwrap();
            let right_type = semantic_context.node_primary_type_map.get(&right_node.get_id()).unwrap().clone();

            if right_type!= Symbol_Type::i32 && right_type!= Symbol_Type::f64 {
                panic!("error the mul value {:?}", right_type);
            }
        }

        if node.get_type() == AST_Node_Type::CompareExpression {

            let left_node = node.getChildren().get(0).unwrap();
            let left_type = semantic_context.node_primary_type_map.get(&left_node.get_id()).unwrap().clone();

            let right_node = node.getChildren().get(1).unwrap();
            let right_type = semantic_context.node_primary_type_map.get(&right_node.get_id()).unwrap().clone();

            if left_type != right_type {
                panic!("can not compare value between {:?} and {:?}", left_type, right_type);
            }
        }

        if node.get_type() == AST_Node_Type::AssignmentStmt {

            let left_type = semantic_context.node_primary_type_map.get(&node.get_id()).unwrap().clone();

            let right_node = node.getChildren().get(0).unwrap();
            let right_type = semantic_context.node_primary_type_map.get(&right_node.get_id()).unwrap().clone();

            if left_type != right_type {
                panic!("can not assign {:?} to {:?} ", right_type, left_type);
            }
        }

        if node.get_type() == AST_Node_Type::Declaration {
            let left_type = semantic_context.node_primary_type_map.get(&node.get_id()).unwrap().clone();

            let right_node = node.getChildren().get(2).unwrap();
            let right_type = semantic_context.node_primary_type_map.get(&right_node.get_id()).unwrap().clone();

            if left_type != right_type {
                panic!("can not assign {:?} to {:?} ", right_type, left_type);
            }
        }

        if node.get_type() == AST_Node_Type::Function {

            let left_node = node.getChildren().get(1).unwrap();

            // return void
            if left_node.get_type() == AST_Node_Type::FunctionBody {
                let left_type = semantic_context.node_primary_type_map.get(&left_node.get_id()).unwrap().clone();

                if left_type != Symbol_Type::void {
                    panic!("function {} must return void but {:?} ", node.get_value(), left_type);
                }
            } else {
                let left_type = semantic_context.node_primary_type_map.get(&left_node.get_id()).unwrap().clone();

                let right_node = node.getChildren().get(2).unwrap();
                let right_type = semantic_context.node_primary_type_map.get(&right_node.get_id()).unwrap().clone();

                if left_type != right_type {
                    panic!("function {} must return {:?} but {:?} ",node.get_value(), left_type, right_type);
                }
            }

        }
    }
}

pub fn semanticParse(root:&AST_Node) -> Semantic_Context {

    let mut semantic_context = Semantic_Context{
        symbol_list:Vec::new(),
        scope_list:Vec::new(),
        node_scope_map:HashMap::new(),
        node_symbol_map:HashMap::new(),
        node_primary_type_map:HashMap::new(),
    };

    // 作用域的解析
    let mut scope_resolver = Scope_Resolver::new(&mut semantic_context);
    walk_AST_tree(root, &mut scope_resolver, &mut semantic_context);


    // 变量的解析
    let mut variable_resolver = Variable_Resolver::new();
    walk_AST_tree(root, &mut variable_resolver, &mut semantic_context);


    // 类型的解析
    let mut primary_type_resolver = PrimaryType_Resolver{};
    walk_AST_tree(root, &mut primary_type_resolver, &mut semantic_context);

    // 类型检查
    let mut  primary_type_checker = PrimaryType_Checker{};
    walk_AST_tree(root, &mut primary_type_checker, &mut semantic_context);

    println!("variable {:?}", semantic_context.symbol_list);
    println!("primary {:?}", semantic_context.node_primary_type_map);
    return semantic_context;
}


#[cfg(test)]
mod tests {

    use crate::lib::conselexer;
    use crate::lib::conselexer::{TokenList,Token, Token_Type};
    use crate::lib::consesyntax;
    use crate::lib::consesyntax::{AST_Node, AST_Node_Type};
    use std::collections::HashMap;

    #[test]
    fn test_semanticParse() {
        let code = String::from("
        fn add(c:i32, d:i32) -> i32 {
	       return c+d;
        }

        fn main() {
	       let a:i32 = 1;
	       let b:i32 = 2;

	       let c:i32 = add(a, b);
           if b > c {
              c = 10;
           };
        }");

        let mut tokens = conselexer::lexerParse(code.as_str());

        //assert_eq!(19, tokens.len());

        let ast_node = consesyntax::syntaxParse(&mut tokens);
        assert!(ast_node.is_some());
        let mut node = ast_node.unwrap();
        println!("{:?}", node);

        let semantic_context = super::semanticParse(&node);
    }
}