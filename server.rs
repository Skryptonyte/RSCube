use std::net::TcpStream;
use std::collections::HashMap;

use std::io::Cursor;
use std::fs::File;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use flate2::read::GzDecoder;
use std::io::{Read, Write};
use crate::client::Client;
pub struct Server {
    pub server_name: String,
    pub server_motd: String,
    pub clients: HashMap<i8, Client>,
    pub world_data: Vec<u8>,
    pub world_x: u16,
    pub world_z: u16,
    pub world_y: u16,
    pub spawn_x: u16,
    pub spawn_y: u16,
    pub spawn_z: u16
}

impl Server{
    pub fn new(server_name: &str, server_motd: &str) -> Server
    {
        return Server {server_name: String::from(server_name), server_motd: String::from(server_motd), clients: HashMap::new(), 
            world_data: Vec::new(), world_x: 0, world_y: 0, world_z: 0, spawn_x: 0, spawn_y: 0, spawn_z: 0};
    }

    pub fn world_load(self: &mut Server)
        {
            let file = File::open("world.lvl").unwrap();

            let mut header: [u8; 18] = [0; 18];

            let mut d = GzDecoder::new(file);

            d.read_exact(&mut header).unwrap();

            let mut cur = Cursor::new(&header);
            let magicNumber = cur.read_u16::<LittleEndian>().unwrap();
            println!("Magic Number: {}", magicNumber);

            self.world_x = cur.read_u16::<LittleEndian>().unwrap();
            self.world_z = cur.read_u16::<LittleEndian>().unwrap();
            self.world_y = cur.read_u16::<LittleEndian>().unwrap();
            self.spawn_x = cur.read_u16::<LittleEndian>().unwrap();
            self.spawn_z = cur.read_u16::<LittleEndian>().unwrap();
            self.spawn_y = cur.read_u16::<LittleEndian>().unwrap();

            println!("World Size: {} x {} x {}", self.world_x, self.world_y, self.world_z);

            d.read_to_end(&mut self.world_data).unwrap();

        }
}