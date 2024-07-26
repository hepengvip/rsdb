use std::net::TcpStream;

use packet::{Packet, PacketReaderWriter};

extern crate storage;
pub use storage::{Direction, IteratorMode};

mod errors;
pub use errors::{RsDBError, RsDBResult};

extern crate packet;

pub struct RsDBClient {
    db_name: Option<String>,
    rw: Option<PacketReaderWriter<TcpStream>>,
}

impl RsDBClient {
    pub fn new() -> Self {
        Self {
            db_name: None,
            rw: None,
        }
    }

    pub fn connect(&mut self, addr: &str) -> RsDBResult<()> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nodelay(true)?;
        self.rw = Some(PacketReaderWriter::new(stream));
        Ok(())
    }

    pub fn set(&mut self, key: &[u8], value: &[u8]) -> RsDBResult<()> {
        self.check_db()?;
        let mut bytes_parts = vec![];
        bytes_parts.push(key.to_vec());
        bytes_parts.push(value.to_vec());
        let packet = Packet::CmdWrite(bytes_parts);
        self.send_request(&packet)?;
        let res = self.read_resp()?;
        if let Packet::RespError(ref msg) = res {
            return Err(RsDBError::RespError(msg.to_string()));
        }

        Ok(())
    }

    pub fn get(&mut self, key: &[u8]) -> RsDBResult<Option<Vec<u8>>> {
        self.check_db()?;
        let packet = Packet::CmdRead(vec![key.to_owned()]);
        self.send_request(&packet)?;
        let resp = self.read_resp()?;
        match resp {
            Packet::RespTokens(vals) => {
                if vals.len() != 1 {
                    return Err(RsDBError::RespError("invalid response".to_string()));
                }
                if vals[0].len() == 0 {
                    return Ok(None);
                }
                for val in vals {
                    return Ok(Some(val));
                }
                panic!("Should be here")
            }
            _ => Err(RsDBError::RespError("invalid response".to_string())),
        }
    }

    pub fn delete(&mut self, key: &[u8]) -> RsDBResult<()> {
        self.check_db()?;
        let packet = Packet::CmdDelete(vec![key.to_owned()]);
        self.send_request(&packet)?;
        let resp = self.read_resp()?;
        match resp {
            Packet::RespOk(_msg) => Ok(()),
            _ => Err(RsDBError::RespError("invalid response".to_string())),
        }
    }

    pub fn use_db(&mut self, name: &str) -> RsDBResult<()> {
        let packet = Packet::CmdUse(name.as_bytes().to_owned());
        self.send_request(&packet)?;
        let resp = self.read_resp()?;
        match resp {
            Packet::RespOk(_msg) => {
                self.db_name = Some(name.to_string());
                Ok(())
            }
            _ => Err(RsDBError::RespError("invalid response".to_string())),
        }
    }

    pub fn detach_db(&mut self, name: &str) -> RsDBResult<()> {
        let packet = Packet::CmdDetach(name.as_bytes().to_owned());
        self.send_request(&packet)?;
        let resp = self.read_resp()?;
        match resp {
            Packet::RespOk(_msg) => {
                if let Some(ref db_name) = self.db_name {
                    if db_name == name {
                        self.db_name = None;
                    }
                }
                Ok(())
            }
            _ => Err(RsDBError::RespError("invalid response".to_string())),
        }
    }

    pub fn get_db_name(&self) -> &Option<String> {
        &self.db_name
    }

    pub fn get_current_db(&mut self) -> RsDBResult<Option<String>> {
        let packet = Packet::CmdCurrentDB();
        self.send_request(&packet)?;
        let resp = self.read_resp()?;
        match resp {
            Packet::RespToken(data) => match data.len() {
                0 => Ok(None),
                _ => {
                    let msg = String::from_utf8(data)?;
                    self.db_name = Some(msg.clone());
                    Ok(Some(msg))
                }
            },
            _ => Err(RsDBError::RespError("invalid response".to_string())),
        }
    }

    pub fn list_db(&mut self) -> RsDBResult<Vec<String>> {
        let packet = Packet::CmdListDb();
        self.send_request(&packet)?;
        let resp = self.read_resp()?;

        match resp {
            Packet::RespTokens(tokens) => {
                let db_names = tokens
                    .iter()
                    .map(|ele| String::from_utf8(ele.to_owned()).unwrap())
                    .collect();
                Ok(db_names)
            }
            _ => Err(RsDBError::RespError("invalid response".to_string())),
        }
    }

    pub fn range(
        &mut self,
        iter_mode: IteratorMode,
        page_size: u16,
        exclude_current: bool,
    ) -> RsDBResult<Vec<(Vec<u8>, Vec<u8>)>> {
        self.check_db()?;

        let packet = match iter_mode {
            IteratorMode::Start => Packet::CmdRangeBegin(page_size),
            IteratorMode::End => Packet::CmdRangeEnd(page_size),
            IteratorMode::From(key, direction) => match (direction, exclude_current) {
                (Direction::Forward, false) => Packet::CmdRangeFromAsc(page_size, key.to_vec()),
                (Direction::Forward, true) => Packet::CmdRangeFromAscEx(page_size, key.to_vec()),
                (Direction::Reverse, false) => Packet::CmdRangeFromDesc(page_size, key.to_vec()),
                (Direction::Reverse, true) => Packet::CmdRangeFromDescEx(page_size, key.to_vec()),
            },
        };
        self.send_request(&packet)?;
        let resp = self.read_resp()?;

        match resp {
            Packet::RespPairs(mut tokens) => {
                let mut pairs = vec![];
                let pair_count = tokens.len() / 2;
                for _idx in 0..pair_count {
                    let v = tokens.pop().ok_or(RsDBError::EmptyToken)?;
                    let k = tokens.pop().ok_or(RsDBError::EmptyToken)?;
                    pairs.push((k, v))
                }
                pairs.reverse();
                Ok(pairs)
            }
            _ => Err(RsDBError::RespError("invalid response".to_string())),
        }
    }

    fn read_resp(&mut self) -> RsDBResult<Packet> {
        if let Some(ref mut rw) = self.rw {
            let rs = rw.read_packet()?;
            return Ok(rs);
        }
        Err(RsDBError::NotConnect)
    }

    fn send_request(&mut self, packet: &Packet) -> RsDBResult<()> {
        if let Some(ref mut rw) = self.rw {
            rw.write_packet(packet)?;
            rw.flush()?;
            return Ok(());
        }
        Err(RsDBError::NotConnect)
    }

    fn check_db(&self) -> RsDBResult<()> {
        if let None = self.db_name {
            return Err(RsDBError::NoDbSelected);
        }
        return Ok(());
    }
}
