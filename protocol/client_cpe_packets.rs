
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
use crate::client_packets::*;
use crate::protocol::server_cpe_packets::*;
pub fn cpe_client_extinfo(server: &mut Server, cur: &mut Cursor<&Vec<u8>>, client_id: i8)
{
    let mut software_name: [u8; 64] = [0;64];
    cur.read_exact(&mut software_name).unwrap();

    let software_name = String::from_utf8_lossy(& software_name);
    let software_name = software_name.trim();

    let ext_count = cur.read_i16::<BigEndian>().unwrap();
    let client = server.clients.get_mut(&client_id).unwrap();

    client.extension_count_state = ext_count;
    println!("CPE Client joined with software: {}",software_name);
    println!("CPE Client Extension Count: {}",ext_count);
}

pub fn cpe_client_extentry(server: &mut Server, cur: &mut Cursor<&Vec<u8>>, client_id: i8)
{
    let mut app_name: [u8; 64] = [0;64];
    cur.read_exact(&mut app_name).unwrap();

    let app_name = String::from_utf8_lossy(& app_name);
    let app_name = app_name.trim();

    let version = cur.read_u32::<BigEndian>().unwrap();
    let client = server.clients.get_mut(&client_id).unwrap();

    client.extension_count_state = client.extension_count_state - 1;
    println!("CPE Extension Name: {} ( Version : {} )",app_name,version);

    match app_name
    {
        "FastMap" =>
        {
            println!("FastMap Enabled!");
            client.fastmap = true;
        }
        "CustomBlocks" =>
        {
            client.customblocksupportlevel = 1;
            client.expecting_customblock = 1;
        }
        _ =>
        {

        }
    }
    if client.extension_count_state == 0
    {
        if client.expecting_customblock == 0
        {
            println!("Resume normal login!");
            login_procedure(server, cur, client_id);
        }
        else
        {
            cpe_server_customblocklevelsupport(server, client_id, 1);
        }
    }
}

pub fn cpe_client_customblocksupportlevel(server: &mut Server, cur: &mut Cursor<&Vec<u8>>, client_id: i8)
{
    println!("Recieving Custom Block Level support!");

    let support_level = cur.read_u8();

    let client = server.clients.get_mut(&client_id).unwrap();

    if client.expecting_customblock == 1
    {
        client.expecting_customblock = 0;
        println!("Resume normal login!");
        login_procedure(server, cur, client_id);
    }
}
pub fn cpe_client_twowayping(server: &mut Server, cur: &mut Cursor<&Vec<u8>>, client_id: i8)
{
    let direction: u8 = cur.read_u8().unwrap();
    let unique_data: u16 = cur.read_u16::<BigEndian>().unwrap();

    if (direction == 0)
    {
        cpe_server_twowayping(server,client_id,direction,unique_data);
    }
}