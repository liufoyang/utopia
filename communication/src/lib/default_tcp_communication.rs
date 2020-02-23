use std::io::prelude::*;
use std::thread;
use std::sync::mpsc::channel;
use std::net::TcpStream;
use std::net::TcpListener;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::boxed::Box;
use std::collections::HashMap;
use std::sync::{Mutex,Arc};
use std::rc::Rc;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;
use super::communication::{BftCommunication,BftCommunicationMsg};
use std::result::Result;
use std::option::Option::Some;
use flexi_logger::{Logger, opt_format};
use log::*;
use std::io;
use std::sync::MutexGuard;

pub struct Default_TCP_Communication {
    connections:Arc<Mutex<HashMap<String, Sender<BftCommunicationMsg>>>>,
    sys_nodify_map:Arc<Mutex<HashMap<String, Sender<Box<BftCommunicationMsg>>>>>,
    msg_sender:Sender<Box<BftCommunicationMsg>>
}

impl Default_TCP_Communication {
    pub fn startListen(addr:&str, port:&str) ->(Default_TCP_Communication, Receiver<Box<(BftCommunicationMsg)>>) {
        let mut address_all = String::from(addr);
        address_all.push_str(":");
        address_all.push_str(port);
        let listener = TcpListener::bind(address_all.as_str()).unwrap();

        let (msg_sender, msg_receiver) = channel();
        let connections:HashMap<String, Sender<BftCommunicationMsg>> = HashMap::new();

        let connections_mutex: Arc<Mutex<HashMap<String, Sender<BftCommunicationMsg>>>> = Arc::new(Mutex::new(connections));

        let connection_listener = Arc::clone(&connections_mutex);
        let msg_sender_sub = Sender::clone(&msg_sender);
        thread::Builder::new().name("bft_node_listener".to_string()).spawn(move|| {
            let mut thread_index = 0;
            for stream in listener.incoming() {

                info!("receive one connection");

                //
                // read the connection
                let mut stream = stream.unwrap();

                let mut msg_sender_reader = Sender::clone(&msg_sender_sub);
                Default_TCP_Communication::create_new_reader(msg_sender_reader,   stream);
            }
        });


        let nodify_map:HashMap<String, Sender<Box<BftCommunicationMsg>>> = HashMap::new();
        let nodify_map_mutex: Arc<Mutex<HashMap<String, Sender<Box<BftCommunicationMsg>>>>> = Arc::new(Mutex::new(nodify_map));


        let (main_sender, main_receiver) =  channel();
        let nodify_sender_map = Arc::clone(&nodify_map_mutex);
        let connection_receiver = Arc::clone(&connections_mutex);
        thread::Builder::new().name("bft_node_recevier".to_string()).spawn(move|| {
            while (true) {
                let msg_result = msg_receiver.recv();

                if msg_result.is_ok() {
                    let msg_box:Box<BftCommunicationMsg> = msg_result.unwrap();

                    let msg = *msg_box;
                    info!("receive one msg for receiver {}", msg.command);
                    if msg.command.eq_ignore_ascii_case("reply") {
                        let mut notify_map = nodify_sender_map.lock().unwrap();

                        if notify_map.contains_key(&msg.id) {
                            let notify_sender = notify_map.remove(&msg.id).unwrap();
                            let msg_box = Box::new(msg);
                            info!("give one msg for notify");
                            notify_sender.send(msg_box);
                        } else {

                        }

                    } else if msg.command.eq_ignore_ascii_case("disconnection") {
                        info!("remove connection for {}", msg.payload);
                        let mut connections = connection_receiver.lock().unwrap();
                        connections.remove(&msg.payload);
                    }else {
                        info!("give one msg for main");
                        let msg_box = Box::new(msg);
                        main_sender.send(msg_box);
                    }
                }
            }

        });

        let comminications = Default_TCP_Communication {
            connections:connections_mutex,
            sys_nodify_map:nodify_map_mutex,
            msg_sender:msg_sender,
        };

        return (comminications, main_receiver);
    }

    fn create_new_reader(msg_sender: Sender<Box<BftCommunicationMsg>>, mut stream: TcpStream) {

        let connection_name = stream.peer_addr().unwrap().to_string();
        info!("new connection put the pools {}",connection_name);
        let mut name = String::from("bft_node_reader_");
        name.push_str(connection_name.as_str());
        stream.set_read_timeout(None).expect("set_read_timeout call failed");

        thread::Builder::new().name(name).spawn(move || {
            let mut not_receiver_time = 0;
            while true {
                let mut buffer = [0; 2048];

                let read_result = stream.read(&mut buffer);

                if read_result.is_ok() {
                    let lensize = read_result.unwrap();

                    if lensize<= 0 {
                        not_receiver_time +=1;
                        info!("read stream zero");
                        let millis_100 = Duration::from_millis(200*not_receiver_time);
                        thread::sleep(millis_100);

                    } else {
                        not_receiver_time = 0;
                        let (left, right) = buffer.split_at(lensize);
                        let receiver_str = String::from_utf8_lossy(&left[..]).to_string();
                        //info!("receive source msg {}", receiver_str);
                        let msgList:Vec<&str> = receiver_str.split("#end#").collect();

                        info!("receive source msg after split {:?}", msgList);
                        for msg_str in msgList {
                            let message_str = msg_str.to_string();

                            if message_str.is_empty() {
                                continue;
                            }

                            if message_str.as_str() == "got" {
                                continue;
                            }
                            let communication_msg_result = Default_TCP_Communication::parsePbftMsg(connection_name.as_str(), message_str);

                            if communication_msg_result.is_none() {
                                continue;
                            }
                            let box_msg = Box::new(communication_msg_result.unwrap());
                            info!("send msg to processor");
                            msg_sender.send(box_msg);
                        }
                    }
                } else {
                    warn!("read stream error {:?}", read_result.err());
                    not_receiver_time +=1;
                    let millis_100 = Duration::from_millis(200*not_receiver_time);
                    thread::sleep(millis_100);
                    if not_receiver_time>3 {
                        break;
                    }
                }
            }
            // send disconnection msg;
            let communication_msg = BftCommunicationMsg {
                id: "disconnection_id".to_string(),
                is_sys: false,
                command: "disconnection".to_string(),
                version: "v1.0".to_string(),
                payload: connection_name.clone(),
                from: connection_name.clone()
            };

            let box_msg = Box::new(communication_msg);
            info!("send disconnection msg to processor and quit");
            msg_sender.send(box_msg);
            //return Ok("sender finish job".to_string());
        });
    }

    fn parsePbftMsg(connection_name: &str, message_str: String) -> Option<BftCommunicationMsg> {

         // check the msg format
        ///  command version leng \n
        ///  body
        let result = Option::None;
        let mut lines = message_str.lines();
// match header
        let mut command = String::new();
        let mut version = String::new();
        let mut payload = String::new();
        let mut is_sys = false;
        let mut id = String::new();
        match lines.next() {
            Some(header_str) => {
                // check header
                let mut iter = header_str.split_whitespace();
                let mut i = 0;
                for token in iter {
                    if i == 0 {
                        command.push_str(token);
                    }
                    if i == 1 {
                        version.push_str(token);
                    }

                    if i == 2 {
                        is_sys = token.eq_ignore_ascii_case("true");
                    }

                    if i == 3 {
                        id.push_str(token);
                    }
                    i += 1;
                }
                if (command.is_empty() || version.is_empty()) {
                    // header format error
                    warn!("header format error");
                    return result;
                }
            }
            None => {
                // message format error
                warn!("message format error");
                return result;
            }
        }
        match lines.next() {
            Some(payload_str) => {
                // check body
                payload.push_str(payload_str);
            }
            None => {
                // message format error
                warn!("message format error");
                return result;
            }
        }
        let communication_msg = BftCommunicationMsg {
            id: id,
            is_sys: is_sys,
            command: command,
            version: version,
            payload: payload,
            from: connection_name.to_string()
        };
        return Some(communication_msg)
    }


    fn create_new_writer(msg_sender: Sender<Box<BftCommunicationMsg>>, connections: &mut HashMap<String, Sender<BftCommunicationMsg>>, mut stream: TcpStream) {

        let (readerSender, readerReceiver) = channel();
        let connection_name = stream.peer_addr().unwrap().to_string();
        connections.insert(connection_name.clone(), readerSender);
        info!("new connection put the pools {}",connection_name);
        let mut name = String::from("bft_node_writer_");
        name.push_str(connection_name.as_str());

        thread::Builder::new().name(name).spawn(move || {
            let millis_100 = Duration::from_millis(200);
            let mut not_write_time = 0;
            while true {

                let send_msg_result: Result<BftCommunicationMsg, RecvTimeoutError> = readerReceiver.recv_timeout(millis_100);
                if send_msg_result.is_ok() {
                    let data: BftCommunicationMsg = send_msg_result.unwrap();
                    let data_str = data.to_string();
                    info!("begin send the msg {}", data_str);
                    //let msg_data = .as_bytes();
                    let mut write_result = stream.write(data_str.as_bytes());
                    write_result = stream.write(b"#end#");
                    let flush_result = stream.flush();

                    if write_result.is_ok()&& write_result.unwrap()>0 && flush_result.is_ok() {
                        info!("send finish {}", data_str);
                    } else {

                        warn!("send msg fail, reconnection to {}", connection_name);
                        // reconnection
                        let stream_result = TcpStream::connect(connection_name.as_str());
                        if !stream_result.is_ok() {
                            // not connection
                            error!("connection {} fail {:?}", connection_name, stream_result.err());

                            break;
                        } else {
                            info!("reconnection {}", connection_name);
                            stream = stream_result.unwrap();

                            //let msg_data = .as_bytes();
                            let mut write_result = stream.write(data_str.as_bytes());
                            write_result = stream.write(b"#end#");
                            let flush_result = stream.flush();

                            if write_result.is_ok() && flush_result.is_ok() {
                                info!("send finish {}", data_str);
                            }
                        }
                    }

                } else {
                    info!("no msg to send to stream {}, {:?}", connection_name, readerReceiver);
                    not_write_time +=1;

                    if not_write_time%20 == 0 {
                        let  write_result = stream.write(b"got#end#");
                        let flush_result = stream.flush();
                        if write_result.is_ok()&& write_result.unwrap()>0 && flush_result.is_ok(){
                            info!("send heart hit to {}", connection_name);
                            not_write_time = 0;
                        } else {
                            break;
                        }
                    }
                }

            }
            // send disconnection msg;
            let communication_msg = BftCommunicationMsg {
                id: "disconnection_id".to_string(),
                is_sys: false,
                command: "disconnection".to_string(),
                version: "v1.0".to_string(),
                payload: connection_name.clone(),
                from: connection_name.clone()
            };

            let box_msg = Box::new(communication_msg);
            info!("send disconnection msg to processor and quit");
            msg_sender.send(box_msg);
            //return Ok("sender finish job".to_string());
        });
    }

    pub fn sendMessage(&mut self, address:&str, port:&str, data:BftCommunicationMsg, isAsync:bool) -> Option<BftCommunicationMsg>{
        let mut address_all = String::from(address);
        address_all.push_str(":");
        address_all.push_str(port);

        if isAsync {
            return self.sendMessageAsync(address_all, data);
        } else {
            return self.sendMessageSync(address_all, data);
        }


    }

    fn sendMessageAsync(&self, connection_name:String, data:BftCommunicationMsg) -> Option<BftCommunicationMsg>{
        // find the sender
        let connection_sender = Arc::clone(&self.connections);

        let mut connections = connection_sender.lock().unwrap();

        if connections.contains_key(&connection_name) {
            let sender = connections.get(&connection_name).unwrap();
            info!("give msg to reader thread {} {:?}", connection_name, sender);
            sender.send(data);
        } else {
            let connection_result = self.doConnectionTo(connection_name.as_str(), &mut connections);

            if connection_result.is_some() {
                let sender = connections.get(&connection_name).unwrap();
                sender.send(data);
            }
        }

        return None;
    }

    fn doConnectionTo(&self, connection_name: &str, connections: &mut HashMap<String, Sender<BftCommunicationMsg>>) -> Option<String> {
        warn!("reconnection to {}", connection_name);
        let stream_result = TcpStream::connect(connection_name);
        if !stream_result.is_ok() {
            // not connection
            error!("connection {} fail {:?}", connection_name, stream_result.err());
            return None;
        }
        let stream = stream_result.unwrap();
        info!("connection to {} success", connection_name);
        let msg_sender_sub = Sender::clone(&self.msg_sender);
        Default_TCP_Communication::create_new_writer(msg_sender_sub, connections, stream);
        return Some("Success".to_string());
    }

    fn sendMessageSync(&mut self, connection_name:String, mut data:BftCommunicationMsg) -> Option<BftCommunicationMsg>{

        data.id = String::from("one_time");
        // find the sender
        let mut connection_sender = Arc::clone(&self.connections);
        let reply_recevier = self.applySysNotify(data.id.as_str());

        let mut has_send = false;
        if !has_send {
            // todo not need to lock at send
            let mut connections = connection_sender.lock().unwrap();

            if connections.contains_key(&connection_name) {
                let sender = connections.get(&connection_name).unwrap();

                sender.send(data);
                has_send = true

            } else {
                let connection_result = self.doConnectionTo(connection_name.as_str(), &mut connections);

                if connection_result.is_some() {
                    let sender = connections.get(&connection_name).unwrap();
                    sender.send(data);
                    has_send = true;
                }
            }
        }

        if has_send {
            let reply_result = reply_recevier.recv();
            if reply_result.is_ok() {
                let replay_msg_box:Box<BftCommunicationMsg> = reply_result.unwrap();
                return Some(*replay_msg_box);
            } else {
                return None;
            }
        } else {
            return None;
        }

    }

    fn applySysNotify(&mut self, msg_id:&str) -> Receiver<Box<BftCommunicationMsg>> {
        let (notify_sender, notify_receiver) = channel();

        let nodify_sender_map = Arc::clone(&self.sys_nodify_map);
        let mut notify_map = nodify_sender_map.lock().unwrap();

        notify_map.insert(msg_id.to_string(), notify_sender);

        return notify_receiver;
    }

}