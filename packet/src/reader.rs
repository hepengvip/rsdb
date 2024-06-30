use std::io::Read;

use byteorder::{BigEndian, ReadBytesExt};

use crate::packet;

pub struct PacketReader<T: Read> {
    reader: T,
}

impl<T: Read> PacketReader<T> {
    pub fn new(reader: T) -> Self {
        Self { reader }
    }

    pub fn read_packet(&mut self) -> packet::Packet {
        let header = self.read_header();

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
                packet::Packet::CmdWrite(tokens)
            }
            packet::CMD_DELETE => {
                let key_count = self.read_size();
                let mut keys = Vec::new();
                for _ in 0..key_count {
                    let key = self.read_token();
                    keys.push(key);
                }
                packet::Packet::CmdDelete(keys)
            }
            packet::CMD_READ => {
                let key_count = self.read_size();
                let mut keys = Vec::new();
                for _ in 0..key_count {
                    let key = self.read_token();
                    keys.push(key);
                }
                packet::Packet::CmdRead(keys)
            }
            packet::CMD_USE => {
                let token = self.read_token();
                packet::Packet::CmdUse(token)
            }
            packet::CMD_CURRENT_DB => packet::Packet::CmdCurrentDB(),
            packet::CMD_LIST_DB => packet::Packet::CmdListDb(),
            packet::CMD_DETACH => {
                let token = self.read_token();
                packet::Packet::CmdDetach(token)
            }

            packet::CMD_RANGE_BEGIN => {
                let page_size = self.read_size();
                packet::Packet::CmdRangeBegin(page_size)
            }
            packet::CMD_RANGE_END => {
                let page_size = self.read_size();
                packet::Packet::CmdRangeEnd(page_size)
            }
            packet::CMD_RANGE_FROM_ASC => {
                let page_size = self.read_size();
                let token = self.read_token();
                packet::Packet::CmdRangeFromAsc(page_size, token)
            }
            packet::CMD_RANGE_FROM_ASC_EX => {
                let page_size = self.read_size();
                let token = self.read_token();
                packet::Packet::CmdRangeFromAscEx(page_size, token)
            }
            packet::CMD_RANGE_FROM_DESC => {
                let page_size = self.read_size();
                let token = self.read_token();
                packet::Packet::CmdRangeFromDesc(page_size, token)
            }
            packet::CMD_RANGE_FROM_DESC_EX => {
                let page_size = self.read_size();
                let token = self.read_token();
                packet::Packet::CmdRangeFromDescEx(page_size, token)
            }

            packet::RESP_OK => {
                let message = self.read_token();
                let message = String::from_utf8(message).unwrap();
                packet::Packet::RespOk(message)
            }
            packet::RESP_ERROR => {
                let message = self.read_token();
                let message = String::from_utf8(message).unwrap();
                packet::Packet::RespError(message)
            }
            packet::RESP_TOKEN => {
                let token = self.read_token();
                packet::Packet::RespToken(token)
            }
            packet::RESP_TOKENS => {
                let token_count = self.read_size();
                let mut tokens = Vec::new();
                for _ in 0..token_count {
                    let token = self.read_token();
                    tokens.push(token);
                }
                packet::Packet::RespTokens(tokens)
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
                packet::Packet::RespPairs(pairs)
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
            packet::CMD_WRITE, // packet type id
            0,
            2, // pair count
            0,
            0,
            0,
            3,
            b'k',
            b'e',
            b'y', // key 1
            0,
            0,
            0,
            3,
            b'v',
            b'a',
            b'l', // value 1
            0,
            0,
            0,
            5,
            b'h',
            b'e',
            b'l',
            b'l',
            b'o', // key 2
            0,
            0,
            0,
            5,
            b'w',
            b'o',
            b'r',
            b'l',
            b'd', // value 2
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(
            packet,
            packet::Packet::CmdWrite(vec![
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
            packet::CMD_DELETE, // packet type id
            0,
            2, // pair count
            0,
            0,
            0,
            3,
            b'k',
            b'e',
            b'y', // key 1
            0,
            0,
            0,
            5,
            b'w',
            b'o',
            b'r',
            b'l',
            b'd', // value 2
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(
            packet,
            packet::Packet::CmdDelete(vec![b"key".to_vec(), b"world".to_vec()]),
        );
    }

    #[test]
    fn test_cmd_read() {
        let bytes = vec![
            packet::CMD_READ, // packet type id
            0,
            2, // pair count
            0,
            0,
            0,
            3,
            b'k',
            b'e',
            b'y', // key 1
            0,
            0,
            0,
            5,
            b'w',
            b'o',
            b'r',
            b'l',
            b'd', // value 2
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(
            packet,
            packet::Packet::CmdRead(vec![b"key".to_vec(), b"world".to_vec()]),
        );
    }

    #[test]
    fn test_cmd_use() {
        let bytes = vec![
            packet::CMD_USE, // packet type id
            0,
            0,
            0,
            5,
            b'w',
            b'o',
            b'r',
            b'l',
            b'd', // token
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(packet, packet::Packet::CmdUse(b"world".to_vec()),);
    }

    #[test]
    fn test_cmd_current_db() {
        let bytes = vec![packet::CMD_CURRENT_DB];
        let mut packer = PacketReader::new(&bytes[..]);
        let p = packer.read_packet();
        assert_eq!(p, packet::Packet::CmdCurrentDB());
    }

    #[test]
    fn test_cmd_list_db() {
        let bytes = vec![packet::CMD_LIST_DB];
        let mut packer = PacketReader::new(&bytes[..]);
        let p = packer.read_packet();
        assert_eq!(p, packet::Packet::CmdListDb());
    }

    #[test]
    fn test_cmd_detach() {
        let bytes = vec![
            packet::CMD_DETACH, // packet type id
            0,
            0,
            0,
            5,
            b'w',
            b'o',
            b'r',
            b'l',
            b'd', // token
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(packet, packet::Packet::CmdDetach(b"world".to_vec()),);
    }

    #[test]
    fn test_cmd_range_begin() {
        let bytes = vec![
            packet::CMD_RANGE_BEGIN, // packet type id
            2,
            16,
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(packet, packet::Packet::CmdRangeBegin(0x0210));
    }

    #[test]
    fn test_cmd_range_end() {
        let bytes = vec![
            packet::CMD_RANGE_END, // packet type id
            2,
            16,
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(packet, packet::Packet::CmdRangeEnd(0x0210));
    }

    #[test]
    fn test_cmd_range_from_asc() {
        let bytes = vec![
            packet::CMD_RANGE_FROM_ASC, // packet type id
            2,
            16, // size
            0,
            0,
            0,
            5,
            b'w',
            b'o',
            b'r',
            b'l',
            b'd', // token
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(
            packet,
            packet::Packet::CmdRangeFromAsc(0x0210, b"world".to_vec())
        );
    }

    #[test]
    fn test_cmd_range_from_asc_ex() {
        let bytes = vec![
            packet::CMD_RANGE_FROM_ASC_EX, // packet type id
            2,
            16, // size
            0,
            0,
            0,
            5,
            b'w',
            b'o',
            b'r',
            b'l',
            b'd', // token
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(
            packet,
            packet::Packet::CmdRangeFromAscEx(0x0210, b"world".to_vec())
        );
    }

    #[test]
    fn test_cmd_range_from_desc() {
        let bytes = vec![
            packet::CMD_RANGE_FROM_DESC, // packet type id
            2,
            16, // size
            0,
            0,
            0,
            5,
            b'w',
            b'o',
            b'r',
            b'l',
            b'd', // token
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(
            packet,
            packet::Packet::CmdRangeFromDesc(0x0210, b"world".to_vec())
        );
    }

    #[test]
    fn test_cmd_range_from_desc_ex() {
        let bytes = vec![
            packet::CMD_RANGE_FROM_DESC_EX, // packet type id
            2,
            16, // size
            0,
            0,
            0,
            5,
            b'w',
            b'o',
            b'r',
            b'l',
            b'd', // token
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(
            packet,
            packet::Packet::CmdRangeFromDescEx(0x0210, b"world".to_vec())
        );
    }

    #[test]
    fn test_resp_ok() {
        let bytes = vec![
            packet::RESP_OK, // packet type id
            0,
            0,
            0,
            5,
            b'w',
            b'o',
            b'r',
            b'l',
            b'd', // message
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(packet, packet::Packet::RespOk("world".to_string()),);
    }

    #[test]
    fn test_resp_error() {
        let bytes = vec![
            packet::RESP_ERROR, // packet type id
            0,
            0,
            0,
            5,
            b'w',
            b'o',
            b'r',
            b'l',
            b'd', // message
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(packet, packet::Packet::RespError("world".to_string()),);
    }

    #[test]
    fn test_resp_token() {
        let bytes = vec![
            packet::RESP_TOKEN, // packet type id
            0,
            0,
            0,
            5,
            b'w',
            b'o',
            b'r',
            b'l',
            b'd', // token
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(packet, packet::Packet::RespToken(b"world".to_vec()),);
    }

    #[test]
    fn test_resp_tokens() {
        let bytes = vec![
            packet::RESP_TOKENS, // packet type id
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
            packet::Packet::RespTokens(vec![
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
            packet::RESP_PAIRS, // packet type id
            0,
            2, // pair count
            0,
            0,
            0,
            3,
            b'k',
            b'e',
            b'y', // key 1
            0,
            0,
            0,
            5,
            b'v',
            b'a',
            b'l',
            b'u',
            b'e', // value 1
            0,
            0,
            0,
            5,
            b'h',
            b'e',
            b'l',
            b'l',
            b'o', // key 2
            0,
            0,
            0,
            5,
            b'w',
            b'o',
            b'r',
            b'l',
            b'd', // value 2
        ];
        let mut packer = PacketReader::new(&bytes[..]);
        let packet = packer.read_packet();
        assert_eq!(
            packet,
            packet::Packet::RespPairs(vec![
                b"key".to_vec(),
                b"value".to_vec(),
                b"hello".to_vec(),
                b"world".to_vec()
            ]),
        );
    }
}
