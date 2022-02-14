use clap::clap_app;

use crate::client::Client;
use crate::request::{ClientRequest, Connect, ConnectType, Message};
use crate::server::Server;
use crate::utils::to_string;

mod request;
mod server;
mod args;
mod utils;
mod client;

const SOCKET_ADDRS: &'static str = "127.0.0.1:8000";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = clap_app!(socket =>
        (@arg SERVER:-s --server)
        (@arg CLIENT: -c --client)
        (@arg USERNAME: -u --username + takes_value )
    ).get_matches();
    
    if matches.is_present("SERVER") {
        let mut server = Server::new(SOCKET_ADDRS).await?;
        server.start().await?;
    } else {
        let username = if matches.is_present("USERNAME") {
            matches.value_of("USERNAME").unwrap()
        } else {
            "test"
        };
        println!("your username is {}", username);
        let mut client = Client::new(username).await?;
        client.start().await;
    }
    
    Ok(())
}

