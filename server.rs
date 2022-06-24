use std::net::TcpStream;
use std::collections::HashMap;

use crate::client::Client;
pub struct Server {
    pub server_name: String,
    pub server_motd: String,
    pub clients: HashMap<i8, Client>
}

impl Server{
    pub fn new(server_name: &str, server_motd: &str) -> Server
    {
        return Server {server_name: String::from(server_name), server_motd: String::from(server_motd), clients: HashMap::new() };
    }
}