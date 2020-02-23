use std::fs::File;
use std::fs::OpenOptions;
use std::str::FromStr;
use std::string::String;
use std::vec::Vec;
use std::io::{self, Write, Read, Seek, SeekFrom};
use std::convert::Infallible;
use std::result::Result;
use super::threadpool::ThreadPool;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::collections::HashMap;
use flexi_logger::{Logger, opt_format};
use log::*;
use super::conse_block::Conse_Block;
use super::conse_block::Block_Header;
use super::conse_block::Block_Tree;
//use std::convert::From::from;

///只是做一个直接写log，写businessfile的保存命令结果。
//
pub struct Command_Executor {
    threadpools:ThreadPool,
    msglogfiles:File,
    busifiles:File,
    blockfile:File,
    datafileName:String,
    size:usize,
    valueMap:HashMap<String, String>,
    block_chain:Vec<Conse_Block>,
    block_index:usize,
}

impl Command_Executor {
    pub fn new(msglogfile_name:&str, pre_file_name:&str) ->Command_Executor {
        let mut busifile_name = String::from(pre_file_name);
        let mut datafile_name = String::from(pre_file_name);
        let mut blockfile_name = String::from(pre_file_name);

        blockfile_name.push_str("block.dat");
        datafile_name.push_str("data.dat");
        busifile_name.push_str("busi.dat");

        //let mut pools_vec = Vec::with_capacity(_size);
        let mut logfile_result = OpenOptions::new().append(true).open(msglogfile_name.to_string());
        if logfile_result.is_err() {
            logfile_result = File::create(msglogfile_name.to_string());
        }

        let logfile: File = logfile_result.unwrap();

        let mut busifile_result = OpenOptions::new().append(false).open(busifile_name.clone());
        if busifile_result.is_err() {
            busifile_result  = File::create(busifile_name.clone());
        }
        let busifile:File = busifile_result.unwrap();

        let mut blockfile_result = OpenOptions::new().append(true).open(blockfile_name.clone());
        if blockfile_result.is_err() {
            blockfile_result  = File::create(blockfile_name.clone());
        }
        let blockfile:File = blockfile_result.unwrap();

        let mut executor =  Command_Executor {
            threadpools:ThreadPool::new(1),
            msglogfiles:logfile,
            busifiles:busifile,
            datafileName:datafile_name,
            blockfile:blockfile,
            size:1,
            valueMap:HashMap::new(),
            block_chain:Vec::new(),
            block_index:0
        };

        // load blockchain file and
        executor.load();
        return executor;
    }

    // record the msg log 记录消息日志
    pub fn savelog(&mut self,payload: &str) {
        let v: Vec<&str> = payload.split(' ').collect();


        // TODO 计算key的hash取模匹配文件index值, 分组消息日志

        let mut s = String::from(payload);
        s.push_str("\n");
        let mut  buf = s.as_ref();
        self.msglogfiles.write(buf);
        self.msglogfiles.flush();

        info!("write the log file {}", s);


    }

    /// the command for key value, 3 command put key = value, delete key, get key
    /// key is path split by /,  value is string max length 1024;
    pub fn command_execute(&mut self,command: &str) -> Option<String> {

        let mut result:Option<String> = Option::None;

        let mut commandKey = "";

        if command.starts_with("put") {
            commandKey = "put";
        }else if command.starts_with("delete") {
            commandKey = "delete";
        }else if command.starts_with("get") {
            commandKey = "get";
        } else {
            info!("not valid command {}", command);
            return result;
        }

        let mut keyValueStr = command.replace(commandKey, "");

        if commandKey== "put" && !keyValueStr.contains("=") {
            info!("not valid command {}", command);
            return result;
        }
        let playloads:Vec<&str> = keyValueStr.split('=').collect();

        let key = playloads[0].trim();

        let mut out = String::from(command);
        out.push_str("\n");
        let mut  buf = out.as_ref();
        self.busifiles.write(buf);
        self.busifiles.flush();
        info!("write the business file {}", command);


        let keyStr = key.to_string();
        if commandKey == "put" {
            let value = playloads[1].trim();
            self.valueMap.insert(key.to_string(), value.to_string());
        } else if commandKey == "delete" {
            let delete_result = self.valueMap.remove(&keyStr);
            if delete_result.is_some() {
                result = Some(delete_result.unwrap());
            } else {
                return Some("".to_string());
            }
            return result;
        }

        if self.valueMap.contains_key(&keyStr) {
            let value_str = self.valueMap.get(&keyStr).unwrap();
            result = Some(value_str.to_string());
        } else {
            return Some("".to_string());
        }

        return result;

    }

    pub fn save_to_block_chain(&mut self,command: &str, node_id:&str, seq_num:u64)->Option<String> {
        info!("save to block chain begin");
        if self.block_chain.len() == 0 {
            let block = Conse_Block::new("first_block", node_id, seq_num);
            self.block_chain.push(block);
        }
        let mut last_block = self.block_chain.pop().unwrap();
        let add_result = last_block.addTrans(command);

        if add_result.is_none() {

            if last_block.isHashed() {
                let mut block = Conse_Block::new(last_block.get_hash_id().as_str(), node_id, seq_num);
                self.block_chain.push(last_block);
                //self.block_chain.push(block);

                let add_result = block.addTrans(command);
                self.block_chain.push(block);
                info!("save to block chain {} end {:?}", self.block_chain.len(), add_result);
                return add_result;
            } else {

                self.block_chain.push(last_block);
                info!("save to block chain error {:?}", add_result);
                return add_result;
            }
        } else {
            self.block_chain.push(last_block);
            info!("save to block chain end {:?}", add_result);
            return add_result;
        }


    }
    pub fn save_check_point(&mut self,check_point_num: &u64) -> Option<String> {

        let check_point_file_index = ((*check_point_num)%2) as usize;
        let mut fileName = String::from(self.datafileName.as_str());
        fileName.push_str(check_point_file_index.to_string().as_str());
        //fileName.push_str(".log");

        let file_result = File::create(fileName.as_str());
        if !file_result.is_ok() {
            return None;
        }

        let mut file = file_result.unwrap();
        for (key, value) in self.valueMap.iter() {
            let mut line = String::from(key.as_str());
            line.push_str("=");
            line.push_str(value.as_str());
            line.push_str("\n");

            file.write_all(line.as_str().as_bytes());
        }

        file.flush();

        // save the command
        let mut add_check_point_line = String::from("checkpoint ");
        add_check_point_line.push_str(check_point_file_index.to_string().as_str());
        add_check_point_line.push_str("\n");

        let mut buf = add_check_point_line.as_str().as_bytes();
        self.busifiles.write(buf);
        self.busifiles.flush();


        // save block to block file.
        let mut begin_index = self.block_index.clone();
        let mut end_index = self.block_chain.len();

        info!("begin to write the block file {} {}", begin_index, end_index);
        for index in begin_index..end_index {
            let mut block = self.block_chain.get_mut(index).unwrap();
            if !block.isHashed() {
                block.hashSign();
            }
            let mut header = block.get_header().clone();
            let header_buffer = header.as_bytes();

            let mut body = block.get_body().clone();
            let body_buffer = body.as_bytes();

            let header_len = header_buffer.len();
            let body_len = body_buffer.len();

            self.blockfile.write(&header_len.to_be_bytes());
            self.blockfile.write(header_buffer);

            self.blockfile.write(&body_len.to_be_bytes());

            self.blockfile.write(body_buffer);

            self.blockfile.flush();
            info!("write block to file {} {}", header_len, body_len);
        }

        self.block_index = end_index;
        return Some("add check point".to_string());
    }

    fn load(&mut self) {

        while true {
            self.read_block_bufer();
            let header_size = self.read_block_bufer();

            if header_size <= 0 {
                warn!("no size for header len {}", header_size);
                break;
            }

            let mut header_buf:Vec<u8> = Vec::with_capacity(header_size);
            self.blockfile.read(&mut header_buf);

            let block_header = Block_Header::from_bytes(header_buf.as_mut_slice());

            let body_size = self.read_block_bufer();

            if body_size <= 0 {
                break;
            }

            let mut body_buf:Vec<u8> = Vec::with_capacity(body_size);
            self.blockfile.read(&mut body_buf);
            let block_body = Block_Tree::from_bytes(body_buf.as_mut_slice());

            self.block_chain.push(Conse_Block::from(block_header, block_body));
            self.block_index +=1;
        }

        let mut block = self.block_chain.get_mut(self.block_index-1).unwrap();
        let mut body = block.get_body();

        let buckets = body.get_buckets();

        let mut commands = Vec::new();
        for bucket in buckets{
            let datas  =  bucket.get_datas();
            for data in datas {
                commands.push(data.clone());
            }
        }

        for command in commands {
            self.command_execute(command.as_str());
        }
        info!("load the block index {}", self.block_index);
    }

    fn read_block_bufer(&mut self) -> usize {
        let mut len_buf: [u8; 8] = [0; 8];
        let result = self.blockfile.read(&mut len_buf);
        if !result.is_ok() {
            info!("read the block file end {:?}", len_buf);
            return 0;
        }
        if result.unwrap() < 8 {
            info!("read the block file end {:?}", len_buf);
            return 0;
        }
        let size = usize::from_be_bytes(len_buf);
        return size;
    }
}






