

use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::io::prelude::*;
use std::cmp;

use std::io::prelude::*;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Cursor;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use std::fs::File;
use crate::client::Client;

use std::io::BufReader;
use std::io::BufWriter;

use crate::server::Server;

pub fn cpe_server_extinfo(server: &mut Server, client_id: i8, ext_count: i16)
{
    println!("CPE Handshake has begun!");
    let mut packet: Vec<u8> = Vec::new();

    packet.write_u8(0x10);

    for i in 0..cmp::min(server.software_name.len(),64)
    {
        packet.push(server.software_name.as_bytes()[i]);
    }

    for i in cmp::min(server.software_name.len(),64)..64
    {
        packet.push(0x20);
    }
    packet.write_i16::<BigEndian>(ext_count).unwrap();

    let client = server.clients.get_mut(&client_id).unwrap();

    client.stream.write(&packet).unwrap();
}