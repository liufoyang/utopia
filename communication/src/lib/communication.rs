use std::sync::mpsc::Receiver;
///bft通信模块接口
///
///
pub trait  BftCommunication {
    fn startListen(&self, addr:&str, port:&str) ->Receiver<Box<BftCommunicationMsg>>;
}

pub struct BftCommunicationMsg {
    pub id:String,
    pub is_sys:bool,
    pub command:String,
    pub version:String,
    pub payload:String,
    pub from:String,
}

impl BftCommunicationMsg {
    pub fn new(_command:&str, _payload:&str)->BftCommunicationMsg {
        let msg = BftCommunicationMsg{
            id:String::from("default_id"),
            is_sys:false,
            command:_command.to_string(),
            version:"1.0".to_string(),
            payload:_payload.to_string(),
            from:String::new()
        };
        return msg;
    }
    ///  command version leng \n
       ///  body
    pub fn to_string(&self)->String {
        let mut msg_str = String::new();
        msg_str.push_str(self.command.as_str());
        msg_str.push_str(" ");
        msg_str.push_str(self.version.as_str());
        msg_str.push_str(" ");
        msg_str.push_str(self.is_sys.to_string().as_str());
        msg_str.push_str(" ");
        msg_str.push_str(self.id.to_string().as_str());
        msg_str.push_str(" ");
        msg_str.push_str(self.payload.len().to_string().as_str());

        msg_str.push_str("\n");
        msg_str.push_str(self.payload.as_str());
        msg_str.push_str("\n");
        return msg_str;
    }
}