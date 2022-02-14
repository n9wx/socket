use std::io::stdin;
use std::sync::{Arc, Mutex};

use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::{ClientRequest, Connect, ConnectType, Message, SOCKET_ADDRS, to_string};
use crate::utils::from_slice;

pub struct Client {
    username: String,
    client: TcpStream,
    msg_list: Arc<Mutex<Vec<Message>>>,
}

impl Client {
    pub async fn new(username: impl ToString) -> io::Result<Self> {
        let client = TcpStream::connect(SOCKET_ADDRS).await?;
        Ok(Self {
            username: username.to_string(),
            client,
            msg_list: Arc::new(Mutex::new(vec![])),
        })
    }
    
    pub async fn start(&mut self) {
        // set read task
        let msg_list = self.msg_list.clone();
        let username = self.username.clone();
        tokio::spawn(async move {
            let read_connect = ClientRequest::Connect(
                Connect::new(ConnectType::Read, username)
            );
            let mut read_client = TcpStream::connect(SOCKET_ADDRS).await.unwrap();
            let _ = read_client.write(
                to_string(read_connect).as_bytes()
            ).await.unwrap();
            
            loop {
                let mut buf = vec![0u8; 1024];
                let len = read_client.read(&mut buf).await.unwrap();
                buf.resize(len, 0);
                let msg: Message = from_slice(&buf);
                msg_list.lock().unwrap()
                        .push(msg);
            }
        });
        
        // set main thread(task)
        let write_connect = ClientRequest::Connect(
            Connect::new(ConnectType::Write, self.username.clone())
        );
        let _ = self.client.write(
            to_string(write_connect).as_bytes()
        ).await.unwrap();
        
        let stdin = stdin();
        let read_str = || {
            let mut str = String::new();
            let _ = stdin.read_line(&mut str).unwrap();
            str.remove(str.len() - 1);
            str
        };
        loop {
            match read_str().as_str() {
                "r" => {
                    let mut vec_guard = self.msg_list.lock().unwrap();
                    if !vec_guard.is_empty() {
                        let len = vec_guard.len();
                        for _ in 0..len {
                            println!("{}", vec_guard.pop().unwrap());
                        }
                    } else {
                        println!("receive nothing");
                    }
                }
                "w" => {
                    println!("receiver username :");
                    let to = read_str();
                    println!("message : ");
                    let msg = read_str();
                    
                    let message = Message::new(
                        self.username.clone(),
                        to,
                        msg,
                    );
                    let _ = self.client.write(
                        to_string(ClientRequest::Message(message)).as_bytes()
                    ).await.unwrap();
                }
                _ => {
                    println!("undefined command");
                }
            }
        }
    }
}