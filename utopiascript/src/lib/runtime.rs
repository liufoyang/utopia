#![feature(float_to_from_bytes)]
use std::io::{self, Write, Read, Seek, SeekFrom};
use std::fs::File;
use std::fs::OpenOptions;
use std::collections::HashMap;

#[derive(Clone)]
struct stack_frame {
    instruct_index:usize,
    operate_stack:[u8;128],
    stack_top:usize,
    data_vec:[u8;1024]
}

impl stack_frame {

    pub fn new() ->stack_frame{
        let frame = stack_frame{
            instruct_index:0,
            operate_stack:[0;128],
            stack_top:0,
            data_vec:[0;1024]
        };

        return frame;
    }
    pub fn store(&mut self, _index:usize, type_size:usize) {

        println!("store stack to data {} {}", _index, type_size);
        let end_index = _index + type_size - 1;
        for i in 0..type_size {
            self.data_vec[end_index -i] = self.operate_stack[self.stack_top];
            self.stack_top-=1;
        }
    }

    pub fn load(&mut self, _index:usize, type_size:usize) {
        for i in 0..type_size {
            self.stack_top+=1;
            self.operate_stack[self.stack_top] = self.data_vec[_index + i];
        }
    }


    pub fn pop_i32(&mut self) -> i32{
        let mut value_byte:[u8;4] = [0;4];
        for i in 0..value_byte.len() {
            value_byte[3-i] = self.operate_stack[self.stack_top];
            self.stack_top-=1;
        }
        let value =  i32::from_be_bytes(value_byte);
        println!("pop i32 {}", value);
        return value;
    }

    pub fn pop_i64(&mut self) -> i64 {
        let mut value_byte:[u8;8] = [0;8];

        for i in 0..value_byte.len() {
            value_byte[7-i] = self.operate_stack[self.stack_top];
            self.stack_top-=1;
        }
        println!("pop i64 ");
        return i64::from_be_bytes(value_byte);
    }

    pub fn pop_f32(&mut self) ->f32 {
        let mut value_byte:[u8;4] = [0;4];

        for i in 0..value_byte.len() {
            value_byte[3-i] = self.operate_stack[self.stack_top];
            self.stack_top-=1;
        }

        return f32::from_be_bytes(value_byte);
    }

    pub fn pop_f64(&mut self) ->f64 {
        let mut value_byte:[u8;8] = [0;8];

        for i in 0..value_byte.len() {
            value_byte[7-i] = self.operate_stack[self.stack_top];
            self.stack_top-=1;
        }

        return f64::from_be_bytes(value_byte);
    }

    pub fn pop_bool(&mut self) -> bool {
        let mut value_byte = 0 as u8;

        value_byte = self.operate_stack[self.stack_top];
        self.stack_top-=1;

        return value_byte == 1;
    }

    pub fn push_i32(&mut self, value:i32) {
        let value_byte = value.to_be_bytes();

        for i in 0..value_byte.len() {
            self.stack_top+=1;
            self.operate_stack[self.stack_top] = value_byte[i];
        }

        println!("push i32 {} {}", value, self.stack_top);
    }

    pub fn push_i64(&mut self, value:i64) {
        let value_byte = value.to_be_bytes();

        for i in 0..value_byte.len() {
            self.stack_top+=1;
            self.operate_stack[self.stack_top] = value_byte[i];
        }
    }

    pub fn push_f32(&mut self, value:f32) {
        let value_byte = value.to_be_bytes();

        for i in 0..value_byte.len() {
            self.stack_top+=1;
            self.operate_stack[self.stack_top] = value_byte[i];
        }
    }

    pub fn push_f64(&mut self, value:f64) {
        let value_byte = value.to_be_bytes();
        for i in 0..value_byte.len() {
            self.stack_top+=1;
            self.operate_stack[self.stack_top] = value_byte[i];
        }
    }

    pub fn push_bool(&mut self, value:bool) {
        let mut value_byte:[u8;1] = [0;1];
        if value {
            value_byte[0] = 1;
        } else {
            value_byte[0] = 0;
        }
        for i in 0..value_byte.len() {
            self.stack_top+=1;
            self.operate_stack[self.stack_top] = value_byte[i];
        }
    }

    pub fn push(&mut self, value:u8) {

        self.stack_top+=1;
        self.operate_stack[self.stack_top] = value;
        //println!("push u8 {} {}", value, self.stack_top);
    }

    pub fn pop(&mut self) ->u8 {
        let value = self.operate_stack[self.stack_top];
        self.stack_top-=1;

        return value;
    }

    pub fn get_stack_len(&self) -> usize {
        return self.stack_top.clone();
    }

}


/// 脚本执行的虚拟机，对自定义的操作栈帧，
/// 操作命令三段式的 comand type position, 例如 store i3 1 将第一个局部变量，类型为i32放入栈帧。
/// store 将局部变量放入栈帧 0
/// load 将局部变量帧放入 1
/// const 将常量放入栈 2
/// add 相加  3
/// sub 减    4  取栈顶两个值，第一个减去第二个
/// mul 乘    5
/// div 除    6  取栈顶两个值，第一个除去第二个
/// less   7 小于 取栈顶两个值，第一个< 小于 比较结果放入栈
/// greater 8 大于 取栈顶两个值，第一个< 小于 比较结果放入栈
/// le 9 小于等于 取栈顶两个值，第一个< 小于 比较结果放入栈
/// ge 10 大于等于 取栈顶两个值，第一个< 小于 比较结果放入栈
/// equal 11 等于 取栈顶两个值，第一个< 小于 比较结果放入栈
/// ifnotgo 12 取栈顶的值，看是否标记false，是则跳到某个指令位
/// goto 13 直接跳到某个指令位
/// push_frame 14 函数栈进栈
/// pop_frame 15  函数栈出栈 9
/// exist   16 退出函数
///
pub struct conse_execute_machine {
    frame_stack:Vec<stack_frame>,
    top_frame:stack_frame,
    code_vec:[u32;10240],
    code_index:usize,
    frame_index:usize,
}


impl conse_execute_machine {

    pub fn new(file_name:&str) ->conse_execute_machine {
        let code_file_result  = File::open(file_name);
        if code_file_result.is_err() {
            panic!("file not found {}", file_name);
        }

        let mut code_file = code_file_result.unwrap();

        let mut buf:[u8;1024] = [0;1024];
        let mut n = code_file.read(&mut buf).unwrap();

        println!("code len {}", n);
        let mut code_vec:[u32;10240] = [0;10240];
        let mut code_index:usize = 0;
        let main_index = usize::from_be_bytes(read8byte(&buf[8..16]));

        let mut i:usize = 16;
        while i < n {
            code_vec[code_index] = u32::from_be_bytes(read4byte(&buf[i..i+4]));
            code_index +=1;
            i = i+4;
        }

        while n == 1024  {
            n = code_file.read(&mut buf).unwrap();
            if n <4 {
                break;
            }

            i = 0;
            while i < n {
                code_vec[code_index] = u32::from_be_bytes(read4byte(&buf[i..i+4]));
                code_index +=1;
                i = i+4;
            }
        }

        println!("load code index {} {}", code_index, main_index);
        for i in 0..33 {
            println!("{:?}", code_vec[i].to_be_bytes());
        }

        let mut main_frame = stack_frame::new();
        let mut machine = conse_execute_machine {
            frame_stack:Vec::new(),
            top_frame:main_frame,
            code_vec:code_vec,
            code_index:main_index,
            frame_index:0,
        };

        return machine;
    }


    pub fn execute_instruct(&mut self) {
        while self.code_index < self.code_vec.len() {
            // 取指令
            let cmd_bytes = self.get_next_cmd();


            // store type index 将操作栈，放入数据存储
            if cmd_bytes[0] == 0 {
                let type_u = cmd_bytes[1] as usize;
                let index_u = cmd_bytes[2] as usize;
                self.top_frame.store(index_u, type_u, );
            }

            // load type index 将局部变量放入栈帧 0
            if cmd_bytes[0] == 1 {
                let type_u = cmd_bytes[1] as usize;
                let index_u = cmd_bytes[2] as usize;
                self.top_frame.load(index_u, type_u);
            }

            // 2 const type value 将常量放入栈
            if cmd_bytes[0] == 2 {
                let type_u = cmd_bytes[1] as usize;

                self.mv_code_to_op_stack(type_u);
            }

            //3 add type 相加  类型
            if cmd_bytes[0] == 3 {
                let type_u = cmd_bytes[1] as usize;

                if (type_u == 1) {
                    let value1 = self.top_frame.pop_i32();
                    let value2 = self.top_frame.pop_i32();
                    self.top_frame.push_i32(value1+value2);
                }

                if (type_u == 2) {
                    let value1 = self.top_frame.pop_i64();
                    let value2 = self.top_frame.pop_i64();
                    self.top_frame.push_i64(value1+value2);
                }

                if (type_u == 3) {
                    let value1 = self.top_frame.pop_f32();
                    let value2 = self.top_frame.pop_f32();
                    self.top_frame.push_f32(value1+value2);
                }

                if (type_u == 4) {
                    let value1 = self.top_frame.pop_f64();
                    let value2 = self.top_frame.pop_f64();
                    self.top_frame.push_f64(value1+value2);
                }


            }

            //4 sub type 减      取栈顶两个值，第一个减去第二个
            if cmd_bytes[0] == 4 {
                let type_u = cmd_bytes[1] as usize;

                if (type_u == 1) {
                    let value1 = self.top_frame.pop_i32();
                    let value2 = self.top_frame.pop_i32();
                    self.top_frame.push_i32(value1-value2);
                }

                if (type_u == 2) {
                    let value1 = self.top_frame.pop_i64();
                    let value2 = self.top_frame.pop_i64();
                    self.top_frame.push_i64(value1-value2);
                }

                if (type_u == 3) {
                    let value1 = self.top_frame.pop_f32();
                    let value2 = self.top_frame.pop_f32();
                    self.top_frame.push_f32(value1-value2);
                }

                if (type_u == 4) {
                    let value1 = self.top_frame.pop_f64();
                    let value2 = self.top_frame.pop_f64();
                    self.top_frame.push_f64(value1-value2);
                }


            }

            //5 mul type 乘
            if cmd_bytes[0] == 5 {
                let type_u = cmd_bytes[1] as usize;

                if (type_u == 1) {
                    let value1 = self.top_frame.pop_i32();
                    let value2 = self.top_frame.pop_i32();
                    self.top_frame.push_i32(value1*value2);
                }

                if (type_u == 2) {
                    let value1 = self.top_frame.pop_i64();
                    let value2 = self.top_frame.pop_i64();
                    self.top_frame.push_i64(value1*value2);
                }

                if (type_u == 3) {
                    let value1 = self.top_frame.pop_f32();
                    let value2 = self.top_frame.pop_f32();
                    self.top_frame.push_f32(value1*value2);
                }

                if (type_u == 4) {
                    let value1 = self.top_frame.pop_f64();
                    let value2 = self.top_frame.pop_f64();
                    self.top_frame.push_f64(value1*value2);
                }


            }

            //6 div type 除
            if cmd_bytes[0] == 6 {
                let type_u = cmd_bytes[1] as usize;

                if (type_u == 1) {
                    let value1 = self.top_frame.pop_i32();
                    let value2 = self.top_frame.pop_i32();
                    self.top_frame.push_i32(value1 / value2);
                }

                if (type_u == 2) {
                    let value1 = self.top_frame.pop_i64();
                    let value2 = self.top_frame.pop_i64();
                    self.top_frame.push_i64(value1 / value2);
                }

                if (type_u == 3) {
                    let value1 = self.top_frame.pop_f32();
                    let value2 = self.top_frame.pop_f32();
                    self.top_frame.push_f32(value1 / value2);
                }

                if (type_u == 4) {
                    let value1 = self.top_frame.pop_f64();
                    let value2 = self.top_frame.pop_f64();
                    self.top_frame.push_f64(value1 / value2);
                }
            }

                //less   7 小于 取栈顶两个值，第一个< 小于 比较结果放入栈
                if cmd_bytes[0] == 7 {
                    let type_u = cmd_bytes[1] as usize;

                    if (type_u == 1) {
                        let value1 = self.top_frame.pop_i32();
                        let value2 = self.top_frame.pop_i32();
                        self.top_frame.push_bool((value1<value2));
                    }

                    if (type_u == 2) {
                        let value1 = self.top_frame.pop_i64();
                        let value2 = self.top_frame.pop_i64();
                        self.top_frame.push_bool(value1<value2);
                    }

                    if (type_u == 3) {
                        let value1 = self.top_frame.pop_f32();
                        let value2 = self.top_frame.pop_f32();
                        self.top_frame.push_bool(value1<value2);
                    }

                    if (type_u == 4) {
                        let value1 = self.top_frame.pop_f64();
                        let value2 = self.top_frame.pop_f64();
                        self.top_frame.push_bool(value1>value2);
                    }
                }

                //greater 8 大于 取栈顶两个值，第一个< 小于 比较结果放入栈
                if cmd_bytes[0] == 8 {
                    let type_u = cmd_bytes[1] as usize;

                    if (type_u == 1) {
                        let value1 = self.top_frame.pop_i32();
                        let value2 = self.top_frame.pop_i32();
                        self.top_frame.push_bool((value1>value2));
                    }

                    if (type_u == 2) {
                        let value1 = self.top_frame.pop_i64();
                        let value2 = self.top_frame.pop_i64();
                        self.top_frame.push_bool(value1>value2);
                    }

                    if (type_u == 3) {
                        let value1 = self.top_frame.pop_f32();
                        let value2 = self.top_frame.pop_f32();
                        self.top_frame.push_bool(value1>value2);
                    }

                    if (type_u == 4) {
                        let value1 = self.top_frame.pop_f64();
                        let value2 = self.top_frame.pop_f64();
                        self.top_frame.push_bool(value1>value2);
                    }
                }

                //le 9 小于等于 取栈顶两个值，第一个<= 小于 比较结果放入栈
                if cmd_bytes[0] == 9 {
                    let type_u = cmd_bytes[1] as usize;

                    if (type_u == 1) {
                        let value1 = self.top_frame.pop_i32();
                        let value2 = self.top_frame.pop_i32();
                        self.top_frame.push_bool((value1<=value2));
                    }

                    if (type_u == 2) {
                        let value1 = self.top_frame.pop_i64();
                        let value2 = self.top_frame.pop_i64();
                        self.top_frame.push_bool(value1<=value2);
                    }

                    if (type_u == 3) {
                        let value1 = self.top_frame.pop_f32();
                        let value2 = self.top_frame.pop_f32();
                        self.top_frame.push_bool(value1<=value2);
                    }

                    if (type_u == 4) {
                        let value1 = self.top_frame.pop_f64();
                        let value2 = self.top_frame.pop_f64();
                        self.top_frame.push_bool(value1<=value2);
                    }
                }

                //ge 10 小于等于 取栈顶两个值，第一个<= 小于 比较结果放入栈
                if cmd_bytes[0] == 10 {
                    let type_u = cmd_bytes[1] as usize;

                    if (type_u == 1) {
                        let value1 = self.top_frame.pop_i32();
                        let value2 = self.top_frame.pop_i32();
                        self.top_frame.push_bool((value1>=value2));
                    }

                    if (type_u == 2) {
                        let value1 = self.top_frame.pop_i64();
                        let value2 = self.top_frame.pop_i64();
                        self.top_frame.push_bool(value1>=value2);
                    }

                    if (type_u == 3) {
                        let value1 = self.top_frame.pop_f32();
                        let value2 = self.top_frame.pop_f32();
                        self.top_frame.push_bool(value1>=value2);
                    }

                    if (type_u == 4) {
                        let value1 = self.top_frame.pop_f64();
                        let value2 = self.top_frame.pop_f64();
                        self.top_frame.push_bool(value1>=value2);
                    }
                }

                //ge 11 等于 取栈顶两个值，第一个<= 小于 比较结果放入栈
                if cmd_bytes[0] == 11 {
                    let type_u = cmd_bytes[1] as usize;

                    if (type_u == 1) {
                        let value1 = self.top_frame.pop_i32();
                        let value2 = self.top_frame.pop_i32();
                        self.top_frame.push_bool((value1==value2));
                    }

                    if (type_u == 2) {
                        let value1 = self.top_frame.pop_i64();
                        let value2 = self.top_frame.pop_i64();
                        self.top_frame.push_bool(value1==value2);
                    }

                    if (type_u == 3) {
                        let value1 = self.top_frame.pop_f32();
                        let value2 = self.top_frame.pop_f32();
                        self.top_frame.push_bool(value1==value2);
                    }

                    if (type_u == 4) {
                        let value1 = self.top_frame.pop_f64();
                        let value2 = self.top_frame.pop_f64();
                        self.top_frame.push_bool(value1==value2);
                    }
                }

                //ifnotgo 12 取栈顶的值，看是否标记ture，是则跳到某个指令位
                if cmd_bytes[0] == 12 {
                    let value = self.top_frame.pop_bool();
                    if !value {
                        let mut next_index_bytes:[u8;2] = [0;2];
                        next_index_bytes[0] = cmd_bytes[1];
                        next_index_bytes[1] = cmd_bytes[2];
                        let next_index = u16::from_be_bytes(next_index_bytes) as usize;

                        self.code_index = next_index;
                        //break;
                    }
                }

                //goto 12 取栈顶的值，看是否标记ture，是则跳到某个指令位
                if cmd_bytes[0] == 13 {
                    let mut next_index_bytes:[u8;2] = [0;2];
                    next_index_bytes[0] = cmd_bytes[1];
                    next_index_bytes[1] = cmd_bytes[2];
                    let next_index = u16::from_be_bytes(next_index_bytes) as usize;

                    self.code_index = next_index;
                    //break;
                }

                // push_frame 14 函数栈进栈 push_frame next_index operator
            if cmd_bytes[0] == 14 {
                let mut next_index_bytes:[u8;2] = [0;2];
                next_index_bytes[0] = cmd_bytes[1];
                next_index_bytes[1] = cmd_bytes[2];
                let next_index = u16::from_be_bytes(next_index_bytes) as usize;

                let operator_num = cmd_bytes[3] as usize;
                // 存储这个帧的执行指令位置，用于出栈后继续执行。
                self.top_frame.instruct_index = self.code_index;
                let mut new_frame = stack_frame::new();
                self.push_frame(new_frame, operator_num);
                self.code_index = next_index;
                //break;
            }

            //pop_frame 15  函数栈出栈 9
            if cmd_bytes[0] == 15 {
                self.pop_frame();
                self.code_index = self.top_frame.instruct_index;
                //break;
            }

            //16  exist
            if cmd_bytes[0] == 16 {
                //println!("script exist success, stack len last {}", self.top_frame.operate_stack.len());
                return;
            }

        }

        // 如果需要根据指令取数据


    }

    pub fn mv_code_to_op_stack(&mut self, len:usize) {
        println!("begin mv_code_to_op_stack {} {}", len, self.code_index);
        let mut temp_byte = self.code_vec[self.code_index].to_be_bytes();
        self.code_index += 1;
        for i in 0..len {

            self.top_frame.push(temp_byte[i%4]);
            if i>0 && i%4 == 0 {
                temp_byte = self.code_vec[self.code_index].to_be_bytes();
                self.code_index += 1;
            }
        }

        println!("end mv_code_to_op_stack {} {}", len, self.code_index);
    }

    pub fn get_next_i32(&mut self) -> i32 {
        self.code_index += 1;
        let byte_value = self.code_vec[self.code_index].to_be_bytes();
        return i32::from_be_bytes(byte_value);
    }

    pub fn get_next_i64(&mut self) -> i64 {
        self.code_index += 1;
        let mut byte_value:[u8;8] = [0;8];
        let mut temp_byte = self.code_vec[self.code_index].to_be_bytes();
        for i in 0..8{
            if i==4 {
                self.code_index += 1;
                temp_byte = self.code_vec[self.code_index].to_be_bytes();
            }
            byte_value[i] = temp_byte[i%4];

        }
        return i64::from_be_bytes(byte_value);
    }

    pub fn get_next_f32(&mut self) -> f32 {
        self.code_index += 1;
        let byte_value = self.code_vec[self.code_index].to_be_bytes();
        return f32::from_be_bytes(byte_value);
    }

    pub fn get_next_f64(&mut self) -> f64 {
        self.code_index += 1;
        let mut byte_value:[u8;8] = [0;8];
        let mut temp_byte = self.code_vec[self.code_index].to_be_bytes();
        for i in 0..8{
            if i==4 {
                self.code_index += 1;
                temp_byte = self.code_vec[self.code_index].to_be_bytes();
            }
            byte_value[i] = temp_byte[i%4];

        }
        return f64::from_be_bytes(byte_value);
    }

    pub fn get_next_cmd(&mut self) -> [u8;4] {

        let byte_value = self.code_vec[self.code_index].to_be_bytes();
        println!("get code {}, {:?}", self.code_index, byte_value);
        self.code_index += 1;
        return byte_value;
    }

    pub fn get_next_bool(&mut self) -> bool {
        self.code_index += 1;
        let byte_value = self.code_vec[self.code_index].to_be_bytes();
        return byte_value[0]>0;
    }

    fn pop_frame(&mut self) {
        let mut next_top_frame = self.frame_stack.pop().unwrap();

        // 如果这个frame操作栈还有内容，就是返回值
        let operate_num = self.top_frame.get_stack_len();
        if operate_num > 0 {

            let mut temp:Vec<u8>  = Vec::new();

            for i in 0..operate_num {
                temp.push(self.top_frame.pop());
            }

            while temp.len() > 0 {
                next_top_frame.push(temp.pop().unwrap());
            }
        }

        self.top_frame = next_top_frame;
        self.frame_index -=1;
    }

    fn push_frame(&mut self, mut new_frame:stack_frame, operate_num:usize) {

        // 有参数传递，把当前帧操作栈的operate_num 曹的内容，copy到新帧的操作栈
        if operate_num > 0 {
            let mut temp:Vec<u8>  = Vec::new();
            //let mut top_frame = self.top_frame.clone();
            for i in 0..operate_num {
                temp.push(self.top_frame.pop());
            }

            while temp.len() > 0 {
                new_frame.push(temp.pop().unwrap());
            }
        }

        self.frame_stack.push(self.top_frame.clone());
        self.frame_index +=1;
        self.top_frame = new_frame;
    }

}

pub struct code_machine {
    fun_label_map:HashMap<String, usize>,
    type_map:HashMap<String, u8>,
    type_len_map:HashMap<String, u8>,
    call_map:HashMap<usize, String>,
    code_vec:Vec<u32>,
    code_index:usize,
    code_begin_index: usize,
    version:usize,
    file_name:String,
    code_line_map:HashMap<usize, usize>
}

impl code_machine {
    pub fn new(file_name:&str) -> code_machine{
        let mut type_map:HashMap<String, u8> = HashMap::new();
        type_map.insert("i32".to_string(), 1);
        type_map.insert("i64".to_string(), 2);
        type_map.insert("f32".to_string(), 3);
        type_map.insert("f64".to_string(), 4);
        type_map.insert("bool".to_string(),5);
        type_map.insert("char".to_string(), 6);

        let mut type_len_map:HashMap<String, u8> = HashMap::new();
        type_len_map.insert("i32".to_string(), 4);
        type_len_map.insert("i64".to_string(), 8);
        type_len_map.insert("f32".to_string(), 4);
        type_len_map.insert("f64".to_string(), 8);
        type_len_map.insert("bool".to_string(),1);
        type_len_map.insert("char".to_string(), 1);

        let machine = code_machine{
            fun_label_map:HashMap::new(),
            type_map:type_map,
            type_len_map:type_len_map,
            call_map:HashMap::new(),
            code_vec:Vec::new(),
            code_index:0,
            code_begin_index:0,
            version:1000,
            file_name:file_name.to_string(),
            code_line_map:HashMap::new()
        };

        return machine;


    }

    pub fn simple_code_to_bytes(&mut self) {
        let base_code_file_result = File::open(self.file_name.clone());
        if base_code_file_result.is_err() {
            panic!("code file can not open {}", self.file_name);
        }

        let mut buffer = String::new();

        let mut base_code_file = base_code_file_result.unwrap();
        let file_size = base_code_file.read_to_string(&mut buffer).unwrap();

        if buffer.len() == 0 {
            panic!("code file is empty {}", self.file_name);
        }

        let lines: Vec<&str> = buffer.split('\n').collect();

        let mut sime_code_line = 1;
        for line_str in lines {
            self.code_line_map.insert( sime_code_line.clone(), self.code_index.clone());
            self.base_code_to_bytes(line_str);
            sime_code_line +=1;
        }

    }

    /// 脚本执行的虚拟机，对自定义的操作栈帧，
/// 操作命令三段式的 comand type position, 例如 store i3 1 将第一个局部变量，类型为i32放入栈帧。
/// store 将局部变量放入栈帧 0
/// load 将局部变量帧放入 1
/// const 将常量放入栈 2
/// add 相加  3
/// sub 减    4  取栈顶两个值，第一个减去第二个
/// mul 乘    5
/// div 除    6  取栈顶两个值，第一个除去第二个
/// less   7 小于 取栈顶两个值，第一个< 小于 比较结果放入栈
/// greater 8 大于 取栈顶两个值，第一个< 小于 比较结果放入栈
/// le 9 小于等于 取栈顶两个值，第一个< 小于 比较结果放入栈
/// ge 10 大于等于 取栈顶两个值，第一个< 小于 比较结果放入栈
/// equal 11 等于 取栈顶两个值，第一个< 小于 比较结果放入栈
/// ifgo 12 取栈顶的值，看是否标记ture，是则跳到某个指令位
/// goto 13 直接跳到某个指令位
/// push_frame 14 函数栈进栈
/// pop_frame 15  函数栈出栈 9
/// exist   16 退出函数
    fn base_code_to_bytes(&mut self, cmd_line:&str) {

        //println!("receive base code {}", cmd_line);

        let mut code_bytes:[u8;4] = [0;4];

        let tokens: Vec<&str> = cmd_line.split(' ').collect();

        if tokens[0] == "store" {
            code_bytes[0] = 0;

            let type_len = self.type_len_map.get(tokens[1]).unwrap().clone();

            let store_index = tokens[2].trim().parse::<u16>().unwrap();

            let store_index_byte = store_index.to_be_bytes();

            code_bytes[1] = type_len;
            code_bytes[2] = store_index_byte[0];
            code_bytes[3] = store_index_byte[1];

        }

        if tokens[0] == "load" {
            code_bytes[0] = 1;

            let type_len = self.type_len_map.get(tokens[1]).unwrap().clone();

            let store_index = tokens[2].trim().parse::<u16>().unwrap();

            let store_index_byte = store_index.to_be_bytes();

            code_bytes[1] = type_len;
            code_bytes[2] = store_index_byte[0];
            code_bytes[3] = store_index_byte[1];
        }

        if tokens[0] == "add" {
            code_bytes[0] = 3;

            let type_name = tokens[1];
            code_bytes[1] = self.type_map.get(type_name).unwrap().clone();
        }

        if tokens[0] == "sub" {
            code_bytes[0] = 4;

            let type_name = tokens[1];
            code_bytes[1] = self.type_map.get(type_name).unwrap().clone();
        }

        if tokens[0] == "mul" {
            code_bytes[0] = 5;

            let type_name = tokens[1];
            code_bytes[1] = self.type_map.get(type_name).unwrap().clone();
        }

        if tokens[0] == "div" {
            code_bytes[0] = 6;

            let type_name = tokens[1];
            code_bytes[1] = self.type_map.get(type_name).unwrap().clone();
        }

        if tokens[0] == "less" {
            code_bytes[0] = 7;

            let type_name = tokens[1];
            code_bytes[1] = self.type_map.get(type_name).unwrap().clone();
        }

        if tokens[0] == "greater" {
            code_bytes[0] = 8;

            let type_name = tokens[1];
            code_bytes[1] = self.type_map.get(type_name).unwrap().clone();
        }

        if tokens[0] == "le" {
            code_bytes[0] = 9;

            let type_name = tokens[1];
            code_bytes[1] = self.type_map.get(type_name).unwrap().clone();
        }

        if tokens[0] == "ge" {
            code_bytes[0] = 10;

            let type_name = tokens[1];
            code_bytes[1] = self.type_map.get(type_name).unwrap().clone();
        }

        if tokens[0] == "equal" {
            code_bytes[0] = 11;

            let type_name = tokens[1];
            code_bytes[1] = self.type_map.get(type_name).unwrap().clone();
        }

        if tokens[0] == "ifnotgo" {
            code_bytes[0] = 12;

            //let type_name = tokens[1];
            let line_index = tokens[1].trim().parse::<u16>().unwrap();

            let store_index_bytes = line_index.to_be_bytes();

            code_bytes[1] = store_index_bytes[0];
            code_bytes[2] = store_index_bytes[1];
        }

        if tokens[0] == "goto" {
            code_bytes[0] = 13;

            let line_index = tokens[1].trim().parse::<u16>().unwrap();

            let store_index_bytes = line_index.to_be_bytes();

            code_bytes[1] = store_index_bytes[0];
            code_bytes[2] = store_index_bytes[1];
        }

        if tokens[0] == "call" {
            code_bytes[0] = 14;
            let fun_name = tokens[1];

            let operator_num = tokens[2].trim().parse::<u8>().unwrap();
            code_bytes[3] = operator_num;

            self.call_map.insert(self.code_index.clone(), fun_name.to_string());
        }

        if tokens[0] == "return" {
            code_bytes[0] = 15;
        }

        if tokens[0] == "exist" {
            code_bytes[0] = 16;
        }

        if tokens[0] == "#fun" {
            let fun_name = tokens[1];
            if fun_name == "main" {
                println!("set the main index {}", self.code_index);
                self.code_begin_index = self.code_index.clone();
            }
            self.fun_label_map.insert(fun_name.to_string(), self.code_index.clone());
            return;
        }

        // const type value
        if tokens[0] == "const" {

            code_bytes[0] = 2;
            let type_name = tokens[1];
            let value_str = tokens[2];


            if type_name == "i32" {
                code_bytes[1] = 4;

                let code_instrust = u32::from_be_bytes(code_bytes);

                self.code_vec.push(code_instrust);
                self.code_index +=1;

                let value = value_str.trim().parse::<i32>().unwrap();
                let byte_u32 = u32::from_be_bytes(value.to_be_bytes());
                self.code_vec.push(byte_u32);
                self.code_index +=1;
            }

            if type_name == "f32" {
                code_bytes[1] = 4;

                let code_instrust = u32::from_be_bytes(code_bytes);

                self.code_vec.push(code_instrust);
                self.code_index +=1;

                let value = value_str.trim().parse::<f32>().unwrap();
                let byte_u32 = u32::from_be_bytes(value.to_be_bytes());
                self.code_vec.push(byte_u32);
                self.code_index +=1;
            }

            if type_name == "i64" {
                code_bytes[1] = 8;

                let code_instrust = u32::from_be_bytes(code_bytes);

                self.code_vec.push(code_instrust);
                self.code_index +=1;

                let value = value_str.trim().parse::<i64>().unwrap();
                let bytes_u32_vec:[u8;8] = value.to_be_bytes();
                let byte_u32_1 = u32::from_be_bytes(read4byte(&bytes_u32_vec[0..4]));
                self.code_vec.push(byte_u32_1);
                self.code_index +=1;

                let byte_u32_2 = u32::from_be_bytes(read4byte(&bytes_u32_vec[4..8]));
                self.code_vec.push(byte_u32_2);
                self.code_index +=1;
            }

            if type_name == "f64" {
                code_bytes[1] = 8;

                let code_instrust = u32::from_be_bytes(code_bytes);

                self.code_vec.push(code_instrust);
                self.code_index +=1;

                let value:f64 = value_str.trim().parse::<f64>().unwrap();
                let mut bytes_u32_vec:[u8;8] = value.to_be_bytes();
                let byte_u32_1 = u32::from_be_bytes(read4byte(&bytes_u32_vec[0..4]));
                self.code_vec.push(byte_u32_1);
                self.code_index +=1;

                let byte_u32_2 = u32::from_be_bytes(read4byte(&bytes_u32_vec[4..8]));
                self.code_vec.push(byte_u32_2);
                self.code_index +=1;
            }

            if type_name == "bool" {
                code_bytes[1] = 1;

                let code_instrust = u32::from_be_bytes(code_bytes);

                self.code_vec.push(code_instrust);
                self.code_index +=1;

                let mut value_byte:[u8;4] = [0;4];
                let value = value_str.trim().parse::<bool>().unwrap();
                if value {
                    value_byte[0] = 1;
                }
                let byte_u32 = u32::from_be_bytes(value_byte);
                self.code_vec.push(byte_u32);
                self.code_index +=1;

            }

            println!("instrust {} bytecode {:?}", cmd_line, code_bytes);

        } else {
            let code_instrust = u32::from_be_bytes(code_bytes);

            self.code_vec.push(code_instrust);
            self.code_index +=1;
        }

        println!("code index {}", self.code_index);


    }


    pub fn writeCodeClass(&mut self) {

        let class_file_name = self.file_name.replace(".fycs", ".fyb");
        let class_file_result = File::create(class_file_name.to_string());

        if class_file_result.is_err() {
            panic!("error to create class file {}", class_file_name);
        }

        let mut class_file:File = class_file_result.unwrap();

        let version_byte = self.version.to_be_bytes();
        class_file.write(&version_byte);

        let main_index_byte = self.code_begin_index.to_be_bytes();
        class_file.write(&main_index_byte);

        let mut code_write_line = 0;
        for code in &self.code_vec {
            let mut code_bytes = code.to_be_bytes();

            if code_bytes[0] == 12 || code_bytes[0] == 13 {
                let mut  byte_for_line:[u8;2] = [0;2];
                byte_for_line[0] = code_bytes[1];
                byte_for_line[1] = code_bytes[2];

                let line_num = u16::from_be_bytes(byte_for_line) as usize;
                let go_code_line = self.code_line_map.get(&line_num).unwrap().clone() as u16;

                let go_code_bytes = go_code_line.to_be_bytes();
                code_bytes[1] = go_code_bytes[0];
                code_bytes[2] = go_code_bytes[1];

            }

            if code_bytes[0] == 14 {
                let fun_name = self.call_map.get(&code_write_line).unwrap().to_string();
                let fun_index = self.fun_label_map.get(&fun_name).unwrap().clone() as u16;
                let fun_index_bytes = fun_index.to_be_bytes();
                code_bytes[1] = fun_index_bytes[0];
                code_bytes[2] = fun_index_bytes[1];
            }
            class_file.write(&code_bytes);
            code_write_line +=1;
        }

    }
}

pub fn read4byte(array:&[u8]) -> [u8;4] {
    let mut result:[u8;4] = [0;4];
    result[0] = array[0];
    result[1] = array[1];
    result[2] = array[2];
    result[3] = array[3];

    return result;
}

pub fn read8byte(array:&[u8]) -> [u8;8] {
    let mut result:[u8;8] = [0;8];
    result[0] = array[0];
    result[1] = array[1];
    result[2] = array[2];
    result[3] = array[3];

    result[4] = array[4];
    result[5] = array[5];
    result[6] = array[6];
    result[7] = array[7];

    return result;
}



#[cfg(test)]
mod tests {
    use super::code_machine;
    use super::conse_execute_machine;

    #[test]
    fn test_simple_code() {
        let mut code_machine_instance = code_machine::new("/Users/liufoyang/opensource/rust_demo-master/consescript/test_data/demo.fycs");
        code_machine_instance.simple_code_to_bytes();
        code_machine_instance.writeCodeClass();

        let mut execute_machine =  conse_execute_machine::new("/Users/liufoyang/opensource/rust_demo-master/consescript/test_data/demo.fycs");
        execute_machine.execute_instruct();
    }

    #[test]
    fn test_simple_code2() {

        let value = "100".trim().parse::<i32>().unwrap();
        let mut code_bytes = value.to_be_bytes();

        let code_instrust = u32::from_be_bytes(code_bytes);

        code_bytes = code_instrust.to_be_bytes();

        let value2 = i32::from_be_bytes(code_bytes);

        println!("change value compare {}, {}", value, value2);

    }
}