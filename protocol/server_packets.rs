
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::io::prelude::*;
use std::cmp;

use std::io::prelude::*;
use flate2::read::GzDecoder;
use flate2::write::{GzEncoder, DeflateEncoder};
use flate2::Compression;
use std::io::Cursor;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use std::fs::File;
use crate::client::Client;

use std::io::BufReader;
use std::io::BufWriter;

use crate::server::Server;

pub fn server_identification_packet(server: &mut Server, client_id: i8)
{
    let mut packet: [u8; 131] = [0;131];
    packet[0] = 0x0;
    packet[1] = 0x7;

    for i in 0..cmp::min(64,server.server_name.len())
    {
        packet[2+i] = server.server_name.as_bytes()[i];
    }

    for i in 0..cmp::min(64,server.server_motd.len())
    {
        packet[66+i] = server.server_motd.as_bytes()[i];
    }

    let client = &mut server.clients.get_mut(&client_id).unwrap();
    println!("Sending packet: {}",String::from_utf8_lossy(&packet));
    match client.stream.write(&packet)
    {
        Ok(_) => {},
        Err(e) => {}
    }
}

pub fn level_init(server: &mut Server, client_id: i8)
{
    let client = server.clients.get_mut(&client_id).unwrap();

    let use_fastmap: bool = client.fastmap;

    if (use_fastmap)
    {
        let mut packet: Vec<u8> = Vec::new();

        packet.write_u8(2);
        let world_size:u32 = (u32::from(server.world_x) * u32::from(server.world_y) * u32::from(server.world_z)).into();
        packet.write_u32::<BigEndian>(world_size);
        client.stream.write(&packet).unwrap();
    }
    else{
        let mut packet: [u8; 1] = [2];
        match client.stream.write(&packet)
        {
            Ok(_) => {},
            Err(e) => {}
        }
    }
}

pub fn level_load(server: &mut Server, client_id: i8) -> (u16, u16, u16, u16, u16, u16)
{
    let tuple: (u16, u16, u16, u16, u16, u16);
    
    let (world_X, world_Y, world_Z, spawn_X, spawn_Y, spawn_Z) = (server.world_x, server.world_y, server.world_z, server.spawn_x, server.spawn_y, server.spawn_z);
    let mut gzipped_map: Vec<u8> = Vec::new();

    let mut data: Vec<u8> = Vec::new();


    let use_fastmap: bool = server.clients.get(&client_id).unwrap().fastmap;

    let world_size:u32 = (u32::from(world_X) * u32::from(world_Y) * u32::from(world_Z)).into();

    if (!use_fastmap)
    {
        println!("Loading level [Fastmap: false]");
        let mut e = GzEncoder::new(&mut gzipped_map, Compression::default());
        e.write_u32::<BigEndian>(world_size).unwrap();
        e.write(& server.world_data).unwrap();
    }
    else
    {
        println!("Loading level [Fastmap: true]");
        let mut e = DeflateEncoder::new(&mut gzipped_map, Compression::default());
        e.write(& server.world_data).unwrap();
    }
    println!("World data size: {}", data.len());
    println!("Gzipped data size: {}", gzipped_map.len());

    let client = server.clients.get(&client_id).unwrap();

    let mut i: usize = 0;
    while usize::from(i) < gzipped_map.len()
    {
        let chunk_end: usize = cmp::min(gzipped_map.len(),i+1024);
        let chunk_length: usize = (chunk_end - i).into();

        let mut packet: Vec<u8> = Vec::new();
        packet.write_u8(0x3).unwrap();
        packet.write_u16::<BigEndian>(chunk_length.try_into().unwrap()).unwrap();

        for j in 0..chunk_length
        {
            packet.push(gzipped_map[i+j]);  
        }

        for j in chunk_length..1024
        {
            packet.push(0);  
        }
        packet.write_u8(0x0).unwrap();

        println!("Sending level chunk [Offset: {}, Size: {}]",i,chunk_length);
        let mut stream = & client.stream;
        match stream.write(&packet)
        {
            Ok(_) => {},
            Err(e) => {}
        }

        i += 1024;
    }

    return (world_X, world_Y, world_Z, spawn_X, spawn_Y, spawn_Z);
}

pub fn level_finalize(client: &mut Client, world_x: u16, world_y:u16, world_z:u16)
{
    let mut packet: Vec<u8> = Vec::new();
    packet.write_u8(0x4).unwrap();
    packet.write_u16::<BigEndian>(world_x).unwrap();
    packet.write_u16::<BigEndian>(world_y).unwrap();
    packet.write_u16::<BigEndian>(world_z).unwrap();
    println!("Finalizing Level Load");
    match client.stream.write(&packet)
    {
        Ok(_) => {},
        Err(e) => {}
    }
}

pub fn spawn_player(client: & Client, player_id: i8,player_name: &str, spawn_x: u16, spawn_y: u16, spawn_z: u16)
{
    println!("Sending spawn player");
    let mut packet: Vec<u8> = Vec::new();

    packet.write_u8(0x7).unwrap();
    packet.write_i8(player_id).unwrap();

    for i in 0..player_name.len()
    {
        packet.push(player_name.as_bytes()[i]);
    }

    for _i in player_name.len()..64
    {
        packet.push(0x20);
    }
    packet.write_u16::<BigEndian>(spawn_x).unwrap();
    packet.write_u16::<BigEndian>(spawn_y).unwrap();
    packet.write_u16::<BigEndian>(spawn_z).unwrap();

    packet.write_u8(0x0).unwrap();
    packet.write_u8(0x0).unwrap();


    let mut stream = &client.stream;
    match stream.write(&packet)
    {
        Ok(_) => {},
        Err(e) => {}
    }

}


pub fn despawn_player_broadcast(server: & Server, player_id: i8)
{
    let mut packet: Vec<u8> = Vec::new();

    packet.write_u8(0xc);
    packet.write_i8(player_id);

    for (c_id, client) in & server.clients
    {
        let mut stream = &client.stream;
        match stream.write(&packet)
        {
            Ok(_) => {},
            Err(e) => {}
        }
    }
}

pub fn server_chat_packet(server: &mut Server ,client_id: i8,message: &str)
{
    let client = & server.clients.get(&client_id).unwrap();
    let mut stream = & client.stream;


    let mut i: usize = 0;

    while (i < message.len())
    {
        println!("Offset: {}, Message Length: {}",i,message.len());
        let mut packet: Vec<u8> = Vec::new();

        packet.write_u8(0xd).unwrap();
        packet.write_i8(client_id).unwrap();
    
        for j in i..cmp::min(i+64,message.len())
        {
            packet.push(message.as_bytes()[j]);
        }
    
        for j in cmp::min(i+64,message.len())..(i+64)
        {
            packet.push(0x20);
        }

        match stream.write(&packet)
        {
            Ok(_) => {},
            Err(e) => {}
        };
        i += 64;
    }
}

pub fn server_chat_packet_broadcast(server: &mut Server ,client_id: i8, message: &str)
{
    let mut i: usize = 0;

    while (i < message.len())
    {
        println!("Offset: {}, Message Length: {}",i,message.len());
        let mut packet: Vec<u8> = Vec::new();

        packet.write_u8(0xd).unwrap();
        packet.write_i8(client_id).unwrap();
    
        for j in i..cmp::min(i+64,message.len())
        {
            packet.push(message.as_bytes()[j]);
        }
    
        for j in cmp::min(i+64,message.len())..(i+64)
        {
            packet.push(0x20);
        }

        for (player_id, client) in &mut server.clients
        {

            match client.stream.write(&packet)
            {
                Ok(_) => {},
                Err(e) => {}
            }
        }

        i += 64;
    }
}

pub fn server_set_block_packet_broadcast(server: &mut Server, x: u16, y: u16, z: u16, block_id: u8)
{
    let mut packet: Vec<u8> = Vec::new();

    let block_id_fallback = match block_id
    {
        0x32 => 0x2c,
        0x33 => 0x27,
        0x34 => 0xc,
        0x35 => 0x0,
        0x36 => 0xa,
        0x37 => 0x21,
        0x38 => 0x19,
        0x39 => 0x3,
        0x3a => 0x1d,
        0x3b => 0x1c,
        0x3c => 0x14,
        0x3d => 0x2a,
        0x3e => 0x31,
        0x3f => 0x24,
        0x40 => 0x5,
        0x41 => 0x1,
        _ => block_id
    };
    packet.write_u8(0x6).unwrap();

    packet.write_u16::<BigEndian>(x).unwrap();
    packet.write_u16::<BigEndian>(y).unwrap();
    packet.write_u16::<BigEndian>(z).unwrap();

    packet.write_u8(block_id).unwrap();

    let world_x = server.world_x;
    let world_y = server.world_y;
    let world_z = server.world_z;
    {
        let world_data = &mut server.world_data;
        let calculated_index: u32 = u32::from(x) + u32::from(world_x) * ( u32::from(z) + u32::from( world_z ) * u32::from(y)) ;
        let calculated_usize: usize = calculated_index.try_into().unwrap();
        world_data[calculated_usize] = block_id;
    }
    for (player_id, client) in & server.clients
    {
        println!("Block change sent with block_id: {}", block_id);
        let mut stream = & client.stream;   

        if (client.customblocksupportlevel >= 1)
        {
            packet[7] = block_id;
        }
        else
        {
            packet[7] = block_id_fallback;
        }
        match stream.write(&packet)
        {
            Ok(_) => {},
            Err(e) => {}
        }    
    }
}

pub fn server_position_packet(server: &mut Server, client_id: i8, x: u16, y: u16, z: u16, yaw: u8, pitch: u8)
{
    let mut packet: Vec<u8> = Vec::new();

    packet.write_u8(0x8).unwrap();
    packet.write_i8(-1).unwrap();

    packet.write_u16::<BigEndian>(x).unwrap();
    packet.write_u16::<BigEndian>(y).unwrap();
    packet.write_u16::<BigEndian>(z).unwrap();

    packet.write_u8(yaw).unwrap();
    packet.write_u8(pitch).unwrap();


    let client = server.clients.get_mut(&client_id).unwrap();
    match client.stream.write(&packet)
    {
        Ok(_) => {},
        Err(e) => {}
    }

}

pub fn server_position_packet_broadcast(server: &mut Server, calling_player_id: i8, x: u16, y: u16, z: u16, yaw: u8, pitch: u8)
{
    let mut packet: Vec<u8> = Vec::new();

    packet.write_u8(0x8).unwrap();
    packet.write_i8(calling_player_id).unwrap();

    packet.write_u16::<BigEndian>(x).unwrap();
    packet.write_u16::<BigEndian>(y).unwrap();
    packet.write_u16::<BigEndian>(z).unwrap();

    packet.write_u8(yaw).unwrap();
    packet.write_u8(pitch).unwrap();


    for (player_id, client) in &mut server.clients
    {
        if (*player_id != calling_player_id)
        {
            match client.stream.write(&packet)
            {
                Ok(_) => {},
                Err(e) => {}
            }
        }
    }
}

pub fn server_disconnect_player(server: &mut Server, client_id: i8, message: &str)
{
    let mut packet: Vec<u8> = Vec::new();

    packet.write_u8(0xe).unwrap();

    for i in 0..cmp::min(message.len(),64)
    {
        packet.write_u8(message.as_bytes()[i]);
    }

    for i in cmp::min(message.len(),64)..64
    {
        packet.write_u8(0x20);
    }
    match server.clients.get_mut(&client_id).unwrap().stream.write(&packet)
    {
        Ok(_) => {},
        Err(e) => {}
    }
}

pub fn server_update_user_type(server: &mut Server, client_id: i8, user_type: u8)
{
    let mut packet: Vec<u8> = Vec::new();

    packet.write_u8(0xf).unwrap();
    packet.write_u8(user_type).unwrap();

    match server.clients.get_mut(&client_id).unwrap().stream.write(&packet)
    {
        Ok(_) => {},
        Err(e) => {}
    }
}
