use std::net::TcpStream;

pub struct Server {
    pub server_name: String,
    pub server_motd: String
}

impl Server{
    pub fn new(server_name: &str, server_motd: &str) -> Server
    {
        return Server {server_name: String::from(server_name), server_motd: String::from(server_motd)};
    }
}