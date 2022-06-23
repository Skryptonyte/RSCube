use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::io::prelude::*;
use std::io::BufReader;
use std::cmp;

use std::io::prelude::*;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Cursor;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use std::fs::File;
use std::sync::{Mutex, Arc, RwLock};
mod protocol;

use protocol::client_packets;
use protocol::server_packets;

mod client;
mod server;
use crate::client::Client;
use crate::server::Server;
// Client packets

fn handle_client(mut stream: TcpStream, player_id: i8, mut rw_lock: Arc<Mutex<Server>>)
{
    println!("Connected");
    
    let mut client = Client::new(player_id,stream);

    loop
    {
        let mut s = rw_lock.lock().unwrap();

        let mut packetID: [u8; 1] = [0; 1];
        client.stream.read_exact(&mut packetID).unwrap();

        match packetID[0]
        {
            0 => {
                client_packets::client_identification_packet(&mut client);
            },
            5 => {
                client_packets::client_set_block_packet(&mut client);
            },
            8 =>{
                client_packets::client_position_packet(&mut client);
            }
            13 =>{
                client_packets::client_chat_packet(&mut client);
            }
            _ =>   {println!("Invalid packet ID {} recieved! Terminating", packetID[0]);
            return;}
        }

    }
}

fn main()
{
    let listener = TcpListener::bind("0.0.0.0:25565").unwrap();
    let server_rw = Arc::new(Mutex::new(Server::new("RSCube Server", "Lol")));
    
    let i: i8 = 0;
    for stream in listener.incoming()
    {
        let rw_lock_clone = Arc::clone(&server_rw);

        thread::spawn(move || {
            handle_client(stream.unwrap(), i,rw_lock_clone);
        });
    }
}