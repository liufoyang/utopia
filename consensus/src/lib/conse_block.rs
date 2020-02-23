extern crate crypto;
use crypto::digest::Digest;
use crypto::md5::Md5;
use std::time::SystemTime;
use std::time::Duration;
use flexi_logger::{Logger, opt_format};
use log::*;


#[derive(Clone)]
pub struct Bucket {
    hash_id:String,
    data:Vec<String>,
    byte_buffer:Vec<u8>
}

#[derive(Clone)]
pub struct Block_Tree {
    hash_id_vec: Vec<String>,
    bucket_vec:Vec<Bucket>,
    bytes_buffer:Vec<u8>,
}

impl Bucket {
    pub fn addTrans(&mut self, trans_command:&str) -> Option<String> {
        if self.data.len() >= 128 {
            return None;
        }

        self.data.push(trans_command.to_string());

        return Some("success".to_string());
    }

    pub fn new() -> Bucket{
        let bucket = Bucket {
            hash_id:"".to_string(),
            data: Vec::new(),
            byte_buffer:Vec::new()
        };

        return bucket;
    }

    pub fn dohash(&mut self) -> String {

        let mut buffer_size = 0;
        for bucket in &self.data {
            buffer_size +=2;
            buffer_size +=bucket.as_bytes().len();
        }

        let mut index = 0 as usize;
        let mut data_buffer:Vec<u8> = Vec::new();
        for bucket in &self.data {
            let size = bucket.as_bytes().len() as u16;
            println!("save bucket size {}", size);
            let mut  size_byte = size.to_be_bytes().clone();
            data_buffer.push(size_byte[0]);
            data_buffer.push(size_byte[1]);
            data_buffer.extend_from_slice(bucket.as_bytes());
            println!("save the data buffer {:?}",data_buffer);
        }

        let mut md5 = Md5::new();
        md5.input(data_buffer.as_slice());
        let hash_id = md5.result_str();

        //println!("save the data buffer {:?}",data_buffer);

        self.hash_id = hash_id;

        let mut buf = self.hash_id.as_bytes().clone();
        let id_hash_size = buf.len() as u16;
        let mut  size_byte = id_hash_size.to_be_bytes();
        self.byte_buffer.extend_from_slice(&mut size_byte);
        self.byte_buffer.extend_from_slice(buf);
        self.byte_buffer.extend_from_slice(data_buffer.as_mut());

        return self.hash_id.clone();
    }

    pub fn as_bytes(&mut self) -> &[u8] {
        if self.byte_buffer.len() == 0 {
            self.dohash();
        }
        return self.byte_buffer.as_slice();
    }

    pub fn from_bytes(byte_buffer:&[u8]) ->Bucket{
        let mut len_vec:[u8;2] = [0;2];
        let mut index = 0 as usize;

        len_vec[0] = byte_buffer[index];
        len_vec[1] = byte_buffer[index+1];
        let mut buf_len = u16::from_be_bytes(len_vec) as usize;

        println!("hash buffer len {}", buf_len);
        index +=2;
        // hash_id
        let hash_id = String::from_utf8_lossy(&byte_buffer[index..(index + buf_len)]).to_string();
        index += buf_len;
        let mut data_vec = Vec::new();
        while index<byte_buffer.len() {
            len_vec[0] = byte_buffer[index];
            len_vec[1] = byte_buffer[index+1];
            buf_len = u16::from_be_bytes(len_vec) as usize;
            println!("data buffer len {}", buf_len);
            index +=2;
            let command = String::from_utf8_lossy(&byte_buffer[index..(index+buf_len)]).to_string();
            data_vec.push(command);
            index+=buf_len;
        }

        let mut buffer = Vec::new();
        buffer.extend_from_slice(byte_buffer);
        let bucket = Bucket {
            hash_id:hash_id,
            data: data_vec,
            byte_buffer:buffer
        };
        return bucket;
    }

    pub fn get_datas(&self) ->&Vec<String> {
         return &self.data;
    }
}


impl Block_Tree {
    pub fn new() -> Block_Tree {
        Block_Tree {
            hash_id_vec:Vec::new(),
            bucket_vec:Vec::new(),
            bytes_buffer:Vec::new(),
        }
    }

    pub fn bucket_size(&self) -> usize {
        return self.bucket_vec.len();
    }
    pub fn addTrans(&mut self, trans_command:&str) -> Option<String> {

         if self.bucket_vec.len() < 1 {
             self.bucket_vec.push(Bucket::new());
         }

         let mut bucket = self.bucket_vec.pop().unwrap();
         let mut result = bucket.addTrans(trans_command);

         self.bucket_vec.push(bucket);

         if result.is_none() {

             if self.bucket_vec.len() < 128 {
                 let mut bucket = Bucket::new();
                 result = bucket.addTrans(trans_command);

                 self.bucket_vec.push(bucket);
             } else {
                 return result;
             }
         }

        return result;
    }

    pub fn doHashTree(&mut self) -> Option<String>{
        let base_len = self.bucket_vec.len();

        if base_len == 0 {
            error!("empty block");
            return None;
        }

        if base_len == 1 {
            info!("just on bucket block");
            self.hash_id_vec = vec![String::new(); 1];
            let buckeA = self.bucket_vec.get_mut(0).unwrap();
            let aHash = buckeA.dohash();
            let mut md5 = Md5::new();
            md5.input(aHash.as_bytes());
            let hash_id = md5.result_str();
            self.hash_id_vec[0] = hash_id;

            return Some(self.hash_id_vec[0].clone());
        }

        let mut hash_vec_len = base_len - 1 as usize;


        let mut total_len = 0 as usize;
        while hash_vec_len > 0 {
            total_len += hash_vec_len;
            hash_vec_len -=1;
        }

        self.hash_id_vec = vec![String::new(); total_len];
        let mut md5 = Md5::new();
        for index in 0..(base_len - 1) {
            let buckeA = self.bucket_vec.get_mut(index).unwrap();
            let aHash = buckeA.dohash();

            let buckeB = self.bucket_vec.get_mut(index+1).unwrap();
            let bHash = buckeB.dohash();

            md5.input(aHash.as_bytes());
            md5.input(bHash.as_bytes());

            let hash_id = md5.result_str();
            md5.reset();

            self.hash_id_vec[index] = hash_id;
        }

        let mut has_len = base_len -2;
        let mut begin_index =0;
        while has_len > 0 {
            for index in begin_index..(begin_index+has_len) {
                let aHash = &self.hash_id_vec[index];
                let bHash = &self.hash_id_vec[index+1];

                md5.input(aHash.as_bytes());
                md5.input(bHash.as_bytes());

                let hash_id = md5.result_str();
                md5.reset();

                self.hash_id_vec[index+has_len+1] = hash_id;
            }

            begin_index += has_len;
            has_len -=1;
        }

        let hash_id = self.hash_id_vec[total_len-1].clone();

        return Some(hash_id);

    }

    pub fn as_bytes(&mut self) -> &[u8] {

        if self.hash_id_vec.len() == 0 || self.bytes_buffer.len() > 0 {
            return self.bytes_buffer.as_slice();
        } else {

            //
            let hash_id_num = self.hash_id_vec.len() as u16;
            let mut  buf = hash_id_num.to_be_bytes();
            self.bytes_buffer.extend_from_slice(&mut buf);

            /// hash_id_vec
           /// bucket_vec
            for hash_id in &self.hash_id_vec {
                let mut buf = hash_id.as_bytes().clone();
                let size = buf.len() as u16;
                let mut  size_byte = size.to_be_bytes();
                self.bytes_buffer.extend_from_slice(&mut size_byte);


                self.bytes_buffer.extend_from_slice(buf);
            }

            for bucket in &mut self.bucket_vec {
                let mut buf = bucket.as_bytes().clone();
                let size = buf.len() as u32;
                let mut  size_byte = size.to_be_bytes();
                self.bytes_buffer.extend_from_slice(&mut size_byte);
                self.bytes_buffer.extend_from_slice(buf);
            }

            return self.bytes_buffer.as_slice();
        }

    }

    pub fn from_bytes(buffer:&[u8]) -> Block_Tree {
        println!("the buffer len {}", buffer.len());
        let mut index = 0;
        let mut len_vec:[u8;2] = [0;2];
        len_vec[0] = buffer[index];
        len_vec[1] = buffer[index+1];

        println!("the hash buffer {:?}", len_vec);
        let hash_num = u16::from_be_bytes(len_vec) as usize;
        let hash_num_len = hash_num as usize;
        index +=2;

        println!("the hash num len {}", hash_num_len);
        let mut hash_id_vec = Vec::new();

        for i in 0..hash_num_len {
            len_vec[0] = buffer[index];
            len_vec[1] = buffer[index+1];
            let buf_len = u16::from_be_bytes(len_vec) as usize;
            index += 2;

            hash_id_vec.push(String::from_utf8_lossy(&buffer[index..(index+buf_len)]).to_string());

            index += buf_len;
        }

        let mut  bucket_vec = Vec::new();

        while index <buffer.len() {


            let mut  buf8:[u8;4] = [0;4];
            buf8[0] = buffer[index].clone();
            buf8[1] = buffer[index +1].clone();
            buf8[2] = buffer[index +2].clone();
            buf8[3] = buffer[index +3].clone();
            index += 4;

            let bucket_len = u32::from_be_bytes(buf8) as usize;
            let buckt_buf =  &buffer[index..(index +bucket_len)];

            let bucket = Bucket::from_bytes(buckt_buf);
            bucket_vec.push(bucket);
            index += bucket_len;
        }

        let mut byte_buffer = Vec::new(); //buffer.clone();
        byte_buffer.extend_from_slice(& buffer);
        let block_tree = Block_Tree {
            hash_id_vec: hash_id_vec,
            bucket_vec:bucket_vec,
            bytes_buffer:byte_buffer
        };

        return block_tree;
    }

    pub fn get_buckets(&self) -> &Vec<Bucket> {
        return &self.bucket_vec;
    }

}

#[derive(Clone)]
pub struct Block_Header {
    parent_id:String,
    block_id:String,
    block_len:usize,
    auth_id: String,
    timestamp:String,
    tree_root:String,
    last_seq_num:u64,
    sign:String,
    bytes_buffer:Vec<u8>,
}

impl Block_Header {
    pub fn new(_parent_id:&str, _auth_id:&str, _tree_root:&str, _last_seq_num:u64) ->Block_Header {
        let times = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let timestamp = times.as_millis().to_string();
        let block_header = Block_Header{
            parent_id:_parent_id.to_string(),
            block_id:String::new(),
            block_len:0,
            auth_id: _auth_id.to_string(),
            timestamp:timestamp,
            tree_root:_tree_root.to_string(),
            last_seq_num:_last_seq_num,
            sign:String::new(),
            bytes_buffer:Vec::new()
        };

        return block_header;
    }

    pub fn hashSign(&mut self) {
        let mut md5 = Md5::new();
        md5.input(self.parent_id.as_bytes());
        md5.input(&self.block_len.to_be_bytes());
        md5.input(self.auth_id.as_bytes());
        md5.input(self.timestamp.as_bytes());
        md5.input(self.tree_root.as_bytes());
        md5.input(&self.last_seq_num.to_be_bytes());
        let hash_id = md5.result_str();
        md5.reset();

        self.block_id = hash_id;

    }

    pub fn as_bytes(&mut self) -> &[u8] {

        if self.block_id.len() == 0 {
            return self.bytes_buffer.as_slice();
        }

        ///
        /// parent_id:String,
        /// block_id:String,
        /// block_len:usize,
        /// auth_id: String,
        /// timestamp:String,
        /// tree_root:String,
        /// last_seq_num:u64,

        let mut total_str = String::new();
        total_str.push_str(self.parent_id.as_str());
        total_str.push_str("#");
        total_str.push_str(self.block_id.as_str());
        total_str.push_str("#");
        total_str.push_str(self.block_len.to_string().as_str());
        total_str.push_str("#");
        total_str.push_str(self.auth_id.as_str());
        total_str.push_str("#");
        total_str.push_str(self.timestamp.as_str());
        total_str.push_str("#");
        total_str.push_str(self.tree_root.as_str());
        total_str.push_str("#");
        total_str.push_str(self.last_seq_num.to_string().as_str());

        let buf = total_str.as_bytes();
        self.bytes_buffer.extend_from_slice(buf);
        return self.bytes_buffer.as_slice();
    }

    pub fn from_bytes(buffer:&[u8]) ->Block_Header {

        let total_str = String::from_utf8_lossy(buffer).to_string();

        let fields:Vec<&str> = total_str.split("#").collect();

        /// parent_id:String,
        /// block_id:String,
        /// block_len:usize,
        /// auth_id: String,
        /// timestamp:String,
        /// tree_root:String,
        /// last_seq_num:u64,

        let parent_id = fields[0].to_string();
        let block_id = fields[1].to_string();
        let block_len_str = fields[2].to_string();
        let block_len = usize::from_str_radix(block_len_str.as_str(),10).unwrap();

        let auth_id = fields[3].to_string();

        let timestamp = fields[4].to_string();
        let tree_root = fields[5].to_string();
        let last_seq_num_str = fields[6].to_string();
        let last_seq_num = u64::from_str_radix(last_seq_num_str.as_str(), 10).unwrap();

        let mut bytes_buffer = Vec::new();
        bytes_buffer.extend_from_slice(& buffer);
        let block_header = Block_Header{
            parent_id:parent_id,
            block_id:block_id,
            block_len: block_len,
            auth_id: auth_id,
            timestamp:timestamp,
            tree_root:tree_root,
            last_seq_num:last_seq_num,
            sign:String::new(),
            bytes_buffer:bytes_buffer
        };

        return block_header;

    }
}

#[derive(Clone)]
pub struct Conse_Block {
    header:Block_Header,
    body:Block_Tree,
    has_hash:bool,
}

impl Conse_Block {
    pub fn from(_header:Block_Header, _body:Block_Tree) -> Conse_Block{
        let conse_block = Conse_Block {
            header:_header,
            body:_body,
            has_hash:true
        };

        return conse_block;
    }
    pub fn new(_parent_id:&str, _node_id:&str, _seq_num:u64) -> Conse_Block {
        let _header = Block_Header::new(_parent_id, _node_id, "", _seq_num);
        let _body = Block_Tree::new();

        let conse_block = Conse_Block {
            header:_header,
            body:_body,
            has_hash:false
        };

        return conse_block;
    }

    pub fn addTrans(&mut self, trans_command:&str) -> Option<String> {
        if self.has_hash{
            return None;
        }
        return self.body.addTrans(trans_command);
    }

    pub fn hashSign(&mut self) -> Option<String>{
        let body_hash_result = self.body.doHashTree();
        if body_hash_result.is_none() {
            return None;
        }

        self.header.tree_root = body_hash_result.unwrap();
        self.header.block_len = self.body.as_bytes().len();
        self.header.hashSign();
        self.has_hash = true;

        return Some("success".to_string());
    }

    pub fn isHashed(&self) -> bool {
        return self.has_hash.clone();
    }

    pub fn get_hash_id(&self) -> String {
        return self.header.block_id.clone();
    }

    pub fn get_header(&mut self) -> &mut Block_Header {
        return &mut self.header;
    }
    pub fn get_body(&mut self) -> &mut Block_Tree {
        return &mut self.body;
    }
}


#[cfg(test)]
mod tests {
    use super::Bucket;
    use super::Conse_Block;
    use super::Block_Tree;

    #[test]
    fn test_block_one() {
        let mut conse_block = Conse_Block::new("first_block", "test_node", 100);

        for i in 0..100 {
            let command = i.to_string();
            conse_block.addTrans(command.as_str());
        }

        conse_block.hashSign();

        let mut header =  conse_block.get_header();
        //println!("the block header {:?}", header.as_bytes());

        let mut body =  conse_block.get_body();

        assert_eq!(1, body.bucket_size());
    }

    #[test]
    fn test_block_two() {
        let mut conse_block = Conse_Block::new("first_block", "test_node", 100);

        for i in 0..130 {
            let command = i.to_string();
            conse_block.addTrans(command.as_str());
        }

        conse_block.hashSign();

        let mut header =  conse_block.get_header();
        println!("the block header id {}", header.block_id);

        let mut body =  conse_block.get_body();


        assert_eq!(2, body.bucket_size());
    }

    #[test]
    fn test_block_too_block() {
        let mut conse_block = Conse_Block::new("first_block", "test_node", 100);

        for i in 0..16386 {
            let command = i.to_string();
            conse_block.addTrans(command.as_str());
        }

        conse_block.hashSign();

        let mut header =  conse_block.get_header();
        println!("the block header id {:?}", header.block_id);

        let mut body =  conse_block.get_body();

        assert_eq!(128, body.bucket_size());
    }

    #[test]
    fn test_ser_unser_bucket() {

        let mut bucket = Bucket::new();
        bucket.addTrans("first command");
        bucket.addTrans("second command");

        bucket.dohash();

        let byte_buffer = bucket.as_bytes();
        println!("bucket byte {:?}", byte_buffer);

        let second = Bucket::from_bytes(byte_buffer);
        assert_eq!(second.get_datas().len(), bucket.get_datas().len());

    }
    #[test]
    fn test_ser_unser() {
        let mut conse_block = Conse_Block::new("first_block", "test_node", 100);

        for i in 0..1024 {
            let command = i.to_string();
            conse_block.addTrans(command.as_str());
        }

        conse_block.hashSign();

        let mut header =  conse_block.get_header();

        println!("the block header id {:?}", header.block_id);

        let mut body =  conse_block.get_body();
        println!("the block {:?}",body.as_bytes());

        let mut body_too = Block_Tree::from_bytes(body.as_bytes());
        println!("the block buckets len {}", body_too.get_buckets().len());
        println!("the block buckets len {}", body.get_buckets().len());
        assert_eq!(body_too.get_buckets().len(), body.get_buckets().len());
    }
}