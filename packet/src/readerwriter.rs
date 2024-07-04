use std::io::{Error, Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::packet;

pub struct PacketReaderWriter<T: Read + Write> {
    rw: T,
}

impl<T: Read + Write> PacketReaderWriter<T> {
    pub fn new(rw: T) -> Self {
        Self { rw }
    }

    pub fn read_packet(&mut self) -> Result<packet::Packet, Error> {
        let header = self.read_header()?;

        match header {
            packet::CMD_WRITE => {
                let pairs = self.read_size();
                let mut tokens = Vec::new();
                for _ in 0..pairs {
                    let token = self.read_token();
                    tokens.push(token);
                    let token = self.read_token();
                    tokens.push(token);
                }
                Ok(packet::Packet::CmdWrite(tokens))
            }
            packet::CMD_DELETE => {
                let key_count = self.read_size();
                let mut keys = Vec::new();
                for _ in 0..key_count {
                    let key = self.read_token();
                    keys.push(key);
                }
                Ok(packet::Packet::CmdDelete(keys))
            }
            packet::CMD_READ => {
                let key_count = self.read_size();
                let mut keys = Vec::new();
                for _ in 0..key_count {
                    let key = self.read_token();
                    keys.push(key);
                }
                Ok(packet::Packet::CmdRead(keys))
            }
            packet::CMD_USE => {
                let token = self.read_token();
                Ok(packet::Packet::CmdUse(token))
            }
            packet::CMD_CURRENT_DB => Ok(packet::Packet::CmdCurrentDB()),
            packet::CMD_LIST_DB => Ok(packet::Packet::CmdListDb()),
            packet::CMD_DETACH => {
                let token = self.read_token();
                Ok(packet::Packet::CmdDetach(token))
            }

            packet::CMD_RANGE_BEGIN => {
                let page_size = self.read_size();
                Ok(packet::Packet::CmdRangeBegin(page_size))
            }
            packet::CMD_RANGE_END => {
                let page_size = self.read_size();
                Ok(packet::Packet::CmdRangeEnd(page_size))
            }
            packet::CMD_RANGE_FROM_ASC => {
                let page_size = self.read_size();
                let token = self.read_token();
                Ok(packet::Packet::CmdRangeFromAsc(page_size, token))
            }
            packet::CMD_RANGE_FROM_ASC_EX => {
                let page_size = self.read_size();
                let token = self.read_token();
                Ok(packet::Packet::CmdRangeFromAscEx(page_size, token))
            }
            packet::CMD_RANGE_FROM_DESC => {
                let page_size = self.read_size();
                let token = self.read_token();
                Ok(packet::Packet::CmdRangeFromDesc(page_size, token))
            }
            packet::CMD_RANGE_FROM_DESC_EX => {
                let page_size = self.read_size();
                let token = self.read_token();
                Ok(packet::Packet::CmdRangeFromDescEx(page_size, token))
            }

            packet::RESP_OK => {
                let message = self.read_token();
                let message = String::from_utf8(message).unwrap();
                Ok(packet::Packet::RespOk(message))
            }
            packet::RESP_ERROR => {
                let message = self.read_token();
                let message = String::from_utf8(message).unwrap();
                Ok(packet::Packet::RespError(message))
            }
            packet::RESP_TOKEN => {
                let token = self.read_token();
                Ok(packet::Packet::RespToken(token))
            }
            packet::RESP_TOKENS => {
                let token_count = self.read_size();
                let mut tokens = Vec::new();
                for _ in 0..token_count {
                    let token = self.read_token();
                    tokens.push(token);
                }
                Ok(packet::Packet::RespTokens(tokens))
            }
            packet::RESP_PAIRS => {
                let pair_count = self.read_size();
                let mut pairs = Vec::new();
                for _ in 0..pair_count {
                    let token = self.read_token();
                    pairs.push(token);
                    let token = self.read_token();
                    pairs.push(token);
                }
                Ok(packet::Packet::RespPairs(pairs))
            }

            _ => {
                panic!("Unknown packet");
            }
        }
    }

    pub fn write_packet(&mut self, packet: &packet::Packet) {
        match packet {
            packet::Packet::CmdWrite(pairs) => {
                self.write_header(packet::CMD_WRITE);
                self.write_size((pairs.len() / 2) as u16);
                for token in pairs {
                    self.write_token(token);
                }
            }
            packet::Packet::CmdRead(keys) => {
                self.write_header(packet::CMD_READ);
                self.write_size(keys.len() as u16);
                for token in keys {
                    self.write_token(token);
                }
            }
            packet::Packet::CmdDelete(keys) => {
                self.write_header(packet::CMD_DELETE);
                self.write_size(keys.len() as u16);
                for token in keys {
                    self.write_token(token);
                }
            }
            packet::Packet::CmdUse(name) => {
                self.write_header(packet::CMD_USE);
                self.write_token(name);
            }
            packet::Packet::CmdCurrentDB() => {
                self.write_header(packet::CMD_CURRENT_DB);
            }
            packet::Packet::CmdListDb() => self.write_header(packet::CMD_LIST_DB),
            packet::Packet::CmdDetach(name) => {
                self.write_header(packet::CMD_DETACH);
                self.write_token(name);
            }

            packet::Packet::CmdRangeBegin(page_size) => {
                self.write_header(packet::CMD_RANGE_BEGIN);
                self.write_size(page_size.to_owned());
            }
            packet::Packet::CmdRangeEnd(page_size) => {
                self.write_header(packet::CMD_RANGE_END);
                self.write_size(page_size.to_owned());
            }
            packet::Packet::CmdRangeFromAsc(page_size, data) => {
                self.write_header(packet::CMD_RANGE_FROM_ASC);
                self.write_size(page_size.to_owned());
                self.write_token(data);
            }
            packet::Packet::CmdRangeFromAscEx(page_size, data) => {
                self.write_header(packet::CMD_RANGE_FROM_ASC_EX);
                self.write_size(page_size.to_owned());
                self.write_token(data);
            }
            packet::Packet::CmdRangeFromDesc(page_size, data) => {
                self.write_header(packet::CMD_RANGE_FROM_DESC);
                self.write_size(page_size.to_owned());
                self.write_token(data);
            }
            packet::Packet::CmdRangeFromDescEx(page_size, data) => {
                self.write_header(packet::CMD_RANGE_FROM_DESC_EX);
                self.write_size(page_size.to_owned());
                self.write_token(data);
            }

            packet::Packet::RespOk(message) => {
                self.write_header(packet::RESP_OK);
                self.write_token(message.as_bytes());
            }
            packet::Packet::RespError(message) => {
                self.write_header(packet::RESP_ERROR);
                self.write_token(message.as_bytes());
            }
            packet::Packet::RespToken(token) => {
                self.write_header(packet::RESP_TOKEN);
                self.write_token(token);
            }
            packet::Packet::RespTokens(tokens) => {
                self.write_header(packet::RESP_TOKENS);
                self.write_size(tokens.len() as u16);
                for token in tokens {
                    self.write_token(token);
                }
            }
            packet::Packet::RespPairs(pairs) => {
                self.write_header(packet::RESP_PAIRS);
                self.write_size((pairs.len() / 2) as u16);
                for token in pairs {
                    self.write_token(token);
                }
            }
        }

        self.flush();
    }

    fn read_header(&mut self) -> Result<u8, Error> {
        self.rw.read_u8()
    }

    fn write_header(&mut self, packet_type: u8) {
        self.rw.write_u8(packet_type).unwrap();
    }

    fn read_size(&mut self) -> u16 {
        self.rw.read_u16::<BigEndian>().unwrap()
    }

    fn write_size(&mut self, size: u16) {
        self.rw.write_u16::<BigEndian>(size).unwrap();
    }

    fn read_token(&mut self) -> Vec<u8> {
        let length = self.rw.read_u32::<BigEndian>().unwrap();
        if length == 0 {
            return Vec::new();
        }
        let mut key = vec![0u8; length as usize];
        self.rw.read_exact(&mut key).unwrap();
        key
    }

    fn write_token(&mut self, token: &[u8]) {
        self.rw.write_u32::<BigEndian>(token.len() as u32).unwrap();
        if token.len() == 0 {
            return;
        }
        self.rw.write_all(token).unwrap();
    }

    pub fn flush(&mut self) {
        self.rw.flush().unwrap();
    }
}
