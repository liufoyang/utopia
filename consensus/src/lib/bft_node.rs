use super::bft_message::*;
use super::command_executor::Command_Executor;
//
use std::io::prelude::*;
use serde_json::{Result, Value};
use std::collections::HashMap;
use std::u64;
use futures::{stream, Future};
use std::thread;
use std::sync::mpsc;
use std::time::SystemTime;
use std::time::Duration;
use super::communication;
extern crate rustc_serialize;
use rustc_serialize::json::{self, ToJson, Json};
use super::communication::{BftCommunication,BftCommunicationMsg};
use super::default_tcp_communication::Default_TCP_Communication;
use rustc_serialize::json::DecodeResult;
use std::collections::BTreeSet;
extern crate crypto;
use crypto::md5;
use super::bft_signtor::Bft_Signtor;
extern crate rustc_hex;
use rustc_hex::{FromHex,ToHex};
use flexi_logger::{Logger, opt_format};
use log::*;


#[derive(RustcDecodable, RustcEncodable)]
#[derive(Clone)]
pub struct Btf_Node_Simple {
    node_id:u64,
    pub address:String,
    pub port:String,
    pub public_key: String,
}

impl Btf_Node_Simple {
    pub fn get_node_id (&self) -> u64 {
        return self.node_id.clone();
    }

    pub fn new(_node_id:u64, _address:&str, _port:&str, _pub_key:&str) ->Btf_Node_Simple {
        let node = Btf_Node_Simple {
            node_id:_node_id,
            address:_address.to_string(),
            port:_port.to_string(),
            public_key: _pub_key.to_string(),
        };

        return node;

    }
}

pub struct Btf_Node{
    base:Btf_Node_Simple,
    status:String,
    view_num:u64,
    seq_num:u64,
    is_primary: bool,
    node_list: Vec<Btf_Node_Simple>,
    msg_cache:HashMap<u64, Bft_Message>,
    msg_sign_cache:BTreeSet<String>,
    prepare_cache: HashMap<u64, Vec<Bft_Prepare_Message>>,
    commit_cache: HashMap<u64, Vec<Bft_Commit_Message>>,
    viewchange_cache: HashMap<u64, Vec<Bft_View_Change_Message>>,
    signtor:Bft_Signtor,
    check_point_num: u64,
    communication:Default_TCP_Communication,
}

impl Btf_Node {

    fn new(_view_num:u64, mut _node_list:Vec<Btf_Node_Simple>, _ip:&str, _port:&str,_node_id:u64, isPrimary:bool, _communication:Default_TCP_Communication) -> Btf_Node{

        let signer = Bft_Signtor::new();
        return Btf_Node::new_with_signtor(_view_num,_node_list, _ip, _port, _node_id, isPrimary, signer, _communication);
    }

    pub fn set_status(&mut self, new_status:&str) {
        self.status = new_status.to_string();
    }

    pub fn new_with_signtor(_view_num:u64, mut _node_list:Vec<Btf_Node_Simple>, _ip:&str, _port:&str,_node_id:u64, isPrimary:bool, _signtor:Bft_Signtor, _communication:Default_TCP_Communication) -> Btf_Node{
        let bft_simple = Btf_Node_Simple{
            node_id:_node_id,
            address:_ip.to_string(),
            port: _port.to_string(),
            public_key:_signtor.get_public_key(),
        };

        // put self to the node list
        _node_list.push(bft_simple.clone());

        let node = Btf_Node{
            base:bft_simple,
            status:"new".to_string(),
            view_num:_view_num,
            seq_num:0,
            is_primary:isPrimary,
            node_list:_node_list,
            msg_cache:HashMap::new(),
            msg_sign_cache:BTreeSet::new(),
            prepare_cache:HashMap::new(),
            commit_cache:HashMap::new(),
            viewchange_cache:HashMap::new(),
            signtor:_signtor,
            check_point_num: 0,
            communication:_communication

        };

        return node;
    }

    pub fn get_node_base(&self) -> &Btf_Node_Simple {
        return &(self.base);
    }

    fn get_node_pub_key(&self, node_id:&u64) -> Option<String> {
        let mut pub_key_result = Option::None;
        for simple in &self.node_list {
            if simple.node_id == *node_id {
                pub_key_result = Some(simple.public_key.clone());
                break;
            }
        }

        return pub_key_result;
    }

    pub fn receiveClientMsg(& mut self, msg:Bft_Message, executor:&mut Command_Executor) -> String {

        let view_num_temp:usize = 10;
        info!("begin process for");

        let msg_md5 = msg.signMd5();

        let msg_md5_str =msg_md5.to_string();
        if self.msg_sign_cache.contains(&msg_md5.to_string()) {
            return msg_md5_str;
        }

        self.msg_sign_cache.insert(msg_md5.to_string());
        if self.is_primary {
            // is primary node send prepare
            let keys = self.msg_cache.keys();
            let mut num:u64 = self.seq_num + 1;

            // clone one msg save in self node;
            self.msg_cache.insert(num.clone(), msg.clone());
            let mut prePrepareMsg:Bft_PrePrepare_Message = Bft_PrePrepare_Message::new(self.view_num.clone(), self.get_node_base().get_node_id(), num.clone(), msg);
            let mut payload = json::encode(&prePrepareMsg).unwrap();

            let sign = self.signtor.sign_string(payload.as_str());
            prePrepareMsg.set_msg_digest(sign);
            payload = json::encode(&prePrepareMsg).unwrap();

            //self.executor.execute(payload.as_str());
            self.broadcastMsg(payload, "prePrepare");

            info!("process for primary");
            self.doPrepare(prePrepareMsg, executor);

            self.seq_num = num;
            return msg_md5_str;

        } else {
            // broadcast the msg to other node
            let payload = json::encode(&msg).unwrap();

            //self.executor.execute(payload.as_str());
            self.broadcastMsg(payload, "forword");
            return msg_md5_str;
        }
    }

    pub fn receiveForwordMsg(& mut self, msg:Bft_Message, executor:&mut Command_Executor) -> String {

        if self.is_primary {
            return self.receiveClientMsg(msg, executor);
        } else {
            let msg_md5 = msg.signMd5();

            let md5_str = msg_md5.to_string();
            if !self.msg_sign_cache.contains(&md5_str) {
                self.msg_sign_cache.insert(md5_str.clone());
            }
            return md5_str;
        }
    }
    pub fn doPrepare(& mut self,  msg:Bft_PrePrepare_Message, mut executor:&mut Command_Executor) -> Option<(u64, u64)>{

        info!("begin doPrepare for");

        // check sign for node
        let mut sign_msg = msg.clone();
        let msg_digest = msg.get_msg_digest();
        sign_msg.set_msg_digest(String::new());
        let msg_node_id = sign_msg.get_node_id();
        let pub_key_result = self.get_node_pub_key(&msg_node_id);

        if pub_key_result.is_none() {
            error!("can not found  node id for {}", sign_msg.get_node_id());
            return Option::None;
        }

        let signMsgStr = json::encode(&sign_msg).unwrap();
        if !Bft_Signtor::check_sign(signMsgStr.as_str(), pub_key_result.unwrap().as_str(), msg_digest.as_str()) {
            warn!("msg sign not pass for {}", sign_msg.get_node_id());
            return Option::None;
        }

        // check is the primary message, check the digest by primary pub key;
        if msg.get_view_num() != self.view_num {
            warn!("doPrepare view num not same {}, {}", msg.get_view_num(), self.view_num);
            return Some((msg.get_view_num(), msg.get_sequence_num()));
        }

        let sequence_num = msg.get_sequence_num();
        if !self.is_primary && sequence_num > self.seq_num {
            self.seq_num = msg.get_sequence_num();
        }

        // check if have before,if not put msg to msg_cache
        //let mut source_msg_option:Option<&Bft_Message> = Option::None;
        if self.msg_cache.contains_key(& msg.get_sequence_num()) {
            info!("doPrepare have recevie the sequence num ");
            // have receive this msg num before, check if the same msg
            let receive_msg = self.msg_cache.get(& sequence_num).unwrap();
            if receive_msg.get_id() == msg.get_client_msg().get_id(){
                // the same
                //source_msg_option = Some(receive_msg);

                // find the pre cache
                let mut has_send= false;
                if self.prepare_cache.contains_key(& sequence_num) {
                    for prepare_msg in self.prepare_cache.get(& sequence_num).unwrap() {
                        if prepare_msg.get_node_id() == self.get_node_base().node_id {
                            // broadcast again
                            let payload = json::encode(&prepare_msg).unwrap();
                            self.broadcastMsg(payload, "prepare");
                            has_send = true;
                            break;
                        }
                    }
                }

                if !has_send {
                    // check pass add to prepare cache;
                    let mut prepare_msg = Bft_Prepare_Message::new(self.view_num, msg.get_sequence_num(), self.get_node_base().node_id);
                    let seq_num = msg.get_sequence_num();

                    // send the prepare msg
                    let mut payload = json::encode(&prepare_msg).unwrap();
                    let sign = self.signtor.sign_string(payload.as_str());
                    prepare_msg.set_msg_digest(sign);
                    payload = json::encode(&prepare_msg).unwrap();

//                    if self.prepare_cache.contains_key(& seq_num) {
//                        let list = self.prepare_cache.get_mut(& seq_num).unwrap();
//                        list.push(prepare_msg.clone());
//                    } else {
//                        let mut prepare_vec = Vec::new();
//                        prepare_vec.push(prepare_msg.clone());
//                        self.prepare_cache.insert(seq_num, prepare_vec);
//                    }

                    self.broadcastMsg(payload, "prepare");
                    self.receivePrepare(prepare_msg, &mut executor);

                }

                let mut receive_msg2 = self.msg_cache.get_mut(& msg.get_sequence_num()).unwrap();
                receive_msg2.set_status(2);


            } else {
                // not same msg for the same num
                error!("not same msg for the same num");
            }
            return Some((msg.get_view_num(), msg.get_sequence_num()));

        } else {
            info!("doPrepare new  sequence num msg");
            let mut client_msg = msg.get_client_msg().clone();
            client_msg.set_status(2);
            self.msg_cache.insert(msg.get_sequence_num(),  client_msg);
            //source_msg_option = Some(& msg.get_client_msg());


            // check pass add to prepare cache;
            let mut prepare_msg = Bft_Prepare_Message::new(self.view_num, msg.get_sequence_num(),  self.get_node_base().node_id);
            let mut payload = json::encode(&prepare_msg).unwrap();
            let sign = self.signtor.sign_string(payload.as_str());
            prepare_msg.set_msg_digest(sign);
            payload = json::encode(&prepare_msg).unwrap();

//            let seq_num = msg.get_sequence_num();
//            if self.prepare_cache.contains_key(& seq_num) {
//                let list = self.prepare_cache.get_mut(& seq_num).unwrap();
//                list.push(prepare_msg.clone());
//            } else {
//                let mut prepare_vec = Vec::new();
//                prepare_vec.push(prepare_msg.clone());
//                self.prepare_cache.insert(seq_num, prepare_vec);
//            }

            // send the prepare msg
            self.broadcastMsg(payload, "prepare");
            self.receivePrepare(prepare_msg, &mut executor);

            return Some((msg.get_view_num(), msg.get_sequence_num()));
        }
    }

    /// receivePrepare msg from other node,
    /// check msg is valid, if yes ,put to precache list
    ///
    pub fn receivePrepare(&mut self, msg:Bft_Prepare_Message, mut executor:&mut Command_Executor) {

        info!("bein receivePrepare");

        // check sign for node
        let mut sign_msg = msg.clone();
        let msg_digest = msg.get_msg_digest();
        sign_msg.set_msg_digest(String::new());
        let msg_node_id = sign_msg.get_node_id();
        let pub_key_result = self.get_node_pub_key(&msg_node_id);

        if pub_key_result.is_none() {
            error!("can not found  node id for {}", sign_msg.get_node_id());
            return;
        }

        let signMsgStr = json::encode(&sign_msg).unwrap();
        if !Bft_Signtor::check_sign(signMsgStr.as_str(), pub_key_result.unwrap().as_str(), msg_digest.as_str()) {
            info!("prepare msg sign not pass for {}", sign_msg.get_node_id());
            return;
        }

        if msg.get_view_num() != self.view_num {
            info!("receivePrepare view num not same {}, {}", msg.get_view_num(), self.view_num);
            return;
        }

        // check desigt
        // check the desigt
        let mut node_option:Option<Btf_Node_Simple>  = Option::None;
        for simple in & self.node_list {
            if simple.node_id == msg.get_node_id() {
                node_option = Some(simple.clone());
            }
        }

        // not know node, not process its prepare,
        if node_option.is_none() {
            info!("receivePrepare no illge node");
            return;
        }

        let simple_node = node_option.unwrap();

        // check design fail
//        if simple_node.public_key.as_str() != msg.get_msg_digest() {
//            return;
//        }

        // check pass put the prepare msg to cache
        let seq_num = msg.get_sequence_num();
        if self.prepare_cache.contains_key(& seq_num) {
            let list = self.prepare_cache.get_mut(& seq_num).unwrap();
            list.push(msg.clone());
        } else {
            let mut prepare_vec = Vec::new();
            prepare_vec.push(msg.clone());
            self.prepare_cache.insert(seq_num, prepare_vec);
        }

        // check if need to do commit;
        let has_commit =  self.checkIfCommit(&msg.get_sequence_num());


        if has_commit && self.msg_cache.contains_key(&msg.get_sequence_num()) {

            // new commit msg and broadcast the msg _view_num:u32, _sequence_num:u32, _digest:HashValue, _node_id:u32
            let mut commit_msg:Bft_Commit_Message = Bft_Commit_Message::new(self.view_num, seq_num.clone(), self.get_node_base().node_id);
            let mut payload = json::encode(&commit_msg).unwrap();

            let sign = self.signtor.sign_string(payload.as_str());
            commit_msg.set_msg_digest(sign);

            info!("begin to commit {}", msg.get_sequence_num());

//            // put msg to log file
//            if self.commit_cache.contains_key(&seq_num) {
//                let list = self.commit_cache.get_mut(&seq_num).unwrap();
//                list.push(commit_msg.clone());
//            } else {
//                let mut commit_msg_list = Vec::new();
//                commit_msg_list.push(commit_msg.clone());
//                self.commit_cache.insert(seq_num.clone(), commit_msg_list);
//            }

            let mut receive_msg = self.msg_cache.get_mut(& msg.get_sequence_num()).unwrap();
            if receive_msg.get_status() != 3 {

                // do commit
                let mut logs_str = String::from("commit ");
                logs_str.push_str(seq_num.to_string().as_str());
                logs_str.push_str(" ");
                logs_str.push_str(receive_msg.get_payload());
                executor.savelog(logs_str.as_str());

                executor.command_execute(receive_msg.get_payload());
                // do command
                let mut payload = json::encode(&commit_msg).unwrap();
                receive_msg.set_status(3);

                let sign = receive_msg.signMd5();
                self.msg_sign_cache.remove(&sign.to_string());

                payload = json::encode(&commit_msg).unwrap();
                info!("commit msg {}", payload);
                // broadcast the msg to other
                self.broadcastMsg(payload, "commit");

                self.receiveCommit(commit_msg, &mut executor);

            }
        }

    }

    fn checkIfCommit(& mut self, _sequence_num:& u64) ->bool {
        if !self.prepare_cache.contains_key(_sequence_num) {
            return false;
        }
        let min_pass_count = self.node_list.len()*2/3 + 1;
        let prepare_msg_list_option = self.prepare_cache.get(_sequence_num);
        if prepare_msg_list_option.is_none() {
            return false;
        }

        let prepare_list = prepare_msg_list_option.unwrap();

        // have enough prepare msg
        info!("enough prepare {} {}", prepare_list.len(), min_pass_count);
        return prepare_list.len()>= min_pass_count;
    }

    pub fn receiveCommit(& mut self, msg:Bft_Commit_Message, mut executor:&mut Command_Executor) {

        // check sign for node
        let mut sign_msg = msg.clone();
        let msg_digest = msg.get_msg_digest();
        sign_msg.set_msg_digest(String::new());
        let msg_node_id = sign_msg.get_node_id();
        let pub_key_result = self.get_node_pub_key(&msg_node_id);

        if pub_key_result.is_none() {
            error!("can not found  node id for {}", sign_msg.get_node_id());
            return;
        }

        let signMsgStr = json::encode(&sign_msg).unwrap();
        if !Bft_Signtor::check_sign(signMsgStr.as_str(), pub_key_result.unwrap().as_str(), msg_digest.as_str()) {
            error!("commit msg sign not pass for {}", sign_msg.get_node_id());
            return;
        }

        info!("check pass, process commit msg");
//        let mut source_msg_option:Option<Bft_Message> = None;
//        if self.msg_cache.contains_key(& msg.get_sequence_num()) {
//            // have receive this msg num before, check if the same msg
//            let receive_msg = self.msg_cache.get(& msg.get_sequence_num()).unwrap();
//            source_msg_option = Some(receive_msg.clone());
//        }
//
//        // have receive pre prepare msg in this node but not same msg, return error;
//        if  source_msg_option.is_some() {
//            let source_msg = source_msg_option.unwrap();
//        }

        // check pass add to prepare cache;
        if self.commit_cache.contains_key(&msg.get_sequence_num()) {
            let list = self.commit_cache.get_mut(&msg.get_sequence_num()).unwrap();
            list.push(msg.clone());
        } else {
            let mut commit_msg_list = Vec::new();
            commit_msg_list.push(msg.clone());
            self.commit_cache.insert(msg.get_sequence_num(), commit_msg_list);
        }

        self.doReplay(msg.get_sequence_num(), executor);

    }

    pub fn doReplay(&mut self, _sequence_num:u64, mut executor:&mut Command_Executor) {
        if !self.commit_cache.contains_key(&_sequence_num) {
            return;
        }

        // never receive primary node pre prepare, not replay
        if !self.msg_cache.contains_key(&_sequence_num) {
            return;
        }

        let mut msg = self.msg_cache.get_mut(&_sequence_num).unwrap();
        /// commit mes count > 2f+1 then pass and view not change commit local;
        ///  commit mes count > f+1 then pass and view have changed commit at this node view;
        let min_pass_count = self.node_list.len()/3 + 1;
        let commit_msg_list = self.commit_cache.get(&_sequence_num).unwrap();
        info!("enough commit {} {}", commit_msg_list.len(), min_pass_count);
        if commit_msg_list.len()>= min_pass_count && msg.get_status()==3 {
            msg.set_status(4);
            let result = executor.command_execute(msg.get_payload());

            let result_payload:String = match result {
                Some(msg) => msg,
                None => "poccess fail".to_string()
            };

            // save to blockchain
            executor.save_to_block_chain(msg.get_payload(), self.base.get_node_id().to_string().as_str(), _sequence_num.clone());

            // new replay msg and send to client _view_num:u32, _payload: &str, _node_id:&str, _source_msg:Bft_Message
            let replay_msg:Bft_Replay = Bft_Replay::new(self.view_num, result_payload.as_str(), self.base.node_id.clone(), msg.clone());
            let payload = json::encode(&replay_msg).unwrap();
            // send to client only
            let communication_msg = BftCommunicationMsg::new("replay", payload.as_str());

            info!("replay to {} {} {}", msg.get_client_ip(), msg.get_port(), payload);
            self.communication.sendMessage(msg.get_client_ip(), msg.get_port(), communication_msg, true);

        }
    }

    /// start new node, connect the bft network
    pub fn start_node(_primary_address:&str, _primary_port: &str, _ip:&str, _port:&str, mut _communication:Default_TCP_Communication) -> Btf_Node{

        // send request for primary
        let mut node_isntance:Btf_Node;
        let mut simple_vec:Vec<Btf_Node_Simple> = Vec::new();
        let mut _view_num:u64 = 0;
        let mut _node_id:u64 = 1;

        let signtor:Bft_Signtor = Bft_Signtor::new();
        if _primary_address.len() > 0 {
            // the bft network primary not null, is not the first node,send init msg to
            let regist_msg = Bft_Regist_Msg::new(_ip, _port,  signtor.get_public_key().as_str());
            let payload = json::encode(&regist_msg).unwrap();

            let send_result = Btf_Node::sendToPrimaryMsg(payload,"regist", _primary_address,_primary_port, &mut _communication);
            if send_result.is_ok() {
                let mut result_str = send_result.unwrap();
                //result_str = "{\"node_list\":[{\"node_id\":1,\"address\":\"10.3.209.223\",\"port\":\"8780\",\"public_key\":\"\"}],\"view_num\":1,\"check_point_num\":0,\"node_id\":2}\n".to_string();//result_str.trim().to_string();
                result_str = result_str.trim().to_string();


                let node_msg_result:DecodeResult<Bft_Regist_Reply> = json::decode(&result_str);
                if node_msg_result.is_ok() {
                    let reply_msg = node_msg_result.unwrap();
                    _view_num = reply_msg.get_view_num();
                    simple_vec = reply_msg.get_node_ist();
                    _node_id = reply_msg.get_node_id();

                } else {
                    info!("regist reply msg error {} {}", node_msg_result.err().unwrap(), result_str);
                }
            }

            node_isntance = Btf_Node::new_with_signtor(_view_num, simple_vec, _ip, _port,_node_id, false, signtor, _communication);


        } else {
            // 没有其他节点，这个就是第一个节点，第一个视图
            let port = _port;
            let view_num = 1;
            let node_list = Vec::new();
            let ip = _ip;
            let node_id = 1;
            node_isntance = Btf_Node::new(view_num, node_list, ip, port,node_id, true, _communication);

        }
        node_isntance.set_status("normal");

        info!("node init finish {}", node_isntance.is_primary);
        return node_isntance;
    }

    pub fn handler_expire(&mut self, msg_sign:&str) {

        if !self.msg_sign_cache.contains(&msg_sign.to_string()) {
            return;
        }

        let new_view_num = self.view_num.clone()+1;
        let mut view_change_msg = Bft_View_Change_Message::new(new_view_num, self.seq_num.clone(), 0,  self.get_node_base().get_node_id());
        let mut payload = json::encode(&view_change_msg).unwrap();

        let sign = self.signtor.sign_string(payload.as_str());
        view_change_msg.set_msg_digest(sign);

        payload = json::encode(&view_change_msg).unwrap();
        if self.viewchange_cache.contains_key(&new_view_num) {
            let mut viewchange_cache = self.viewchange_cache.get_mut(&new_view_num).unwrap();
            viewchange_cache.push(view_change_msg);
        } else {
            let mut viewchange_cache = Vec::new();
            viewchange_cache.push(view_change_msg);
            self.viewchange_cache.insert(new_view_num, viewchange_cache);
        }

        info!("begin view change ");
        self.set_status("view_change");

        self.broadcastMsg(payload, "viewchange");

    }

    pub fn receiveViewChange(& mut self, msg:Bft_View_Change_Message) {

        let new_view_num = msg.get_view_num();
        let check_point_num = msg.get_check_point_num();
        let seq_num:u64 = 1;
        info!("begin view change {}, {}, {}", new_view_num, self.view_num, self.is_primary);

        // check sign for node
        let mut sign_msg = msg.clone();
        let msg_digest = msg.get_msg_digest();
        sign_msg.set_msg_digest(String::new());
        let msg_node_id = sign_msg.get_node_id();
        let pub_key_result = self.get_node_pub_key(&msg_node_id);

        if pub_key_result.is_none() {
            info!("can not found  node id for {}", sign_msg.get_node_id());
            return;
        }

        let signMsgStr = json::encode(&sign_msg).unwrap();
        if !Bft_Signtor::check_sign(signMsgStr.as_str(), pub_key_result.unwrap().as_str(), msg_digest.as_str()) {
            info!("view change msg sign not pass for {}", sign_msg.get_node_id());
            return;
        }

        if self.viewchange_cache.contains_key(&new_view_num) {
            let mut viewchange_cache = self.viewchange_cache.get_mut(&new_view_num).unwrap();
            viewchange_cache.push(msg);
        } else {
            let mut viewchange_cache = Vec::new();
            viewchange_cache.push(msg);
            self.viewchange_cache.insert(new_view_num.clone(), viewchange_cache);
        }

        let viewchange_cache = self.viewchange_cache.get(&new_view_num).unwrap();

        let min_pass_count = self.node_list.len()*2/3 + 1;
        if viewchange_cache.len() >= min_pass_count {

            // change the view num

            let new_view_num_unsize = new_view_num.clone() as usize;
            let new_primary_id = new_view_num_unsize%self.node_list.len();

            if self.get_node_base().node_id == (new_primary_id as u64) {
                self.is_primary = true;
            } else {
                self.is_primary = false;
            }

            // is new primary broad case view_change comfirm msg, resend the prePrepare msg
            if self.is_primary {
                info!("new primary {}", self.get_node_base().node_id);
                self.view_num = new_view_num;
                // broadcase the new view msg

                let mut new_view_msg = Bft_New_View_Message::new(self.view_num.clone(), seq_num, 0,  self.get_node_base().get_node_id());

                // do all prepare for all sequence num biger than new view
                for (key, val) in self.msg_cache.iter() {
                    if key <= &check_point_num || val.get_status()==3 {
                        continue;
                    }

                    let new_seq_num = self.seq_num +1;

                    let prePrepareMsg:Bft_PrePrepare_Message = Bft_PrePrepare_Message::new(self.view_num.clone(), self.get_node_base().get_node_id(), new_seq_num.clone(), val.clone());
                    self.seq_num = new_seq_num;
                    new_view_msg.add_prePrepare(prePrepareMsg);
                }

                let mut payload = json::encode(&new_view_msg).unwrap();

                let sign = self.signtor.sign_string(payload.as_str());
                new_view_msg.set_msg_digest(sign);

                payload = json::encode(&new_view_msg).unwrap();

                self.set_status("normal");
                self.broadcastMsg(payload, "newview");

            }

        }


    }

    pub fn receiveNewView(&mut self, msg:Bft_New_View_Message, mut executor:&mut Command_Executor) {

        let new_view_num = msg.get_view_num();
        info!("begin new view {}, {}, {}", new_view_num, self.view_num, self.is_primary);

        // check sign for node
        let mut sign_msg = msg.clone();
        let msg_digest = msg.get_msg_digest();
        sign_msg.set_msg_digest(String::new());
        let msg_node_id = sign_msg.get_node_id();
        let pub_key_result = self.get_node_pub_key(&msg_node_id);

        if pub_key_result.is_none() {
            info!("can not found  node id for {}", sign_msg.get_node_id());
            return;
        }

        let signMsgStr = json::encode(&sign_msg).unwrap();
        if !Bft_Signtor::check_sign(signMsgStr.as_str(), pub_key_result.unwrap().as_str(), msg_digest.as_str()) {
            info!("new view msg sign not pass for {}", sign_msg.get_node_id());
            return;
        }

        if self.view_num == new_view_num {
            return;
        }

        if self.is_primary {
            self.is_primary = false;
            self.view_num = new_view_num;
        } else {
            self.view_num = new_view_num;
        }

        self.set_status("normal");
        info!("end view change to new view");

        for prePreMsg in msg.get_prePrepare() {
            self.doPrepare(prePreMsg.clone(), &mut executor);
        }
    }

    /// send message to all other node
    ///
    fn broadcastMsg (&mut self, data:String , command:&str) {

        info!("bengin to broadcase {}", self.node_list.len());
        let payload_str = data;
        for node in &(self.node_list) {

            // not send to self
            if node.node_id == self.get_node_base().node_id {
                continue;
            }
//            //build BftCommunicationMsg
            let communication_msg = BftCommunicationMsg::new(command, payload_str.as_str());

            self.communication.sendMessage(node.address.as_str(), node.port.as_str(), communication_msg, true);
            info!("send to node {}", node.address.as_str());

        }

        info!("end to broadcase");

    }

    fn sendToPrimaryMsg (data:String , command:&str, _primary_addr:&str, _port:&str, communication:&mut Default_TCP_Communication) ->std::result::Result<String, &'static str>{

        info!("bengin to sendToPrimaryMsg {} {}", _primary_addr, _port);
        let payload_str = data;
        let communication_msg = BftCommunicationMsg::new(command, payload_str.as_str());

        let send_result = communication.sendMessage(_primary_addr, _port, communication_msg, false);
        info!("send to primary node {}", command);

        if send_result.is_some() {
            let replay:BftCommunicationMsg = send_result.unwrap();
            return Ok(replay.payload);
        } else {
            return Err("read primary stram error");
        }

    }

    fn replayToNode(&mut self, command:&str, _addr:&str, _port:&str, data:String, msgId:&str){
        info!("bengin to broadcase");
        let payload_str = data;
        let mut communication_msg = BftCommunicationMsg::new(command, payload_str.as_str());
        communication_msg.id = msgId.to_string();
        let send_result = self.communication.sendMessage(_addr, _port, communication_msg, true);
        info!("replay to node {} {}", _addr, _port);

    }

    pub fn regist_node(&mut self, msg:Bft_Regist_Msg, msgId:&str) {

        let mut node_list:Vec<Btf_Node_Simple> = Vec::new();

        let mut node_id:u64 = 0;
        for node in & self.node_list {
            if node_id < node.node_id {
                node_id = node.node_id.clone();
            }
            node_list.push(node.clone());
        }
        node_id +=1;
        let reply = Bft_Regist_Reply::new(node_list, self.view_num.clone(), 0, node_id.clone());

        let mut payload = json::encode(&reply).unwrap();
        self.replayToNode("reply", msg.address.as_str(), msg.port.as_str(), payload, msgId);

        // push the new slave node to list
        let bft_slave = Btf_Node_Simple{
            node_id:node_id,
            address:msg.address,
            port: msg.port,
            public_key:msg.public_key
        };

        // broadcase to node list  new regist node
        payload = json::encode(&bft_slave).unwrap();

        self.node_list.push(bft_slave);

        self.broadcastMsg(payload, "newnode");
    }

    pub fn receive_new_node(&mut self, new_node:Btf_Node_Simple) {

        if self.is_primary {
            return;
        }
        info!("have a new node {}", new_node.node_id);
        self.node_list.push(new_node);
    }

    pub fn save_checkpoin(&mut self, mut executor:&mut Command_Executor) {

        let check_point_num = self.check_point_num + 1;

        // save the key value the check file
        let result = executor.save_check_point(&check_point_num);

        if result.is_none() {
            error!("make new check point fail");
            return;
        }

        // remove all finish message
        let mut key_delete:Vec<&u64> = Vec::new();
        for (key, value) in self.msg_cache.iter() {
            if value.get_status() == 3 {
                self.prepare_cache.remove(key);
                self.commit_cache.remove(key);
                key_delete.push(key);
            }
        }

        self.check_point_num = check_point_num;
        info!("make new check point success");
    }


}