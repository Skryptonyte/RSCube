
use std::net::TcpStream;

pub struct Client {
    pub player_id: i8,
    pub stream: TcpStream,
    pub player_name: String
}

impl Client{
    pub fn new(player_id: i8, stream: TcpStream) -> Client
    {
        return Client {player_id: player_id,
            stream: stream,
            player_name: String::from("")};
    }
}