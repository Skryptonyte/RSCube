
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

pub fn cpe_client_extinfo(server: &mut Server, cur: &mut Cursor<&Vec<u8>>, client_id: i8)
{
    let mut software_name: [u8; 64] = [0;64];
    cur.read_exact(&mut software_name).unwrap();

    let software_name = String::from_utf8_lossy(& software_name);
    let software_name = software_name.trim();

    let ext_count = cur.read_i16::<BigEndian>().unwrap();
    let client = server.clients.get_mut(&client_id).unwrap();

    println!("CPE Client joined with software: {}",software_name);
    println!("CPE Client Extension Count: {}",ext_count);
}