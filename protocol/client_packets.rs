use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::io::prelude::*;
use std::cmp;

use std::io::prelude::*;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use std::fs::File;

use crate::server_packets::*;
use crate::client::Client;
use crate::server::Server;

use std::sync::Mutex;
use std::sync::Arc;
pub fn client_identification_packet(client: &mut Client)
{
    println!("Client packet recieved");

    let protocol: u8 = client.stream.read_u8().unwrap();

    let mut player_name: [u8; 64] = [0x0; 64];
    let mut verification_key: [u8; 64] = [0x0; 64];

    client.stream.read_exact(&mut player_name).unwrap();
    client.stream.read_exact(&mut verification_key).unwrap();

    let unused: u8 = client.stream.read_u8().unwrap();
    println!("Parsing packet");

    client.player_name = String::from_utf8_lossy(&player_name).trim().to_string();
    server_identification_packet(client,"&4 RSCube &4","This is an MC Classic server written in Rust");
    println!("Server identification delivered!");
    level_init(client);
    let world_coords = level_load(client);

    level_finalize(client,world_coords.0, world_coords.1, world_coords.2);
    spawn_player(client,-1,"",world_coords.3 << 5, world_coords.4 << 5,world_coords.5 << 5);
}

pub fn client_position_packet(client: &mut Client)
{

    let player_id: i8;
    let x: u16;
    let y: u16;
    let z: u16;

    let yaw: u8;
    let pitch: u8;
    
    {
    player_id = client.stream.read_i8().unwrap();
    x=client.stream.read_u16::<BigEndian>().unwrap();
    y=client.stream.read_u16::<BigEndian>().unwrap();
    z=client.stream.read_u16::<BigEndian>().unwrap();

    yaw=client.stream.read_u8().unwrap();
    pitch=client.stream.read_u8().unwrap();
    }

    //println!("Position: [ X: {}, Y: {}, Z: {} ]",X>>5,Y>>5,Z>>5);
}

pub fn client_set_block_packet(client: &mut Client)
{
    let x: u16 = client.stream.read_u16::<BigEndian>().unwrap();
    let y: u16 = client.stream.read_u16::<BigEndian>().unwrap();
    let z: u16 = client.stream.read_u16::<BigEndian>().unwrap();

    let mode: u8 = client.stream.read_u8().unwrap();
    let block_id: u8 = client.stream.read_u8().unwrap();

    println!("Set block for block ID: {} at position {} {} {} with mode: {}",block_id,x,y,z,mode);
}

pub fn client_chat_packet(client: &mut Client)
{
    let unused: u8 = client.stream.read_u8().unwrap();

    let mut string_buffer: [u8; 64] = [0; 64];

    client.stream.read_exact(&mut string_buffer);

    let chat_message = String::from_utf8_lossy(&string_buffer);
    println!("[CHAT] {}: {}",client.player_name,chat_message);

    let s: String = format!("{}: {}",client.player_name, chat_message);

    server_chat_packet(client, &s);
}