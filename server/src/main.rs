use std::io::Error;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

extern crate packet;
extern crate storage;

use clap::Parser;
use packet::{Packet, PacketReaderWriter};
use storage::{Direction, IteratorMode, MultiDB};

#[derive(Parser, Debug)]
#[command(version, about, long_about = "RSDB server")]
struct Args {
    #[arg(short, long, default_value_t=String::from("127.0.0.1:10110"))]
    addr: String,

    #[arg(short, long)]
    root: String,
    // Number of times to greet
    // #[arg(short, long, default_value_t = 1)]
    // count: u8,
}

fn main() {
    let args = Args::parse();

    let pkg_name = env!("CARGO_PKG_NAME");
    let pkg_version = env!("CARGO_PKG_VERSION");

    println!("\n\t{pkg_name} {pkg_version}\n");

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
        println!("    > Listening at {}", &self.address);
        println!("    > Storage: {}\n", &self.storage_dir);
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

    let mut rw = PacketReaderWriter::new(stream);
    let mut db: Option<Arc<storage::Storage>> = None;
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
                if db.is_some() {
                    for key in cmd {
                        db.as_ref().unwrap().delete(key);
                    }
                    Packet::RespOk("Ok.".to_string())
                } else {
                    Packet::RespError("no db selected".to_string())
                }
            }
            Packet::CmdRead(ref cmd) => {
                println!("Received read command: {:?}", cmd);
                if db.is_some() {
                    let mut values = Vec::new();
                    for key in cmd {
                        let db_rs = db.as_ref().unwrap().get(key);
                        let value = match db_rs {
                            Some(value) => value,
                            None => Vec::new(),
                        };
                        values.push(value);
                    }
                    Packet::RespTokens(values)
                } else {
                    Packet::RespError("no db selected".to_string())
                }
            }
            Packet::CmdWrite(ref cmd) => {
                println!("Received write command: {:?}", cmd);
                if db.is_some() {
                    let pairs = cmd.len() / 2;
                    for idx in 0..pairs {
                        let begin = idx * 2;
                        db.as_ref()
                            .unwrap()
                            .set(cmd.get(begin).unwrap(), cmd.get(begin + 1).unwrap())
                    }
                    Packet::RespOk("Ok.".to_string())
                } else {
                    Packet::RespError("no db selected".to_string())
                }
            }
            Packet::CmdUse(cmd) => {
                println!("Received use command: {:?}", cmd);
                let current_db_name = String::from_utf8(cmd).unwrap();

                db = {
                    let mut msdb = mdb.lock().unwrap();
                    msdb.attach(&current_db_name);
                    msdb.get_db(&current_db_name)
                };

                Packet::RespOk("Ok.".to_string())
            }
            Packet::CmdCurrentDB() => {
                println!("Received current db command");
                if db.is_some() {
                    let sdb = db.as_ref().unwrap().as_ref();
                    Packet::RespToken(sdb.path.as_ref().unwrap().as_bytes().to_vec())
                } else {
                    Packet::RespError("no db selected".to_string())
                }
            }
            Packet::CmdListDb() => {
                println!("Received list db command");
                let mut db_names = vec![];
                {
                    let msdb = mdb.lock().unwrap();
                    for name in msdb.list_db() {
                        db_names.push(name.to_owned().to_vec())
                    }
                }
                Packet::RespTokens(db_names)
            }
            Packet::CmdDetach(cmd) => {
                println!("Received detach command: {:?}", cmd);
                let detach_db = String::from_utf8(cmd).unwrap();
                if db.is_some() {
                    let sdb = db.as_ref().unwrap().as_ref();
                    if sdb.path.as_ref().unwrap() == &detach_db {
                        let _adb = db;
                        db = None;
                    }
                }

                {
                    let mut msdb = mdb.lock().unwrap();
                    msdb.detach(&detach_db);
                }

                Packet::RespOk("Ok.".to_string())
            }
            Packet::CmdRangeBegin(page_size) => {
                println!("Received range(begin) command: {:?}", page_size);
                if db.is_some() {
                    let it = db.as_ref().unwrap().this_db().iterator(IteratorMode::Start);
                    let mut tokens = vec![];
                    for rs in it.take(page_size as usize) {
                        let (k, v) = rs.unwrap();
                        tokens.push(k.to_vec());
                        tokens.push(v.to_vec());
                    }
                    Packet::RespPairs(tokens)
                } else {
                    Packet::RespError("no db selected".to_string())
                }
            }
            Packet::CmdRangeEnd(page_size) => {
                println!("Received range(end) command: {:?}", page_size);
                if db.is_some() {
                    let it = db.as_ref().unwrap().this_db().iterator(IteratorMode::End);
                    let mut tokens = vec![];
                    for rs in it.take(page_size as usize) {
                        let (k, v) = rs.unwrap();
                        tokens.push(k.to_vec());
                        tokens.push(v.to_vec());
                    }
                    Packet::RespPairs(tokens)
                } else {
                    Packet::RespError("no db selected".to_string())
                }
            }
            Packet::CmdRangeFromAsc(page_size, key) => {
                println!("Received range(from asc) command: {:?}", page_size);
                if db.is_some() {
                    let iter_mode = IteratorMode::From(key.as_slice(), Direction::Forward);
                    let it = db.as_ref().unwrap().this_db().iterator(iter_mode);
                    let mut tokens = vec![];
                    for rs in it.take(page_size as usize) {
                        let (k, v) = rs.unwrap();
                        tokens.push(k.to_vec());
                        tokens.push(v.to_vec());
                    }
                    Packet::RespPairs(tokens)
                } else {
                    Packet::RespError("no db selected".to_string())
                }
            }
            Packet::CmdRangeFromAscEx(page_size, key) => {
                println!("Received range(from asc) command: {:?}", page_size);
                if db.is_some() {
                    let iter_mode = IteratorMode::From(key.as_slice(), Direction::Forward);
                    let it = db.as_ref().unwrap().this_db().iterator(iter_mode);
                    let mut tokens = vec![];
                    for (idx, rs) in it.take(page_size as usize + 1).enumerate() {
                        let (k, v) = rs.unwrap();
                        let k_vec = k.to_vec();
                        if idx == 0 && k_vec == key {
                            continue;
                        }
                        tokens.push(k_vec);
                        tokens.push(v.to_vec());
                    }
                    Packet::RespPairs(tokens)
                } else {
                    Packet::RespError("no db selected".to_string())
                }
            }
            Packet::CmdRangeFromDesc(page_size, key) => {
                println!("Received range(from asc) command: {:?}", page_size);
                if db.is_some() {
                    let iter_mode = IteratorMode::From(key.as_slice(), Direction::Reverse);
                    let it = db.as_ref().unwrap().this_db().iterator(iter_mode);
                    let mut tokens = vec![];
                    for rs in it.take(page_size as usize) {
                        let (k, v) = rs.unwrap();
                        tokens.push(k.to_vec());
                        tokens.push(v.to_vec());
                    }
                    Packet::RespPairs(tokens)
                } else {
                    Packet::RespError("no db selected".to_string())
                }
            }
            Packet::CmdRangeFromDescEx(page_size, key) => {
                println!("Received range(from asc) command: {:?}", page_size);
                if db.is_some() {
                    let iter_mode = IteratorMode::From(key.as_slice(), Direction::Reverse);
                    let it = db.as_ref().unwrap().this_db().iterator(iter_mode);
                    let mut tokens = vec![];
                    for (idx, rs) in it.take(page_size as usize + 1).enumerate() {
                        let (k, v) = rs.unwrap();
                        let k_vec = k.to_vec();
                        if idx == 0 && k_vec == key {
                            continue;
                        }
                        tokens.push(k_vec);
                        tokens.push(v.to_vec());
                    }
                    Packet::RespPairs(tokens)
                } else {
                    Packet::RespError("no db selected".to_string())
                }
            }
            _ => Packet::RespError("unknown command".to_string()),
        };
        rw.write_packet(&resp);
    }

    Ok(())
}
