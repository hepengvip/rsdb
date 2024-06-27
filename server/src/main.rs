use std::io::Error;
use std::net::{TcpListener, TcpStream};
use std::thread;

extern crate packet;

use packet::{Packet, PacketReaderWriter};


fn main() {
    let s = Server::new();
    s.listen_and_serve();
}

pub struct Server {
    // addr: Option<String>,
    listener: TcpListener,
}

impl Server {
    pub fn new() -> Self {
        let server = Server {
            // addr: Some("127.0.0.1:1935".to_string()),
            listener: TcpListener::bind("127.0.0.1:1935").unwrap(),
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
                    thread::spawn(move || {
                        handler(stream).unwrap_or_else(|error| eprintln!("{:?}", error));
                    });
                }
            }
        }
    }
}

fn handler(stream: TcpStream) -> Result<(), Error> {
    println!("Connection from {}", stream.peer_addr()?);

    let mut rw = PacketReaderWriter::new(stream);
    loop {
        let packet = rw.read_packet();
        println!("Received packet: {:?}", packet);
        let rsp = Packet::RespOk("All good.".to_string());
        rw.write_packet(&rsp);
    }
}
