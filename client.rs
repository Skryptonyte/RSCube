
use std::net::TcpStream;

pub struct Client {
    pub player_id: i8,
    pub stream: TcpStream,
    pub player_name: String,
    pub fastmap: bool,
    pub customblocksupportlevel: u8,

    pub extension_count_state: i16,
    pub expecting_customblock: u8
}

impl Client{
    pub fn new(player_id: i8, stream: TcpStream) -> Client
    {
        return Client {player_id: player_id,
            stream: stream,
            player_name: String::from(""),
            extension_count_state: 0,
            customblocksupportlevel: 0,
            expecting_customblock: 1,
            fastmap: false};
    }
}