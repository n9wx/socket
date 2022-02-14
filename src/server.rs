use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

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
    login_user: Arc<Mutex<HashSet<String>>>,
    sender: Sender<Message>,
    waiting_list: Arc<Mutex<HashMap<String, Vec<Message>>>>,
    send_list: Vec<Message>,
}

impl Server {
    fn handle_client_read(
        &self,
        ident: String,
        mut receiver: Receiver<Message>,
        mut client: TcpStream,
    ) {
        tokio::spawn(async move {
            loop {
                let msg = receiver.recv().await.unwrap();
                println!("receiver thread {},receive: ", ident);
                println!("{}", msg);
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
    
    fn handle_client_write(&self, mut client: TcpStream) {
        let sender = self.sender.clone();
        let waiting_list = self.waiting_list.clone();
        let login_user = self.login_user.clone();
        tokio::spawn(async move {
            let is_login = |username: &String| {
                login_user.lock().unwrap()
                          .contains(username)
            };
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
                        if !is_login(&msg.to) {
                            println!("user {} not login,push to waiting_list", msg.to);
                            let mut list_guard = waiting_list.lock().unwrap();
                            if let Some(msg_list) = list_guard.get_mut(&msg.to) {
                                msg_list.push(msg);
                            } else {
                                let username = msg.to.clone();
                                let msg_list = vec![msg];
                                list_guard.insert(username, msg_list);
                            }
                        } else {
                            sender.send(msg).unwrap();
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
        });
    }
    
    fn login_user(&mut self, username: String) {
        if self.login_user.lock().unwrap()
               .insert(username.clone()) {
            let option = self.waiting_list.lock().unwrap()
                             .remove(&username);
            if let Some(msg_list) = option {
                self.send_list = self.send_list.iter()
                                     .chain(msg_list.iter())
                                     .cloned()
                                     .collect();
            }
        }
    }
}

impl Server {
    pub async fn new(socket_addrs: impl ToSocketAddrs) -> io::Result<Self> {
        let listener = TcpListener::bind(socket_addrs).await?;
        let login_user = Arc::new(Mutex::new(HashSet::new()));
        let waiting_list = Arc::new(Mutex::new(HashMap::new()));
        let (sender, _) = tokio::sync::broadcast::channel::<Message>(16);
        
        Ok(Self {
            listener,
            login_user,
            sender,
            waiting_list,
            send_list: vec![],
        })
    }
    
    //todo 没有waiting list,发送消息仅能发送给在线用户
    pub async fn start(&mut self) -> io::Result<()> {
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
                self.login_user(thread_ident.clone());
                println!("user {} connect", thread_ident);
            } else {
                // todo 更好的错误处理
                panic!("client don't send connect first")
            }
            
            //read write 是对于client 而言
            match connect_type {
                //handle client write to server
                ConnectType::Write => {
                    self.handle_client_write(client);
                }
                //handle client read from server
                ConnectType::Read => {
                    let receiver = self.sender.subscribe();
                    self.handle_client_read(thread_ident, receiver, client);
                }
            };
            
            let len = self.send_list.len();
            for _ in 0..len {
                self.sender.send(self.send_list.pop().unwrap()).unwrap();
            }
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {}
}
