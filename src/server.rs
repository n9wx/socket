use std::sync::Mutex;

use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::broadcast::{Receiver, Sender};

use crate::request::{ClientRequest, ConnectType, Message};
use crate::to_string;
use crate::utils::from_slice;

// 主线程 监听端口,接受客户端的连接
// 接收线程 ,轮询,接收已有连接发送的信息
// 发送线程 ,通过receiver 接收任务,发送给对应的客户端,未连接的客户端放入等待队列
// 等待线程 发送等待队列中存的信息 ?会有必要吗,也许应该和发送线程合并
pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn new(socket_addrs: impl ToSocketAddrs) -> io::Result<Self> {
        let listener = TcpListener::bind(socket_addrs).await?;
        
        Ok(Self {
            listener,
        })
    }
    
    pub async fn start(&self) -> io::Result<()> {
        let (sender, _) = tokio::sync::broadcast::channel::<Message>(16);
        loop {
            let (mut client, _) = self.listener.accept().await?;
            
            let mut connect_buf = vec![0u8; 1024];
            let len = client.read(&mut connect_buf).await?;
            connect_buf.resize(len, 0);
            
            //通过第一次连接发送的数据确定stream的连接方式
            let first_connect: ClientRequest = from_slice(&connect_buf);
            let (thread_ident, connect_type);
            if let ClientRequest::Connect(connect) = first_connect {
                thread_ident = connect.username;
                connect_type = connect.connect_type;
                println!("client {} connect: {}", thread_ident, connect_type);
            } else {
                // todo 更好的错误处理
                panic!("client don't send connect first")
            }
            
            //read write 是对于client 而言
            match connect_type {
                //handle client write to server
                ConnectType::Write => {
                    let thread_sender = sender.clone();
                    handle_client_write(thread_sender, client);
                }
                //handle client read from server
                ConnectType::Read => {
                    let thread_receiver = sender.subscribe();
                    handle_client_read(thread_receiver, thread_ident, client);
                }
            };
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {}
}

fn handle_client_read(mut receiver: Receiver<Message>, ident: String, mut client: TcpStream) {
    tokio::spawn(async move {
        loop {
            let msg = receiver.recv().await.unwrap();
            if msg.to != ident {
                continue;
            } else {
                let _ = client.write(
                    to_string(&msg).as_bytes()
                ).await.unwrap();
            }
        };
    });
}

fn handle_client_write(sender: Sender<Message>, mut client: TcpStream) {
    tokio::spawn(async move {
        loop {
            let mut buf = vec![0u8; 1024];
            match client.read(&mut buf).await {
                Ok(0) => {
                    break;
                }
                Ok(len) => {
                    buf.resize(len, 0);
                    let request: ClientRequest = from_slice(&buf);
                    let msg = (*request).clone();
                    println!("{}", msg);
                    sender.send(msg).unwrap();
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        }
    });
}