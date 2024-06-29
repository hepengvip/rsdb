use std::io::Error;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

extern crate packet;
extern crate storage;

use clap::Parser;
use packet::{Packet, PacketReaderWriter};
use storage::MultiDB;

#[derive(Parser, Debug)]
#[command(version, about, long_about = "RSDB client utility")]
struct Args {
    #[arg(short, long, default_value_t=String::from("127.0.0.1:1935"))]
    addr: String,

    #[arg(short, long)]
    root: String,
    // Number of times to greet
    // #[arg(short, long, default_value_t = 1)]
    // count: u8,
}

fn main() {
    let args = Args::parse();

    let s = Server::new(&args.addr, &args.root);
    s.listen_and_serve();
}

pub struct Server {
    // addr: Option<String>,
    listener: TcpListener,
    storage: Arc<Mutex<MultiDB>>,
    address: String,
    storage_dir: String,
}

impl Server {
    pub fn new(addr: &str, root: &str) -> Self {
        let server = Server {
            // addr: Some("127.0.0.1:1935".to_string()),
            listener: TcpListener::bind(addr).unwrap(),
            // storage: Arc::new(Storage::new_with_temp_dir("rsdb")),
            // storage: Arc::new(Storage::new(root)),
            storage: Arc::new(Mutex::new(MultiDB::new(root))),
            address: addr.to_string(),
            storage_dir: root.to_string(),
        };

        server
    }

    pub fn listen_and_serve(&self) {
        // Build a server
        println!("Listening at {}", &self.address);
        println!("Storage: {}", &self.storage_dir);
        for streams in self.listener.incoming() {
            match streams {
                Err(e) => {
                    eprintln!("error: {}", e)
                }
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

fn handler(stream: TcpStream, mdb: Arc<Mutex<MultiDB>>) -> Result<(), Error> {
    println!("Connection from {}", stream.peer_addr()?);

    let mut current_db_name = String::from("default");
    let mut rw = PacketReaderWriter::new(stream);
    loop {
        // let packet = rw.read_packet();
        let packet = rw.read_packet();
        if packet.is_err() {
            println!("Connection closed by client. {:?}", packet);
            break;
        }
        // println!("Received packet: {:?}", packet);
        let resp = match packet.unwrap() {
            Packet::CmdDelete(ref cmd) => {
                println!("Received delete command: {:?}", cmd);
                for key in cmd {
                    let msdb = mdb.lock().unwrap();
                    let db = msdb.get_db(&current_db_name).unwrap();
                    db.delete(key);
                }
                Packet::RespOk("Ok.".to_string())
            }
            Packet::CmdRead(ref cmd) => {
                println!("Received read command: {:?}", cmd);
                let mut values = Vec::new();
                for key in cmd {
                    let msdb = mdb.lock().unwrap();
                    let db = msdb.get_db(&current_db_name).unwrap();
                    let value = match db.get(key) {
                        Some(value) => value,
                        None => Vec::new(),
                    };
                    values.push(value);
                }
                Packet::RespTokens(values)
            }
            Packet::CmdWrite(ref cmd) => {
                println!("Received write command: {:?}", cmd);
                let pairs = cmd.len() / 2;
                for idx in 0..pairs {
                    let begin = idx * 2;
                    let msdb = mdb.lock().unwrap();
                    let db = msdb.get_db(&current_db_name).unwrap();
                    db.set(cmd.get(begin).unwrap(), cmd.get(begin + 1).unwrap())
                }
                Packet::RespOk("Ok.".to_string())
            }
            Packet::CmdUse(cmd) => {
                println!("Received use command: {:?}", cmd);
                current_db_name.clear();
                current_db_name = String::from_utf8(cmd).unwrap();

                let mut msdb = mdb.lock().unwrap();
                msdb.attach(&current_db_name);

                Packet::RespOk("Ok.".to_string())
            }
            Packet::CmdCurrentDB() => {
                println!("Received current db command");
                Packet::RespToken(current_db_name.as_bytes().to_vec())
            }
            _ => Packet::RespError("unknown command".to_string()),
        };
        rw.write_packet(&resp);
    }

    Ok(())
}
