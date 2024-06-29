use std::io::{Error, Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

// length constants
pub const CMD_LENGTH: usize = 1;
pub const LEN_LENGTH: usize = 2;
pub const TOKEN_LENGTH: usize = 4;

// commands
const CMD_WRITE: u8 = 0x01;
const CMD_DELETE: u8 = 0x02;
const CMD_READ: u8 = 0x03;
const CMD_USE: u8 = 0x04;

// responses
const RESP_OK: u8 = 0x55;
const RESP_ERROR: u8 = 0x56;
const RESP_TOKEN: u8 = 0x57;
const RESP_TOKENS: u8 = 0x58;
const RESP_PAIRS: u8 = 0x59;

#[derive(Debug, PartialEq)]
pub enum Packet {
    CmdWrite(Vec<Vec<u8>>),
    CmdRead(Vec<Vec<u8>>),
    CmdDelete(Vec<Vec<u8>>),
    CmdUse(Vec<u8>),
    RespOk(String),
    RespError(String),
    RespToken(Vec<u8>),
    RespTokens(Vec<Vec<u8>>),
    RespPairs(Vec<Vec<u8>>),
}

pub struct PacketWriter<T: Write> {
    writer: T,
}

impl<T: Write> PacketWriter<T> {
    pub fn new(writer: T) -> Self {
        Self { writer }
    }

    pub fn write_packet(&mut self, packet: &Packet) {
        match packet {
            Packet::CmdWrite(pairs) => {
                self.write_header(CMD_WRITE);
                self.write_size((pairs.len() / 2) as u16);
                for token in pairs {
                    self.write_token(token);
                }
            }
            Packet::CmdRead(keys) => {
                self.write_header(CMD_READ);
                self.write_size(keys.len() as u16);
                for token in keys {
                    self.write_token(token);
                }
            }
            Packet::CmdDelete(keys) => {
                self.write_header(CMD_DELETE);
                self.write_size(keys.len() as u16);
                for token in keys {
                    self.write_token(token);
                }
            }
            Packet::CmdUse(name) => {
                self.write_header(CMD_USE);
                self.write_token(name);
            }

            Packet::RespOk(message) => {
                self.write_header(RESP_OK);
                self.write_token(message.as_bytes());
            }
            Packet::RespError(message) => {
                self.write_header(RESP_ERROR);
                self.write_token(message.as_bytes());
            }
            Packet::RespToken(token) => {
                self.write_header(RESP_TOKEN);
                self.write_token(token);
            }
            Packet::RespTokens(tokens) => {
                self.write_header(RESP_TOKENS);
                self.write_size(tokens.len() as u16);
                for token in tokens {
                    self.write_token(token);
                }
            }
            Packet::RespPairs(pairs) => {
                self.write_header(RESP_PAIRS);
                self.write_size((pairs.len() / 2) as u16);
                for token in pairs {
                    self.write_token(token);
                }
            }
        }
    }

    fn write_header(&mut self, packet_type: u8) {
        self.writer.write_u8(packet_type).unwrap();
    }

    fn write_size(&mut self, size: u16) {
        self.writer.write_u16::<BigEndian>(size).unwrap();
    }

    fn write_token(&mut self, token: &[u8]) {
        self.writer
            .write_u32::<BigEndian>(token.len() as u32)
            .unwrap();
        if token.len() == 0 {
            return;
        }
        self.writer.write_all(token).unwrap();
    }
}

#[cfg(test)]
mod test_packet_writer {

    use std::vec;

    use super::*;

    #[test]
    fn test_cmd_write() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        packer.write_packet(&Packet::CmdWrite(vec![b"key".to_vec(), b"val".to_vec()]));
        assert_eq!(
            writer,
            [CMD_WRITE, 0, 1, 0, 0, 0, 3, b'k', b'e', b'y', 0, 0, 0, 3, b'v', b'a', b'l']
        );
    }

    #[test]
    fn test_cmd_read() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        packer.write_packet(&&Packet::CmdRead(vec![b"key".to_vec(), b"val".to_vec()]));
        assert_eq!(
            writer,
            [CMD_READ, 0, 2, 0, 0, 0, 3, b'k', b'e', b'y', 0, 0, 0, 3, b'v', b'a', b'l']
        );
    }

    #[test]
    fn test_cmd_delete() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        packer.write_packet(&&Packet::CmdDelete(vec![b"key".to_vec(), b"val".to_vec()]));
        assert_eq!(
            writer,
            [CMD_DELETE, 0, 2, 0, 0, 0, 3, b'k', b'e', b'y', 0, 0, 0, 3, b'v', b'a', b'l']
        );
    }

    #[test]
    fn test_cmd_use() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = Packet::CmdUse(b"world".to_vec());
        packer.write_packet(&packet);
        assert_eq!(writer, [CMD_USE, 0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd'],);
    }

    #[test]
    fn test_resp_ok() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = Packet::RespOk("world".to_string());
        packer.write_packet(&packet);
        assert_eq!(writer, [RESP_OK, 0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd'],);
    }

    #[test]
    fn test_resp_error() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = Packet::RespError("world".to_string());
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [RESP_ERROR, 0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd'],
        );
    }

    #[test]
    fn test_resp_token() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = Packet::RespToken(b"world".to_vec());
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [RESP_TOKEN, 0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd'],
        );
    }

    #[test]
    fn test_resp_tokens() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = Packet::RespTokens(vec![
            "hello".as_bytes().to_vec(),
            "world".as_bytes().to_vec(),
            vec![],
            "rust".as_bytes().to_vec(),
        ]);
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [
                RESP_TOKENS,
                0,
                4,
                0,
                0,
                0,
                5,
                b'h',
                b'e',
                b'l',
                b'l',
                b'o',
                0,
                0,
                0,
                5,
                b'w',
                b'o',
                b'r',
                b'l',
                b'd',
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                4,
                b'r',
                b'u',
                b's',
                b't',
            ],
        );
    }

    #[test]
    fn test_resp_pairs() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = Packet::RespPairs(vec![
            "hello".as_bytes().to_vec(),
            "world".as_bytes().to_vec(),
        ]);
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [
                RESP_PAIRS, 0, 1, 0, 0, 0, 5, b'h', b'e', b'l', b'l', b'o', 0, 0, 0, 5, b'w', b'o',
                b'r', b'l', b'd',
            ],
        );
    }
}

pub struct PacketReader<T: Read> {
    reader: T,
}

impl<T: Read> PacketReader<T> {
    pub fn new(reader: T) -> Self {
        Self { reader }
    }

    pub fn read_packet(&mut self) -> Packet {
        let header = self.read_header();

        match header {
            CMD_WRITE => {
                let pairs = self.read_size();
                let mut tokens = Vec::new();
                for _ in 0..pairs {
                    let token = self.read_token();
                    tokens.push(token);
                    let token = self.read_token();
                    tokens.push(token);
                }
                return Packet::CmdWrite(tokens);
            }
            CMD_DELETE => {
                let key_count = self.read_size();
                let mut keys = Vec::new();
                for _ in 0..key_count {
                    let key = self.read_token();
                    keys.push(key);
                }
                return Packet::CmdDelete(keys);
            }
            CMD_READ => {
                let key_count = self.read_size();
                let mut keys = Vec::new();
                for _ in 0..key_count {
                    let key = self.read_token();
                    keys.push(key);
                }
                return Packet::CmdRead(keys);
            }
            CMD_USE => {
                let token = self.read_token();
                return Packet::CmdUse(token);
            }

            RESP_OK => {
                let message = self.read_token();
                let message = String::from_utf8(message).unwrap();
                return Packet::RespOk(message);
            }
            RESP_ERROR => {
                let message = self.read_token();
                let message = String::from_utf8(message).unwrap();
                return Packet::RespError(message);
            }
            RESP_TOKEN => {
                let token = self.read_token();
                return Packet::RespToken(token);
            }
            RESP_TOKENS => {
                let token_count = self.read_size();
                let mut tokens = Vec::new();
                for _ in 0..token_count {
                    let token = self.read_token();
                    tokens.push(token);
                }
                return Packet::RespTokens(tokens);
            }
            RESP_PAIRS => {
                let pair_count = self.read_size();
                let mut pairs = Vec::new();
                for _ in 0..pair_count {
                    let token = self.read_token();
                    pairs.push(token);
                    let token = self.read_token();
                    pairs.push(token);
                }
                return Packet::RespPairs(pairs);
            }

            _ => {
                panic!("Unknown packet");
            }
        }
    }

    fn read_header(&mut self) -> u8 {
        self.reader.read_u8().unwrap()
    }

    fn read_size(&mut self) -> u16 {
        self.reader.read_u16::<BigEndian>().unwrap()
    }

    fn read_token(&mut self) -> Vec<u8> {
        let length = self.reader.read_u32::<BigEndian>().unwrap();
        if length == 0 {
            return Vec::new();
        }
        let mut key = vec![0u8; length as usize];
        self.reader.read_exact(&mut key).unwrap();
        key
    }
}

#[cfg(test)]
mod test_packet_reader {
    use super::*;

    #[test]
    fn test_cmd_write() {
        let bytes = vec![
            CMD_WRITE, // packet type id
            0, 2, // pair count
            0, 0, 0, 3, b'k', b'e', b'y', // key 1
            0, 0, 0, 3, b'v', b'a', b'l', // value 1
            0, 0, 0, 5, b'h', b'e', b'l', b'l', b'o', // key 2
            0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd', // value 2
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(
            packet,
            Packet::CmdWrite(vec![
                b"key".to_vec(),
                b"val".to_vec(),
                b"hello".to_vec(),
                b"world".to_vec()
            ]),
        );
    }

    #[test]
    fn test_cmd_delete() {
        let bytes = vec![
            CMD_DELETE, // packet type id
            0, 2, // pair count
            0, 0, 0, 3, b'k', b'e', b'y', // key 1
            0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd', // value 2
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(
            packet,
            Packet::CmdDelete(vec![b"key".to_vec(), b"world".to_vec()]),
        );
    }

    #[test]
    fn test_cmd_read() {
        let bytes = vec![
            CMD_READ, // packet type id
            0, 2, // pair count
            0, 0, 0, 3, b'k', b'e', b'y', // key 1
            0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd', // value 2
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(
            packet,
            Packet::CmdRead(vec![b"key".to_vec(), b"world".to_vec()]),
        );
    }

    #[test]
    fn test_cmd_use() {
        let bytes = vec![
            CMD_USE, // packet type id
            0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd', // token
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(packet, Packet::CmdUse(b"world".to_vec()),);
    }

    #[test]
    fn test_resp_ok() {
        let bytes = vec![
            RESP_OK, // packet type id
            0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd', // message
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(packet, Packet::RespOk("world".to_string()),);
    }

    #[test]
    fn test_resp_error() {
        let bytes = vec![
            RESP_ERROR, // packet type id
            0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd', // message
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(packet, Packet::RespError("world".to_string()),);
    }

    #[test]
    fn test_resp_token() {
        let bytes = vec![
            RESP_TOKEN, // packet type id
            0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd', // token
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(packet, Packet::RespToken(b"world".to_vec()),);
    }

    #[test]
    fn test_resp_tokens() {
        let bytes = vec![
            RESP_TOKENS, // packet type id
            0,
            5, // token count
            0,
            0,
            0,
            3,
            b'k',
            b'e',
            b'y', // token 1
            0,
            0,
            0x01,
            0x1c,
            b't',
            b'h',
            b'i',
            b's',
            b' ',
            b'i',
            b's',
            b' ',
            b'a',
            b' ',
            b's',
            b'u',
            b'c',
            b'c',
            b'e',
            b's',
            b's',
            b'f',
            b'u',
            b'l',
            b' ',
            b'm',
            b'e',
            b's',
            b's',
            b'a',
            b'g',
            b'e',
            b' ',
            b'a',
            b'b',
            b'o',
            b'u',
            b't',
            b' ',
            b't',
            b'h',
            b'e',
            b' ',
            b's',
            b'y',
            b's',
            b't',
            b'e',
            b'm',
            b' ',
            b'j',
            b's',
            b'd',
            b'f',
            b'j',
            b'i',
            b'o',
            b'e',
            b'w',
            b'f',
            b'j',
            b'l',
            b'k',
            b's',
            b'f',
            b'j',
            b'i',
            b'e',
            b'w',
            b'f',
            b'w',
            b'o',
            b'f',
            b'j',
            b'd',
            b's',
            b'j',
            b'l',
            b's',
            b'k',
            b'f',
            b'j',
            b'k',
            b'l',
            b's',
            b'f',
            b'j',
            b'l',
            b'd',
            b'k',
            b's',
            b'f',
            b'j',
            b'l',
            b'd',
            b'k',
            b's',
            b'f',
            b'j',
            b'k',
            b'l',
            b's',
            b'f',
            b'j',
            b'd',
            b'k',
            b'l',
            b's',
            b'f',
            b'j',
            b'd',
            b'k',
            b'l',
            b's',
            b'f',
            b'j',
            b'k',
            b'd',
            b'f',
            b'j',
            b'd',
            b'k',
            b'f',
            b'j',
            b'd',
            b'k',
            b'f',
            b'j',
            b'd',
            b'k',
            b'l',
            b's',
            b'f',
            b'j',
            b'd',
            b'k',
            b's',
            b'f',
            b'j',
            b'd',
            b'k',
            b's',
            b'f',
            b'j',
            b'd',
            b's',
            b'k',
            b'f',
            b'j',
            b'd',
            b's',
            b'k',
            b'f',
            b'j',
            b'd',
            b's',
            b'f',
            b'j',
            b'd',
            b's',
            b'f',
            b'd',
            b's',
            b'f',
            b'd',
            b'f',
            b'd',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'f',
            b'e',
            b'f',
            b'e',
            b'f',
            b'e',
            b'w',
            b'f',
            b'e',
            b'f',
            b'a', // token 2
            0,
            0,
            0,
            5,
            b'w',
            b'o',
            b'r',
            b'l',
            b'd', // token 3
            0,
            0,
            0,
            0, // token 4
            0,
            0,
            0,
            4,
            b'r',
            b'u',
            b's',
            b't', // token 5
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(
            packet,
            Packet::RespTokens(vec![
                b"key".to_vec(),
                b"this is a successful message about the system jsdfjioewfjlksfjiewfwofjdsjlskfjklsfjldksfjldksfjklsfjdklsfjdklsfjkdfjdkfjdkfjdklsfjdksfjdksfjdskfjdskfjdsfjdsfdsfdfdfffffffffffffffffffffffffffffffffffaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaadddddddddddddddddddddddddddddddddddddfefefewfefa".to_vec(),
                b"world".to_vec(),
                vec![],
                b"rust".to_vec()
            ]),
        );
    }

    #[test]
    fn test_resp_pairs() {
        let bytes = vec![
            RESP_PAIRS, // packet type id
            0, 2, // pair count
            0, 0, 0, 3, b'k', b'e', b'y', // key 1
            0, 0, 0, 5, b'v', b'a', b'l', b'u', b'e', // value 1
            0, 0, 0, 5, b'h', b'e', b'l', b'l', b'o', // key 2
            0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd', // value 2
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(
            packet,
            Packet::RespPairs(vec![
                b"key".to_vec(),
                b"value".to_vec(),
                b"hello".to_vec(),
                b"world".to_vec()
            ]),
        );
    }
}

pub struct PacketReaderWriter<T: Read + Write> {
    rw: T,
}

impl<T: Read + Write> PacketReaderWriter<T> {
    pub fn new(rw: T) -> Self {
        Self { rw }
    }

    pub fn read_packet(&mut self) -> Result<Packet, Error> {
        let header = self.read_header()?;

        match header {
            CMD_WRITE => {
                let pairs = self.read_size();
                let mut tokens = Vec::new();
                for _ in 0..pairs {
                    let token = self.read_token();
                    tokens.push(token);
                    let token = self.read_token();
                    tokens.push(token);
                }
                return Ok(Packet::CmdWrite(tokens));
            }
            CMD_DELETE => {
                let key_count = self.read_size();
                let mut keys = Vec::new();
                for _ in 0..key_count {
                    let key = self.read_token();
                    keys.push(key);
                }
                return Ok(Packet::CmdDelete(keys));
            }
            CMD_READ => {
                let key_count = self.read_size();
                let mut keys = Vec::new();
                for _ in 0..key_count {
                    let key = self.read_token();
                    keys.push(key);
                }
                return Ok(Packet::CmdRead(keys));
            }
            CMD_USE => {
                let token = self.read_token();
                return Ok(Packet::CmdUse(token));
            }

            RESP_OK => {
                let message = self.read_token();
                let message = String::from_utf8(message).unwrap();
                return Ok(Packet::RespOk(message));
            }
            RESP_ERROR => {
                let message = self.read_token();
                let message = String::from_utf8(message).unwrap();
                return Ok(Packet::RespError(message));
            }
            RESP_TOKEN => {
                let token = self.read_token();
                return Ok(Packet::RespToken(token));
            }
            RESP_TOKENS => {
                let token_count = self.read_size();
                let mut tokens = Vec::new();
                for _ in 0..token_count {
                    let token = self.read_token();
                    tokens.push(token);
                }
                return Ok(Packet::RespTokens(tokens));
            }
            RESP_PAIRS => {
                let pair_count = self.read_size();
                let mut pairs = Vec::new();
                for _ in 0..pair_count {
                    let token = self.read_token();
                    pairs.push(token);
                    let token = self.read_token();
                    pairs.push(token);
                }
                return Ok(Packet::RespPairs(pairs));
            }

            _ => {
                panic!("Unknown packet");
            }
        }
    }

    pub fn write_packet(&mut self, packet: &Packet) {
        match packet {
            Packet::CmdWrite(pairs) => {
                self.write_header(CMD_WRITE);
                self.write_size((pairs.len() / 2) as u16);
                for token in pairs {
                    self.write_token(token);
                }
            }
            Packet::CmdRead(keys) => {
                self.write_header(CMD_READ);
                self.write_size(keys.len() as u16);
                for token in keys {
                    self.write_token(token);
                }
            }
            Packet::CmdDelete(keys) => {
                self.write_header(CMD_DELETE);
                self.write_size(keys.len() as u16);
                for token in keys {
                    self.write_token(token);
                }
            }
            Packet::CmdUse(name) => {
                self.write_header(CMD_USE);
                self.write_token(name);
            }

            Packet::RespOk(message) => {
                self.write_header(RESP_OK);
                self.write_token(message.as_bytes());
            }
            Packet::RespError(message) => {
                self.write_header(RESP_ERROR);
                self.write_token(message.as_bytes());
            }
            Packet::RespToken(token) => {
                self.write_header(RESP_TOKEN);
                self.write_token(token);
            }
            Packet::RespTokens(tokens) => {
                self.write_header(RESP_TOKENS);
                self.write_size(tokens.len() as u16);
                for token in tokens {
                    self.write_token(token);
                }
            }
            Packet::RespPairs(pairs) => {
                self.write_header(RESP_PAIRS);
                self.write_size((pairs.len() / 2) as u16);
                for token in pairs {
                    self.write_token(token);
                }
            }
        }
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
}

#[cfg(test)]
mod test_packet_readerwriter {

    use super::*;

    #[test]
    fn test_w_cmd_write() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        packer.write_packet(&Packet::CmdWrite(vec![b"key".to_vec(), b"val".to_vec()]));
        assert_eq!(
            writer,
            [CMD_WRITE, 0, 1, 0, 0, 0, 3, b'k', b'e', b'y', 0, 0, 0, 3, b'v', b'a', b'l']
        );
    }

    #[test]
    fn test_w_cmd_read() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        packer.write_packet(&&Packet::CmdRead(vec![b"key".to_vec(), b"val".to_vec()]));
        assert_eq!(
            writer,
            [CMD_READ, 0, 2, 0, 0, 0, 3, b'k', b'e', b'y', 0, 0, 0, 3, b'v', b'a', b'l']
        );
    }

    #[test]
    fn test_w_cmd_delete() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        packer.write_packet(&&Packet::CmdDelete(vec![b"key".to_vec(), b"val".to_vec()]));
        assert_eq!(
            writer,
            [CMD_DELETE, 0, 2, 0, 0, 0, 3, b'k', b'e', b'y', 0, 0, 0, 3, b'v', b'a', b'l']
        );
    }

    #[test]
    fn test_w_resp_ok() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = Packet::RespOk("world".to_string());
        packer.write_packet(&packet);
        assert_eq!(writer, [RESP_OK, 0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd'],);
    }

    #[test]
    fn test_w_resp_error() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = Packet::RespError("world".to_string());
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [RESP_ERROR, 0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd'],
        );
    }

    #[test]
    fn test_w_resp_token() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = Packet::RespToken(b"world".to_vec());
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [RESP_TOKEN, 0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd'],
        );
    }

    #[test]
    fn test_w_resp_tokens() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = Packet::RespTokens(vec![
            "hello".as_bytes().to_vec(),
            "world".as_bytes().to_vec(),
        ]);
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [
                RESP_TOKENS,
                0,
                2,
                0,
                0,
                0,
                5,
                b'h',
                b'e',
                b'l',
                b'l',
                b'o',
                0,
                0,
                0,
                5,
                b'w',
                b'o',
                b'r',
                b'l',
                b'd',
            ],
        );
    }

    #[test]
    fn test_w_resp_pairs() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = Packet::RespPairs(vec![
            "hello".as_bytes().to_vec(),
            "world".as_bytes().to_vec(),
        ]);
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [
                RESP_PAIRS, 0, 1, 0, 0, 0, 5, b'h', b'e', b'l', b'l', b'o', 0, 0, 0, 5, b'w', b'o',
                b'r', b'l', b'd',
            ],
        );
    }

    #[test]
    fn test_r_cmd_write() {
        let bytes = vec![
            CMD_WRITE, // packet type id
            0, 2, // pair count
            0, 0, 0, 3, b'k', b'e', b'y', // key 1
            0, 0, 0, 3, b'v', b'a', b'l', // value 1
            0, 0, 0, 5, b'h', b'e', b'l', b'l', b'o', // key 2
            0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd', // value 2
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(
            packet,
            Packet::CmdWrite(vec![
                b"key".to_vec(),
                b"val".to_vec(),
                b"hello".to_vec(),
                b"world".to_vec()
            ]),
        );
    }

    #[test]
    fn test_r_cmd_delete() {
        let bytes = vec![
            CMD_DELETE, // packet type id
            0, 2, // pair count
            0, 0, 0, 3, b'k', b'e', b'y', // key 1
            0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd', // value 2
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(
            packet,
            Packet::CmdDelete(vec![b"key".to_vec(), b"world".to_vec()]),
        );
    }

    #[test]
    fn test_r_cmd_read() {
        let bytes = vec![
            CMD_READ, // packet type id
            0, 2, // pair count
            0, 0, 0, 3, b'k', b'e', b'y', // key 1
            0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd', // value 2
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(
            packet,
            Packet::CmdRead(vec![b"key".to_vec(), b"world".to_vec()]),
        );
    }

    #[test]
    fn test_r_resp_ok() {
        let bytes = vec![
            RESP_OK, // packet type id
            0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd', // message
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(packet, Packet::RespOk("world".to_string()),);
    }

    #[test]
    fn test_r_resp_error() {
        let bytes = vec![
            RESP_ERROR, // packet type id
            0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd', // message
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(packet, Packet::RespError("world".to_string()),);
    }

    #[test]
    fn test_r_resp_token() {
        let bytes = vec![
            RESP_TOKEN, // packet type id
            0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd', // token
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(packet, Packet::RespToken(b"world".to_vec()),);
    }

    #[test]
    fn test_r_resp_tokens() {
        let bytes = vec![
            RESP_TOKENS, // packet type id
            0,
            3, // token count
            0,
            0,
            0,
            3,
            b'k',
            b'e',
            b'y', // token 1
            0,
            0,
            0x01,
            0x1c,
            b't',
            b'h',
            b'i',
            b's',
            b' ',
            b'i',
            b's',
            b' ',
            b'a',
            b' ',
            b's',
            b'u',
            b'c',
            b'c',
            b'e',
            b's',
            b's',
            b'f',
            b'u',
            b'l',
            b' ',
            b'm',
            b'e',
            b's',
            b's',
            b'a',
            b'g',
            b'e',
            b' ',
            b'a',
            b'b',
            b'o',
            b'u',
            b't',
            b' ',
            b't',
            b'h',
            b'e',
            b' ',
            b's',
            b'y',
            b's',
            b't',
            b'e',
            b'm',
            b' ',
            b'j',
            b's',
            b'd',
            b'f',
            b'j',
            b'i',
            b'o',
            b'e',
            b'w',
            b'f',
            b'j',
            b'l',
            b'k',
            b's',
            b'f',
            b'j',
            b'i',
            b'e',
            b'w',
            b'f',
            b'w',
            b'o',
            b'f',
            b'j',
            b'd',
            b's',
            b'j',
            b'l',
            b's',
            b'k',
            b'f',
            b'j',
            b'k',
            b'l',
            b's',
            b'f',
            b'j',
            b'l',
            b'd',
            b'k',
            b's',
            b'f',
            b'j',
            b'l',
            b'd',
            b'k',
            b's',
            b'f',
            b'j',
            b'k',
            b'l',
            b's',
            b'f',
            b'j',
            b'd',
            b'k',
            b'l',
            b's',
            b'f',
            b'j',
            b'd',
            b'k',
            b'l',
            b's',
            b'f',
            b'j',
            b'k',
            b'd',
            b'f',
            b'j',
            b'd',
            b'k',
            b'f',
            b'j',
            b'd',
            b'k',
            b'f',
            b'j',
            b'd',
            b'k',
            b'l',
            b's',
            b'f',
            b'j',
            b'd',
            b'k',
            b's',
            b'f',
            b'j',
            b'd',
            b'k',
            b's',
            b'f',
            b'j',
            b'd',
            b's',
            b'k',
            b'f',
            b'j',
            b'd',
            b's',
            b'k',
            b'f',
            b'j',
            b'd',
            b's',
            b'f',
            b'j',
            b'd',
            b's',
            b'f',
            b'd',
            b's',
            b'f',
            b'd',
            b'f',
            b'd',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'f',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'a',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'd',
            b'f',
            b'e',
            b'f',
            b'e',
            b'f',
            b'e',
            b'w',
            b'f',
            b'e',
            b'f',
            b'a', // token 2
            0,
            0,
            0,
            5,
            b'w',
            b'o',
            b'r',
            b'l',
            b'd', // token 3
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(
            packet,
            Packet::RespTokens(vec![
                b"key".to_vec(),
                b"this is a successful message about the system jsdfjioewfjlksfjiewfwofjdsjlskfjklsfjldksfjldksfjklsfjdklsfjdklsfjkdfjdkfjdkfjdklsfjdksfjdksfjdskfjdskfjdsfjdsfdsfdfdfffffffffffffffffffffffffffffffffffaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaadddddddddddddddddddddddddddddddddddddfefefewfefa".to_vec(),
                b"world".to_vec(),
            ]),
        );
    }

    #[test]
    fn test_r_resp_pairs() {
        let bytes = vec![
            RESP_PAIRS, // packet type id
            0, 2, // pair count
            0, 0, 0, 3, b'k', b'e', b'y', // key 1
            0, 0, 0, 5, b'v', b'a', b'l', b'u', b'e', // value 1
            0, 0, 0, 5, b'h', b'e', b'l', b'l', b'o', // key 2
            0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd', // value 2
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(
            packet,
            Packet::RespPairs(vec![
                b"key".to_vec(),
                b"value".to_vec(),
                b"hello".to_vec(),
                b"world".to_vec()
            ]),
        );
    }
}
