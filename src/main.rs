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
    ).get_matches();
    
    if matches.is_present("SERVER") {
        let server = Server::new(SOCKET_ADDRS).await?;
        server.start().await?;
    } else {
        let mut client = Client::new("test").await?;
        client.start().await;
    }
    
    Ok(())
}

