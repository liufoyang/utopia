#![feature(float_to_from_bytes)]
mod lib;
use lib::conselexer;
use lib::consesyntax;
use lib::consesemantic;
use lib::runtime::{code_machine,conse_execute_machine};
extern crate log;
use log::*;
use std::env;
use std::fs::File;
use std::io::{Read,Write};

use std::io;
use std::collections::HashMap;

fn main() {

    let mut arguments = Vec::new();
    for argument in env::args() {
        arguments.push(argument);
    }

//    if arguments.len() < 2 {
//        panic!("args error must have [-c|-r] option file");
//    }



    if arguments.len() > 2 {
        println!("args {:?}", arguments);
        let complile_file = arguments.pop().unwrap();
        let option_type = arguments[1].as_str();

        if option_type == "-c" {
            info!("complile_file file {}", complile_file);

            let source_file_result = File::open(complile_file.as_str());
            if source_file_result.is_err() {
                panic!("code file can not open {}", complile_file);
            }

            let mut source_file = source_file_result.unwrap();
            let mut contents = String::new();
            source_file.read_to_string(&mut contents).unwrap();

            let mut tokens = conselexer::lexerParse(contents.as_str());

            let ast_node = consesyntax::syntaxParse(&mut tokens);
            let mut node = ast_node.unwrap();

            let mut semantic_context = consesemantic::semanticParse(&node);
            let code_list = consesemantic::print_simple_AST_code(&node, &mut semantic_context);

            // write class file
            let mut class_file_name = String::from(complile_file.as_str());
            if class_file_name.ends_with(".fy"){
                class_file_name = class_file_name.replace(".fy", ".fycs");
            }

            let class_file_result = File::create(class_file_name.to_string());

            if class_file_result.is_err() {
                panic!("error to create class file {}", class_file_name);
            }

            let mut class_file:File = class_file_result.unwrap();
            for code in &code_list {
                class_file.write(code.as_str().as_bytes());
                class_file.write("\n".as_bytes());
            }

            let mut code_machine_instance = code_machine::new(class_file_name.as_str());
            code_machine_instance.simple_code_to_bytes();
            code_machine_instance.writeCodeClass();
        }

        if option_type=="-r" {

            info!("run file {}", complile_file);
            let mut execute_machine =  conse_execute_machine::new(complile_file.as_str());
            execute_machine.execute_instruct();
        }


    } else {
        let mut input_str = String::new();

        let mut varMap = HashMap::new();
        while true {
            input_str = String::new();
            match io::stdin().read_line(&mut input_str) {
                Ok(n) => {
                    if input_str.as_str() == "q" || input_str.as_str() == "quit" {
                        break;
                    }

                    //input_str.replace("\n")
                    let mut tokens = conselexer::lexerParse(input_str.as_str());


                    let ast_node = consesyntax::syntaxParse(&mut tokens);

                    let mut node = ast_node.unwrap();

                    consesyntax::executeSripte(&mut node, &mut varMap);
                }
                Err(error) => panic!("error: {}", error),
            }
        }
    }



}
