use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::io::prelude::*;
use std::cmp;

use std::io::prelude::*;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use std::fs::File;

use crate::server_packets::*;
use crate::protocol::server_cpe_packets::*;
use crate::client::Client;
use crate::server::Server;

use std::sync::Mutex;
use std::sync::Arc;
use std::io::Cursor;


pub fn login_procedure(server: &mut Server, cur: &mut Cursor<&Vec<u8>>, client_id: i8)
{
    let world_coords: (u16, u16, u16, u16, u16, u16);

    server_identification_packet(server,client_id);
    
    println!("Server identification delivered!");
    {
    level_init(server, client_id);
    }
    {
    world_coords = level_load(server, client_id);
    }

    let client = server.clients.get_mut(&client_id).unwrap();

    level_finalize(client,world_coords.0, world_coords.1, world_coords.2);
    spawn_player(client,-1,"",world_coords.3 << 5, world_coords.4 << 5,world_coords.5 << 5);

    
    let client_table = & server.clients;
    let main_client = & server.clients.get(&client_id).unwrap();
    for (player_id, other_client) in client_table
    {
        if (*player_id != client_id)
        {
        spawn_player(other_client,client_id,&main_client.player_name,world_coords.3 << 5, world_coords.4 << 5,world_coords.5 << 5);
        spawn_player(main_client, *player_id, &other_client.player_name, world_coords.3 << 5, world_coords.4 << 5,world_coords.5 << 5);
        }
    }
}
pub fn client_identification_packet(server: &mut Server, cur: &mut Cursor<&Vec<u8>>, client_id: i8)
{
    println!("Client packet recieved");

    let mut player_name: [u8; 64] = [0x0; 64];
    let mut verification_key: [u8; 64] = [0x0; 64];

    

    let protocol: u8 = cur.read_u8().unwrap();

    let mut player_name: [u8; 64] = [0x0; 64];
    let mut verification_key: [u8; 64] = [0x0; 64];

    cur.read_exact(&mut player_name).unwrap();
    cur.read_exact(&mut verification_key).unwrap();

    let unused: u8 = cur.read_u8().unwrap();


    println!("Parsing packet");

    
    {
        let client = server.clients.get_mut(&client_id).unwrap();
        client.player_name = String::from_utf8_lossy(&player_name).trim().to_string();
    }
    if (unused == 0x42)
    {
        cpe_server_extinfo(server,client_id,3);
        cpe_server_extentry(server, client_id, "TwoWayPing", 1);
        cpe_server_extentry(server, client_id, "CustomBlocks", 1);
        cpe_server_extentry(server, client_id, "FastMap", 1);

    }
    else{
        login_procedure(server, cur, client_id);
    }
}

pub fn client_position_packet(server: &mut Server, cur: &mut Cursor<&Vec<u8>>,client_id: i8)
{

    let player_id: i8;
    let x: u16;
    let y: u16;
    let z: u16;

    let yaw: u8;
    let pitch: u8;
    
    let client = &mut server.clients.get_mut(&client_id).unwrap();

    {
    player_id = cur.read_i8().unwrap();
    x=cur.read_u16::<BigEndian>().unwrap();
    y=cur.read_u16::<BigEndian>().unwrap();
    z=cur.read_u16::<BigEndian>().unwrap();

    yaw=cur.read_u8().unwrap();
    pitch=cur.read_u8().unwrap();
    }

    server_position_packet_broadcast(server, client_id,x,y,z,yaw,pitch);
    //println!("Position: [ X: {}, Y: {}, Z: {} ]",X>>5,Y>>5,Z>>5);
}

pub fn client_set_block_packet(server: &mut Server, cur:&mut Cursor<&Vec<u8>>, client_id: i8)
{
    let client = &mut server.clients.get_mut(&client_id).unwrap();

    let x: u16 = cur.read_u16::<BigEndian>().unwrap();
    let y: u16 = cur.read_u16::<BigEndian>().unwrap();
    let z: u16 = cur.read_u16::<BigEndian>().unwrap();

    let mode: u8 = cur.read_u8().unwrap();
    let block_id: u8 = cur.read_u8().unwrap();

    println!("Set block for block ID: {} at position {} {} {} with mode: {}",block_id,x,y,z,mode);
    if (mode == 1)
    {
        server_set_block_packet_broadcast(server, x,y,z,block_id);
    }
    else{
        server_set_block_packet_broadcast(server, x,y,z,0);

    }
}

fn parse_command(server: &mut Server, client_id: i8, cmd_str: &str) -> Result<(),Box<dyn std::error::Error>>
{

    let args = cmd_str.split_whitespace().collect::<Vec<&str>>();

    if args[0].eq("tp")
    {
        if args.len() != 4
        {
            server_chat_packet(server,client_id, "Syntax: /tp <X> <Y> <Z>");
            return Ok(());
        }

        let X: u16 = args[1].parse()?;
        let Y: u16 = args[2].parse()?;
        let Z: u16 = args[3].parse()?;


        server_chat_packet(server,client_id, "Teleporting");
        server_position_packet(server, client_id, X<<5, Y<<5, Z<<5, 0, 0);
    }
    else if args[0].eq("op")
    {
        server_update_user_type(server,client_id,0x64);
        server_chat_packet(server,client_id,"You are now OP");
    }
    else if args[0].eq("kickself")
    {
        server_disconnect_player(server,client_id,"TEST KICK!");
    }
    else if args[0].eq("save")
    {
        server.world_save();
    }
    else{
        server_chat_packet(server,client_id,&format!("No such command: {}",args[0]));
    }

    return Ok(());
}
pub fn client_chat_packet(server: &mut Server,cur: &mut Cursor<&Vec<u8>>,client_id: i8)
{
    let client = server.clients.get_mut(&client_id).unwrap();

    let unused: u8 = cur.read_u8().unwrap();

    let mut string_buffer: [u8; 64] = [0; 64];
    cur.read_exact(&mut string_buffer).unwrap();

    let chat_message = String::from_utf8_lossy(&string_buffer);
    let chat_message = chat_message.trim();
    println!("[CHAT] {}: {}",client.player_name,&chat_message);

    if (chat_message.starts_with("/"))
    {
        let command = &chat_message[1..];
        match (parse_command(server, client_id, command))
        {
            Ok(_) => {
                println!("Command executed successfully");
            }
            Err(e) =>
            {
                println!("[WARN] An error occurred during execution of command!");
            }
        }

    }
    else
    {
        let s: String = format!("{}: {}",client.player_name, chat_message);
        server_chat_packet_broadcast(server, client_id, &s);
    }
}