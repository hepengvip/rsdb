use std::io::Error;
use std::net::{TcpListener, TcpStream};
use std::{sync::Arc, thread};

extern crate packet;
extern crate storage;

use packet::{Packet, PacketReaderWriter};
use storage::Storage;


fn main() {
    let s = Server::new();
    s.listen_and_serve();
}

pub struct Server {
    // addr: Option<String>,
    listener: TcpListener,
    storage: Arc<Storage>,
}

impl Server {
    pub fn new() -> Self {
        let server = Server {
            // addr: Some("127.0.0.1:1935".to_string()),
            listener: TcpListener::bind("127.0.0.1:1935").unwrap(),
            storage: Arc::new(Storage::new_with_temp_dir("rsdb")),
        };

        server
    }

    pub fn listen_and_serve(&self) {
        // Build a server
        println!("Listening...");
        for streams in self.listener.incoming() {
            match streams {
                Err(e) => { eprintln!("error: {}", e) },
                Ok(stream) => {
                    let db_copy = self.storage.clone();
                    thread::spawn(move || {
                        handler(stream, db_copy).unwrap_or_else(|error| eprintln!("{:?}", error));
                    });
                }
            }
        }
    }
}

fn handler(stream: TcpStream, db: Arc<Storage>) -> Result<(), Error> {
    println!("Connection from {}", stream.peer_addr()?);

    let mut rw = PacketReaderWriter::new(stream);
    loop {
        let packet = rw.read_packet();
        if packet.is_err() {
            println!("Connection closed by client. {:?}", packet);
            break;
        }
        println!("Received packet: {:?}", packet);
        let resp = match packet.unwrap() {
            Packet::CmdDelete(ref cmd) => {
                println!("Received delete command: {:?}", cmd);
                for key in cmd {
                    db.delete(key);
                };
                Packet::RespOk("Ok.".to_string())
            },
            Packet::CmdRead(ref cmd) => {
                println!("Received read command: {:?}", cmd);
                let mut values = Vec::new();
                for key in cmd {
                    let value = match db.get(key) {
                        Some(value) => value,
                        None => Vec::new(),
                    };
                    values.push(value);
                };
                Packet::RespTokens(values)
            },
            Packet::CmdWrite(ref cmd) => {
                println!("Received write command: {:?}", cmd);
                let pairs = cmd.len() / 2;
                for idx in 0..pairs {
                    let begin = idx * 2;
                    db.set(
                        cmd.get(begin).unwrap(),
                        cmd.get(begin + 1).unwrap(),
                    )
                };
                Packet::RespOk("Ok.".to_string())
            },
            _ => panic!("Unexpected packet type"),
        };
        rw.write_packet(&resp);
    };

    Ok(())
}
