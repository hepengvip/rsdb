use std::io::{Read, Result, Write};
use std::net::TcpListener;
use std::os::unix::net::UnixListener;
use std::sync::{Arc, Mutex};
use std::thread;

extern crate packet;
extern crate storage;

use packet::{Packet, PacketReaderWriter};
use storage::{Direction, IteratorMode, MultiDB};

use crate::errors::{ServerError, ServerResult};

pub struct Server {
    storage: Arc<Mutex<MultiDB>>,
    address: Option<String>,
    unix_address: Option<String>,
    storage_dir: String,
}

impl Server {
    pub fn new(addr: Option<String>, root: &str, unix_addr: Option<String>) -> ServerResult<Self> {
        let server = Server {
            storage: Arc::new(Mutex::new(MultiDB::new(root))),
            address: addr,
            unix_address: unix_addr,
            storage_dir: root.to_string(),
        };

        Ok(server)
    }

    pub fn listen_and_serve(&self) -> Result<()> {
        // Build a server
        println!("    > Listening at tcp  address {:?}", &self.address);
        println!("    > Listening at unix address {:?}", &self.unix_address);
        println!("    > Storage: {}\n", &self.storage_dir);

        // create a new thread to handle unix domain socket
        if let Some(addr) = &self.unix_address {
            let unix_sock = UnixListener::bind(addr)?;
            let storage = self.storage.clone();
            thread::spawn(move || {
                for stream in unix_sock.incoming() {
                    match stream {
                        Err(e) => eprintln!("error: {}", e),
                        Ok(stream) => {
                            let db_copy = storage.clone();
                            thread::spawn(move || {
                                handler(stream, "<local unix client>", db_copy).unwrap_or_else(
                                    |error| {
                                        eprintln!("{:?}", error);
                                    },
                                );
                            });
                        }
                    }
                }
            });
        }

        // handle tcp incoming connections
        if let Some(addr) = &self.address {
            let listener = TcpListener::bind(addr)?;
            for streams in listener.incoming() {
                match streams {
                    Err(e) => {
                        eprintln!("error: {}", e)
                    }
                    Ok(stream) => {
                        let peer_name = format!("{}", stream.peer_addr().unwrap());
                        stream.set_nodelay(true).unwrap();
                        let db_copy = self.storage.clone();
                        thread::spawn(move || {
                            handler(stream, &peer_name, db_copy).unwrap_or_else(|error| {
                                eprintln!("{:?}", error);
                            });
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

fn handler<T>(stream: T, peer_name: &str, mdb: Arc<Mutex<MultiDB>>) -> ServerResult<()>
where
    T: Read + Write,
{
    println!("Connection from {}", peer_name);

    let mut rw = PacketReaderWriter::new(stream);
    let mut db: Option<Arc<storage::Storage>> = None;
    loop {
        // let packet = rw.read_packet();
        let packet = rw.read_packet();
        if packet.is_err() {
            println!("Connection closed by client: <{peer_name}>");
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
        rw.write_packet(&resp)?;
    }

    Ok(())
}
