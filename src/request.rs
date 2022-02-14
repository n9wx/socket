use std::fmt::{Display, Formatter};
use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientRequest {
    Connect(Connect),
    Message(Message),
    Close(String),
}

impl Deref for ClientRequest {
    type Target = Message;
    
    fn deref(&self) -> &Self::Target {
        if let ClientRequest::Message(msg) = self {
            msg
        } else {
            panic!("unexpected element")
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub msg: String,
}

impl Message {
    pub fn new<T>(from: T, to: T, msg: T) -> Self
        where T: ToString
    {
        Self {
            from: from.to_string(),
            to: to.to_string(),
            msg: msg.to_string(),
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "===============start===============")?;
        writeln!(f, "from : {}", self.from)?;
        writeln!(f, "to : {}", self.to)?;
        writeln!(f, "message content : {}", self.msg)?;
        write!(f, "================end================")
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Connect {
    pub connect_type: ConnectType,
    pub username: String,
}

impl Connect {
    pub fn new(connect_type: ConnectType, username: impl ToString) -> Self {
        Self {
            connect_type,
            username: username.to_string(),
        }
    }
    
    pub fn is_read(&self) -> bool {
        self.connect_type == ConnectType::Read
    }
    
    pub fn is_write(&self) -> bool {
        !self.is_read()
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum ConnectType {
    Read,
    Write,
}

impl Display for ConnectType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let connect_type = match self {
            ConnectType::Read => "Read",
            ConnectType::Write => "Write"
        };
        write!(f, "{connect_type}")
    }
}