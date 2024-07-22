use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

extern crate packet;
extern crate storage;

use clap::Parser;
use packet::{Packet, PacketReaderWriter};
use storage::{Direction, IteratorMode, MultiDB};

mod errors;

use errors::{ServerError, ServerResult};

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
    match s {
        Err(e) => eprintln!("Error: {}", e),
        Ok(s) => s.listen_and_serve(),
    }
}

pub struct Server {
    // addr: Option<String>,
    listener: TcpListener,
    storage: Arc<Mutex<MultiDB>>,
    address: String,
    storage_dir: String,
}

impl Server {
    pub fn new(addr: &str, root: &str) -> ServerResult<Self> {
        let server = Server {
            // addr: Some("127.0.0.1:1935".to_string()),
            listener: TcpListener::bind(addr)?,
            // storage: Arc::new(Storage::new_with_temp_dir("rsdb")),
            // storage: Arc::new(Storage::new(root)),
            storage: Arc::new(Mutex::new(MultiDB::new(root))),
            address: addr.to_string(),
            storage_dir: root.to_string(),
        };

        Ok(server)
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

fn handler(stream: TcpStream, mdb: Arc<Mutex<MultiDB>>) -> ServerResult<()> {
    let peer_addr = stream.peer_addr()?;
    println!("Connection from {}", peer_addr);
    stream.set_nodelay(true)?;

    let mut rw = PacketReaderWriter::new(stream);
    let mut db: Option<Arc<storage::Storage>> = None;
    loop {
        // let packet = rw.read_packet();
        let packet = rw.read_packet();
        if packet.is_err() {
            println!("Connection closed by client:{}.", peer_addr);
            break;
        }
        let resp = match packet? {
            Packet::CmdDelete(ref cmd) => match db.as_ref() {
                Some(sdb) => {
                    for key in cmd {
                        sdb.delete(key)?
                    }
                    Packet::RespOk("Ok.".to_string())
                }
                None => Packet::RespError("no db selected".to_string()),
            },
            Packet::CmdRead(ref cmd) => match db.as_ref() {
                Some(sdb) => {
                    let mut values = Vec::new();
                    for key in cmd {
                        let db_rs = sdb.get(key)?;
                        let value = match db_rs {
                            Some(value) => value,
                            None => Vec::new(),
                        };
                        values.push(value);
                    }
                    Packet::RespTokens(values)
                }
                None => Packet::RespError("no db selected".to_string()),
            },
            Packet::CmdWrite(ref cmd) => match db.as_ref() {
                Some(sdb) => {
                    let pairs = cmd.len() / 2;
                    for idx in 0..pairs {
                        let begin = idx * 2;
                        sdb.set(
                            cmd.get(begin).ok_or(ServerError::InvalidData)?,
                            cmd.get(begin + 1).ok_or(ServerError::InvalidData)?,
                        )?
                    }
                    Packet::RespOk("Ok.".to_string())
                }
                None => Packet::RespError("no db selected".to_string()),
            },
            Packet::CmdUse(cmd) => {
                // println!("Received use command");
                let current_db_name = String::from_utf8(cmd)?;

                let unlocked_db = mdb.lock();
                match unlocked_db {
                    Ok(mut msdb) => {
                        msdb.attach(&current_db_name)?;
                        db = msdb.get_db(&current_db_name);
                        Packet::RespOk("Ok.".to_string())
                    }
                    Err(_) => Packet::RespError("get lock failed".to_string()),
                }
            }
            Packet::CmdCurrentDB() => match db.as_ref() {
                Some(sdb) => {
                    let tmpname = "<temp path>".to_string();
                    let dbname = sdb.path.as_ref().unwrap_or(&tmpname);
                    Packet::RespToken(dbname.as_bytes().to_vec())
                }
                None => Packet::RespError("no db selected".to_string()),
            },
            Packet::CmdListDb() => {
                let db = mdb.lock();
                match db {
                    Ok(msdb) => {
                        let mut db_names = vec![];
                        for name in msdb.list_db() {
                            db_names.push(name.to_owned().to_vec())
                        }
                        Packet::RespTokens(db_names)
                    }
                    Err(_) => Packet::RespError("get lock failed".to_string()),
                }
            }
            Packet::CmdDetach(cmd) => {
                let detach_db = String::from_utf8(cmd)?;
                if let Some(sdb) = db.as_ref() {
                    if let Some(path) = sdb.path.as_ref() {
                        if path == &detach_db {
                            let _abd = db;
                            db = None;
                        }
                    }
                }

                {
                    let db = mdb.lock();
                    match db {
                        Ok(mut msdb) => {
                            msdb.detach(&detach_db);
                            Packet::RespOk("Ok.".to_string())
                        }
                        Err(_) => Packet::RespError("get lock failed".to_string()),
                    }
                }
            }
            Packet::CmdRangeBegin(page_size) => {
                match db.as_ref() {
                    Some(sdb) => {
                        let mut tokens = vec![];
                        let it = sdb.this_db().iterator(IteratorMode::Start);
                        for rs in it.take(page_size as usize) {
                            if let Ok((k, v)) = rs {
                                tokens.push(k.to_vec());
                                tokens.push(v.to_vec());
                            } else {
                                // TODO add warning log
                            }
                        }
                        Packet::RespPairs(tokens)
                    }
                    None => Packet::RespError("no db selected".to_string()),
                }
            }
            Packet::CmdRangeEnd(page_size) => {
                match db.as_ref() {
                    Some(sdb) => {
                        let mut tokens = vec![];
                        let it = sdb.this_db().iterator(IteratorMode::End);
                        for rs in it.take(page_size as usize) {
                            if let Ok((k, v)) = rs {
                                tokens.push(k.to_vec());
                                tokens.push(v.to_vec());
                            } else {
                                // TODO add warning log
                            }
                        }
                        Packet::RespPairs(tokens)
                    }
                    None => Packet::RespError("no db selected".to_string()),
                }
            }
            Packet::CmdRangeFromAsc(page_size, key) => {
                match db.as_ref() {
                    Some(sdb) => {
                        let mut tokens = vec![];
                        let iter_mode = IteratorMode::From(key.as_slice(), Direction::Forward);
                        let it = sdb.this_db().iterator(iter_mode);
                        for rs in it.take(page_size as usize) {
                            if let Ok((k, v)) = rs {
                                tokens.push(k.to_vec());
                                tokens.push(v.to_vec());
                            } else {
                                // TODO add warning log
                            }
                        }
                        Packet::RespPairs(tokens)
                    }
                    None => Packet::RespError("no db selected".to_string()),
                }
            }
            Packet::CmdRangeFromAscEx(page_size, key) => {
                match db.as_ref() {
                    Some(sdb) => {
                        let mut tokens = vec![];
                        let iter_mode = IteratorMode::From(key.as_slice(), Direction::Forward);
                        let it = sdb.this_db().iterator(iter_mode);
                        for (idx, rs) in it.take(page_size as usize + 1).enumerate() {
                            if let Ok((k, v)) = rs {
                                let k_vec = k.to_vec();
                                if idx == 0 && k_vec == key {
                                    continue;
                                }
                                tokens.push(k_vec);
                                tokens.push(v.to_vec());
                            } else {
                                // TODO add warning log
                            }
                        }
                        Packet::RespPairs(tokens)
                    }
                    None => Packet::RespError("no db selected".to_string()),
                }
            }
            Packet::CmdRangeFromDesc(page_size, key) => {
                match db.as_ref() {
                    Some(sdb) => {
                        let mut tokens = vec![];
                        let iter_mode = IteratorMode::From(key.as_slice(), Direction::Reverse);
                        let it = sdb.this_db().iterator(iter_mode);
                        for rs in it.take(page_size as usize) {
                            if let Ok((k, v)) = rs {
                                tokens.push(k.to_vec());
                                tokens.push(v.to_vec());
                            } else {
                                // TODO add warning log
                            }
                        }
                        Packet::RespPairs(tokens)
                    }
                    None => Packet::RespError("no db selected".to_string()),
                }
            }
            Packet::CmdRangeFromDescEx(page_size, key) => {
                match db.as_ref() {
                    Some(sdb) => {
                        let mut tokens = vec![];
                        let iter_mode = IteratorMode::From(key.as_slice(), Direction::Reverse);
                        let it = sdb.this_db().iterator(iter_mode);
                        for (idx, rs) in it.take(page_size as usize + 1).enumerate() {
                            if let Ok((k, v)) = rs {
                                let k_vec = k.to_vec();
                                if idx == 0 && k_vec == key {
                                    continue;
                                }
                                tokens.push(k_vec);
                                tokens.push(v.to_vec());
                            } else {
                                // TODO add warning log
                            }
                        }
                        Packet::RespPairs(tokens)
                    }
                    None => Packet::RespError("no db selected".to_string()),
                }
            }
            _ => Packet::RespError("unknown command".to_string()),
        };
        rw.write_packet(&resp);
    }

    Ok(())
}
