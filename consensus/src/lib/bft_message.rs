/// client source msg to bft node
/// the primiry receive the node and begin to prepare phase
use std::vec::Vec;
use super::bft_node::Btf_Node_Simple;
extern crate rustc_serialize;
// 引入rustc_serialize模块
use rustc_serialize::json;
use std::time::SystemTime;
extern crate crypto;
use crypto::md5::Md5;
use crypto::digest::Digest;

#[derive(RustcDecodable, RustcEncodable)]
#[derive(Clone)]
pub struct Bft_Message {
    id:String,
    client_id:String,
    payload:String,
    status:i32,
    timestamp:u64,
    md5sign: String,
    client_ip:String,
    port:String,
}

impl Bft_Message {
    pub fn new(_payload: &str, _client_id:&str, _ip:&str, _port:&str) ->Bft_Message{
        let msg =  Bft_Message{
            payload:_payload.to_string(),
            client_id: _client_id.to_string(),
            id: "1232345".to_string(),
            timestamp:100000,
            status:1,
            md5sign:String::new(),
            client_ip:_ip.to_string(),
            port:_port.to_string(),
        };
        return msg;
    }

    pub fn sign(&self, private_key:&str, public_key:&str) -> String{
        return public_key.to_string();
    }

    pub fn signMd5(&self) -> String{

        let mut sh = Md5::new();
        sh.input_str(self.get_payload());
        let md5_result = sh.result_str();
        let result = md5_result.clone();
        return result;
    }

    pub fn get_id(&self) -> &str {
        return self.id.as_str()
    }

    pub fn get_payload(&self) ->&str {
        return self.payload.as_str()
    }

    pub fn set_status(&mut self, _status:i32) {
        self.status = _status;
    }

    pub fn get_status(&self) -> i32{
        return self.status.clone();
    }

    pub fn get_client_ip(&self) ->&str {
        return self.client_ip.as_str()
    }

    pub fn get_port(&self) ->&str {
        return self.port.as_str()
    }
}

///  use to contain the byte[] of Bft_Message
///  do the hash for Bft_Message
#[derive(Clone)]
pub struct Bft_Message_Bytes<'a>(pub &'a [u8]);

/// the replay to client from Bft node
///  every Bft node non fault will send the repay;
#[derive(RustcDecodable, RustcEncodable)]
#[derive(Clone)]
pub struct Bft_Replay{
    view_num:u64,
    timestamp:u64,
    client_id:String,
    node_id:u64,
    payload:String
}

impl Bft_Replay {
    pub fn new(_view_num:u64, _payload: &str, _node_id:u64, _source_msg:Bft_Message)->Bft_Replay {
        let replay = Bft_Replay {
            view_num:_view_num,
            timestamp:_source_msg.timestamp,
            client_id:_source_msg.client_id,
            node_id:_node_id,
            payload:_payload.to_string()
        };
        return replay;
    }
}

pub struct Bft_PrePrepare_Simple {
    pub view_num:u64,
    pub msg_sign:String,
    pub time:SystemTime
}
#[derive(RustcDecodable, RustcEncodable)]
#[derive(Clone)]
pub struct Bft_PrePrepare_Message {
    view_num:u64,
    node_id:u64,
    sequence_num:u64,
    msg_digest:String,
    client_msg:Bft_Message,
}

impl Bft_PrePrepare_Message {
    pub fn new(_view_num:u64, _node_id:u64, _sequence_num: u64,_source_msg:Bft_Message)->Bft_PrePrepare_Message {


        let msg = Bft_PrePrepare_Message {
            view_num:_view_num,
            node_id:_node_id,
            sequence_num:_sequence_num,
            msg_digest:_source_msg.sign("", ""),
            client_msg:_source_msg,
        };
        return msg;
    }

    pub fn get_view_num(&self) -> u64 {
        return self.view_num.clone();
    }

    pub fn get_sequence_num(&self) -> u64 {
        return self.sequence_num.clone();
    }

    pub fn get_client_msg(&self) -> &Bft_Message {
        return &(self.client_msg);
    }

    pub fn get_msg_digest(&self) -> String {
        return (self.msg_digest.clone());
    }

    pub fn set_msg_digest(&mut self, sign:String) {
        self.msg_digest = sign;
    }

    pub fn get_node_id(&self) -> u64 {
        return self.node_id.clone();
    }

}
#[derive(RustcDecodable, RustcEncodable)]
#[derive(Clone)]
pub struct Bft_Prepare_Message {
    view_num:u64,
    sequence_num:u64,
    msg_digest:String,
    node_id:u64
}

impl Bft_Prepare_Message {
    pub fn new(_view_num:u64, _sequence_num:u64, _node_id:u64 ) ->Bft_Prepare_Message {
        let msg = Bft_Prepare_Message {
            view_num:_view_num,
            sequence_num:_sequence_num,
            msg_digest: String::new(),
            node_id:_node_id
        };
        return msg;
    }

    pub fn get_view_num(&self) -> u64 {
        return self.view_num.clone();
    }

    pub fn get_sequence_num(&self) -> u64 {
        return self.sequence_num.clone();
    }

    pub fn get_msg_digest(&self) -> String {
        return (self.msg_digest.clone());
    }

    pub fn set_msg_digest(&mut self, sign:String) {
        self.msg_digest = sign;
    }

    pub fn get_node_id(&self) -> u64 {
        return self.node_id.clone();
    }
}
#[derive(RustcDecodable, RustcEncodable)]
#[derive(Clone)]
pub struct Bft_Commit_Message {
    view_num:u64,
    sequence_num:u64,
    msg_digest:String,
    node_id:u64
}

impl Bft_Commit_Message {
    pub fn new(_view_num:u64, _sequence_num:u64, _node_id:u64 ) ->Bft_Commit_Message {
        let msg = Bft_Commit_Message {
            view_num:_view_num,
            sequence_num:_sequence_num,
            msg_digest: String::new(),
            node_id:_node_id
        };
        return msg;
    }
    pub fn get_view_num(&self) -> u64 {
        return self.view_num.clone();
    }

    pub fn get_sequence_num(&self) -> u64 {
        return self.sequence_num.clone();
    }

    pub fn get_msg_digest(&self) -> String {
        return (self.msg_digest.clone());
    }

    pub fn set_msg_digest(&mut self, sign:String) {
        self.msg_digest = sign;
    }

    pub fn get_node_id(&self) -> u64 {
        return self.node_id.clone();
    }
}

#[derive(RustcDecodable, RustcEncodable)]
#[derive(Clone)]
pub struct Bft_View_Change_Message {
    view_num:u64,
    sequence_num:u64,
    check_point_num:u64,
    node_id:u64,
    msg_digest:String,
    prepare_msg_list: Vec<Bft_Prepare_Message>
}

impl Bft_View_Change_Message {
    pub fn new(_view_num:u64, _seq_num:u64, _check_point_num:u64, _node_id:u64) -> Bft_View_Change_Message {
        let msg = Bft_View_Change_Message{
            view_num:_view_num,
            sequence_num:_seq_num,
            check_point_num:_check_point_num,
            node_id:_node_id,
            msg_digest:String::new(),
            prepare_msg_list: Vec::new()
        };

        return msg;

    }

    pub fn addPrePareMsg(&mut self, prePare:Bft_Prepare_Message) {
        self.prepare_msg_list.push(prePare);
    }

    pub fn get_view_num(&self) -> u64 {
        return self.view_num.clone();
    }

    pub fn get_node_id(&self) -> u64 {
        return self.node_id.clone();
    }

    pub fn get_check_point_num(&self) -> u64 {
        return self.check_point_num.clone();
    }

    pub fn get_msg_digest(&self) -> String {
        return (self.msg_digest.clone());
    }

    pub fn set_msg_digest(&mut self, sign:String) {
        self.msg_digest = sign;
    }
}

#[derive(RustcDecodable, RustcEncodable)]
#[derive(Clone)]
pub struct Bft_New_View_Message {
    view_num:u64,
    sequence_num:u64,
    check_point_num:u64,
    node_id:u64,
    msg_digest:String,
    prepare_msg_list: Vec<Bft_PrePrepare_Message>
}

impl Bft_New_View_Message {
    pub fn new(_view_num:u64, _seq_num:u64, _check_point_num:u64,  _node_id:u64) -> Bft_New_View_Message {
        let msg = Bft_New_View_Message{
            view_num:_view_num,
            sequence_num:_seq_num,
            check_point_num:_check_point_num,
            node_id:_node_id,
            msg_digest:String::new(),
            prepare_msg_list: Vec::new()
        };

        return msg;

    }

    pub fn addPrePareMsg(&mut self, prePare:Bft_PrePrepare_Message) {
        self.prepare_msg_list.push(prePare);
    }

    pub fn get_view_num(&self) -> u64 {
        return self.view_num.clone();
    }

    pub fn get_node_id(&self) -> u64 {
        return self.node_id.clone();
    }

    pub fn get_check_point_num(&self) -> u64 {
        return self.check_point_num.clone();
    }

    pub fn get_prePrepare(&self) ->&Vec<Bft_PrePrepare_Message> {
        return &self.prepare_msg_list;
    }

    pub fn add_prePrepare(&mut self, prePareMsg:Bft_PrePrepare_Message) {
        self.prepare_msg_list.push(prePareMsg);
    }

    pub fn get_msg_digest(&self) -> String {
        return (self.msg_digest.clone());
    }

    pub fn set_msg_digest(&mut self, sign:String) {
        self.msg_digest = sign;
    }
}


#[derive(RustcDecodable, RustcEncodable)]
#[derive(Clone)]
pub struct Bft_Regist_Msg{
    pub address:String,
    pub port:String,
    pub public_key: String,
}

impl Bft_Regist_Msg {
    pub fn new(_address:&str, _port:&str, _pubKey:&str) -> Bft_Regist_Msg{
        let msg = Bft_Regist_Msg{
            address:_address.to_string(),
            port:_port.to_string(),
            public_key: _pubKey.to_string(),
        };
        return msg;
    }
}

#[derive(RustcDecodable, RustcEncodable)]
#[derive(Clone)]
pub struct Bft_Regist_Reply{
    node_list: Vec<Btf_Node_Simple>,
    view_num:u64,
    check_point_num:u64,
    node_id:u64,
}

impl Bft_Regist_Reply {
    pub fn new(_node_list:Vec<Btf_Node_Simple>, _view_num:u64, _check_point_num:u64, _node_id:u64) -> Bft_Regist_Reply{
        let msg = Bft_Regist_Reply{
            node_list: _node_list,
            view_num:_view_num,
            check_point_num:_check_point_num,
            node_id:_node_id,
        };
        return msg;
    }

    pub fn get_node_ist(&self) -> Vec<Btf_Node_Simple> {
        return (self.node_list.clone());
    }

    pub fn get_view_num(&self) -> u64 {
        return self.view_num.clone();
    }

    pub fn get_node_id(&self) -> u64 {
        return self.node_id.clone();
    }
}