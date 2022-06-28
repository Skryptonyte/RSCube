use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::io::prelude::*;
use std::io::BufReader;
use std::cmp;

use std::sync::mpsc::channel;
use std::sync::mpsc;
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
use protocol::client_cpe_packets;


mod client;
mod server;
use crate::client::Client;
use crate::server::Server;

use std::collections::HashMap;
// Client packets


fn recieve_packets(mut stream: &mut TcpStream, player_id: i8, tx: & mpsc::Sender<(i8,u8, Vec<u8>)>) -> Result<(),std::io::Error>
{
    loop
    {

        let mut packetID: [u8; 1] = [0; 1];

        stream.read_exact(&mut packetID)?;

        match packetID[0]
        {
            0 => {
                let mut packet_data: [u8; 130] = [0;130];
                stream.read_exact(&mut packet_data)?;
                tx.send((player_id, packetID[0], packet_data.to_vec())).unwrap();

                //client_packets::client_identification_packet(&mut s, player_id);
            },
            5 => {
                let mut packet_data: [u8;  8] = [0;8];
                stream.read_exact(&mut packet_data)?;
                tx.send((player_id, packetID[0], packet_data.to_vec())).unwrap();

                //client_packets::client_set_block_packet(&mut s, player_id);
            },
            8 =>{
                let mut packet_data: [u8; 9] = [0;9];
                stream.read_exact(&mut packet_data)?;
                tx.send((player_id, packetID[0], packet_data.to_vec())).unwrap();

                //client_packets::client_position_packet(&mut s, player_id);
            },
            13 =>{
                let mut packet_data: [u8; 65] = [0;65];
                stream.read_exact(&mut packet_data)?;
                tx.send((player_id, packetID[0], packet_data.to_vec())).unwrap();

                //client_packets::client_chat_packet(&mut s, player_id);
            },
            16 =>
            {
                let mut packet_data: [u8; 66] = [0;66];
                stream.read_exact(&mut packet_data)?;
                tx.send((player_id, packetID[0], packet_data.to_vec())).unwrap();
            },
            17 =>
            {
                let mut packet_data: [u8; 68] = [0;68];
                stream.read_exact(&mut packet_data)?;
                tx.send((player_id, packetID[0], packet_data.to_vec())).unwrap();
            },
            0x2B =>
            {
                let mut packet_data: [u8; 3] = [0; 3];
                stream.read_exact(&mut packet_data)?;
                tx.send((player_id, packetID[0], packet_data.to_vec())).unwrap();
            },
            _ =>   {
                println!("Invalid packet ID {} recieved! Terminating", packetID[0]);
                break;
            }
            
        }


    }

    Ok(())
}
fn handle_client(mut stream: TcpStream, player_id: i8, tx: mpsc::Sender<(i8,u8, Vec<u8>)>)
{


    println!("Player ID {} connected!",player_id);
    //{
    //let mut server = rw_lock.lock().unwrap();
    //server.clients.insert(player_id, client);
    //}

    //let mut client = Client::new(player_id);
    match recieve_packets(&mut stream, player_id, & tx)
    {
        Ok(()) => 
        {
        println!("Client thread terminated cleanly!");
        tx.send((player_id,255,Vec::new()));
        }
        Err(e) => 
        {
        println!("Connection Closed! Terminating Thread!");
        tx.send((player_id,255,Vec::new()));
        }
    }

}


fn consumer_thread(rx: mpsc::Receiver<(i8,u8,Vec<u8>)>, server: Arc<Mutex<Server>>)
{
    for recieved in rx
    {
        //println!("Player {} sent Packet ID: {}", recieved.0, recieved.1);

        let packet_id: u8 = recieved.1;
        let player_id: i8 = recieved.0;
        let packet_data = recieved.2;
        let mut cur = Cursor::new(&packet_data);
        match packet_id
        {
            0 => {
                let mut s = server.lock().unwrap();
                client_packets::client_identification_packet(&mut s, &mut cur,player_id);

                let frmt_str = format!("{} has joined the world.", s.clients.get_mut(&player_id).unwrap().player_name);
                server_packets::server_chat_packet_broadcast(&mut s, -1,&frmt_str);
            },

            5 =>
            {
                let mut s = server.lock().unwrap();
                client_packets::client_set_block_packet(&mut s, &mut cur, player_id);
            },
            8 =>
            {
                let mut s = server.lock().unwrap();
                client_packets::client_position_packet(&mut s, &mut cur,player_id);
            },
            13 => 
            {
                let mut s = server.lock().unwrap();
                client_packets::client_chat_packet(&mut s, &mut cur, player_id);
            },
            0x10 =>
            {
                let mut s = server.lock().unwrap();
                client_cpe_packets::cpe_client_extinfo(&mut s, &mut cur, player_id);
            },
            0x11 =>
            {
                let mut s = server.lock().unwrap();
                client_cpe_packets::cpe_client_extentry(&mut s, &mut cur, player_id);
            },
            0x2B =>
            {
                let mut s = server.lock().unwrap();
                client_cpe_packets::cpe_client_twowayping(&mut s, &mut cur, player_id);
            }
            // Meta Packets for Server Management
            255 =>
            {
                let mut s = server.lock().unwrap();
                println!("Disconnect detected! Cleaning up player!");   

                let frmt_str = format!("{} has left the world.", s.clients.get_mut(&player_id).unwrap().player_name);
                server_packets::server_chat_packet_broadcast(&mut s,-1, &frmt_str);
                s.clients.remove(&player_id);
                server_packets::despawn_player_broadcast(&mut s,player_id);
            }
            _ =>
            {
                println!("Unrecognized packet ID recieved");
            }
            
        }
    }
}


fn main()
{
    let listener = TcpListener::bind("0.0.0.0:25565").unwrap();
    let server_rw = Arc::new(Mutex::new(Server::new("&4 RSCube &4","This is an MC Classic server written in Rust")));
    
    server_rw.lock().unwrap().world_load();

    let (tx, rx) = channel();

    let rw_lock_clone = Arc::clone(&server_rw);
    let client_add_lock = Arc::clone(&server_rw);

    thread::spawn(move || {
        consumer_thread(rx,rw_lock_clone);
    });

    for stream in listener.incoming()
    {
        let mut s = client_add_lock.lock().unwrap();

        let new_tx = tx.clone();
        let write_stream = stream.unwrap();
        let read_stream = write_stream.try_clone().unwrap();

        let i: i8 = s.add_client(write_stream);

        thread::spawn(move || {
            handle_client(read_stream, i,new_tx);
        });
    }
}