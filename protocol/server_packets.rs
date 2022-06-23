
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

pub fn server_identification_packet(client: &mut Client, server_name: &str, motd: &str)
{
    let mut packet: [u8; 131] = [0;131];
    let s: &str= "Rusty Cunt";
    packet[0] = 0x0;
    packet[1] = 0x7;

    for i in 0..cmp::min(64,server_name.len())
    {
        packet[2+i] = server_name.as_bytes()[i];
    }

    for i in 0..cmp::min(64,motd.len())
    {
        packet[66+i] = motd.as_bytes()[i];
    }

    println!("Sending packet: {}",String::from_utf8_lossy(&packet));
    client.stream.write(&packet).unwrap();
}

pub fn level_init(client: &mut Client)
{
    let mut packet: [u8; 1] = [2];

    client.stream.write(&packet).unwrap();
}

pub fn level_load(client: &mut Client) -> (u16, u16, u16, u16, u16, u16)
{
    let mut file = File::open("world.lvl").unwrap();

    let mut header: [u8; 18] = [0; 18];
    let mut data: Vec<u8> = Vec::new();

    let mut d = GzDecoder::new(file);

    d.read_exact(&mut header).unwrap();

    let mut cur = Cursor::new(&header);
    let magicNumber = cur.read_u16::<LittleEndian>().unwrap();
    println!("Magic Number: {}", magicNumber);

    let world_X: u16 = cur.read_u16::<LittleEndian>().unwrap();
    let world_Z: u16 = cur.read_u16::<LittleEndian>().unwrap();
    let world_Y: u16 = cur.read_u16::<LittleEndian>().unwrap();

    let spawn_X: u16 = cur.read_u16::<LittleEndian>().unwrap();
    let spawn_Z: u16 = cur.read_u16::<LittleEndian>().unwrap();
    let spawn_Y: u16 = cur.read_u16::<LittleEndian>().unwrap();

    println!("World Size: {} x {} x {}", world_X, world_Y, world_Z);


    let mut cur2 = Cursor::new(&data);
    let world_size:u32 = (u32::from(world_X) * u32::from(world_Y) * u32::from(world_Z)).into();

    data.write_u32::<BigEndian>(world_size).unwrap();
    d.read_to_end(&mut data).unwrap();

    let mut gzipped_map: Vec<u8> = Vec::new();
    {
    let mut e = GzEncoder::new(&mut gzipped_map, Compression::default());
    e.write(&data).unwrap();
    }
    println!("World data size: {}", data.len());
    println!("Gzipped data size: {}", gzipped_map.len());

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
        client.stream.write(&packet).unwrap();

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
    client.stream.write(&packet).unwrap();
}

pub fn spawn_player(client: &mut Client, player_id: i8,player_name: &str, spawn_x: u16, spawn_y: u16, spawn_z: u16)
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


    client.stream.write(&packet).unwrap();

}

pub fn server_chat_packet(client: &mut Client, message: &str)
{
    let mut packet: Vec<u8> = Vec::new();

    packet.write_u8(0xd).unwrap();
    packet.write_i8(-1).unwrap();


    for i in 0..cmp::min(64,message.len())
    {
        packet.push(message.as_bytes()[i]);
    }
    client.stream.write(&packet).unwrap();
}