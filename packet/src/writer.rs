use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};

use crate::packet;

pub struct PacketWriter<T: Write> {
    writer: T,
}

impl<T: Write> PacketWriter<T> {
    pub fn new(writer: T) -> Self {
        Self { writer }
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
        packer.write_packet(&packet::Packet::CmdWrite(vec![
            b"key".to_vec(),
            b"val".to_vec(),
        ]));
        assert_eq!(
            writer,
            [
                packet::CMD_WRITE,
                0,
                1,
                0,
                0,
                0,
                3,
                b'k',
                b'e',
                b'y',
                0,
                0,
                0,
                3,
                b'v',
                b'a',
                b'l'
            ]
        );
    }

    #[test]
    fn test_cmd_read() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        packer.write_packet(&&packet::Packet::CmdRead(vec![
            b"key".to_vec(),
            b"val".to_vec(),
        ]));
        assert_eq!(
            writer,
            [
                packet::CMD_READ,
                0,
                2,
                0,
                0,
                0,
                3,
                b'k',
                b'e',
                b'y',
                0,
                0,
                0,
                3,
                b'v',
                b'a',
                b'l'
            ]
        );
    }

    #[test]
    fn test_cmd_delete() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        packer.write_packet(&&packet::Packet::CmdDelete(vec![
            b"key".to_vec(),
            b"val".to_vec(),
        ]));
        assert_eq!(
            writer,
            [
                packet::CMD_DELETE,
                0,
                2,
                0,
                0,
                0,
                3,
                b'k',
                b'e',
                b'y',
                0,
                0,
                0,
                3,
                b'v',
                b'a',
                b'l'
            ]
        );
    }

    #[test]
    fn test_cmd_use() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = packet::Packet::CmdUse(b"world".to_vec());
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [packet::CMD_USE, 0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd'],
        );
    }
    #[test]
    fn test_cmd_current_db() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = packet::Packet::CmdCurrentDB();
        packer.write_packet(&packet);
        assert_eq!(writer, [packet::CMD_CURRENT_DB],);
    }
    #[test]
    fn test_cmd_list_db() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = packet::Packet::CmdListDb();
        packer.write_packet(&packet);
        assert_eq!(writer, [packet::CMD_LIST_DB],);
    }
    #[test]
    fn test_cmd_detach() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = packet::Packet::CmdDetach(b"world".to_vec());
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [packet::CMD_DETACH, 0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd'],
        );
    }

    #[test]
    fn test_cmd_range_begin() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = packet::Packet::CmdRangeBegin(0x0210);
        packer.write_packet(&packet);
        assert_eq!(writer, [packet::CMD_RANGE_BEGIN, 2, 16],);
    }

    #[test]
    fn test_cmd_range_end() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = packet::Packet::CmdRangeBegin(0x0210);
        packer.write_packet(&packet);
        assert_eq!(writer, [packet::CMD_RANGE_BEGIN, 2, 16],);
    }

    #[test]
    fn test_cmd_range_from_asc() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = packet::Packet::CmdRangeFromAsc(0x0210, b"world".to_vec());
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [
                packet::CMD_RANGE_FROM_ASC,
                2,
                16,
                0,
                0,
                0,
                5,
                b'w',
                b'o',
                b'r',
                b'l',
                b'd'
            ],
        );
    }

    #[test]
    fn test_cmd_range_from_asc_ex() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = packet::Packet::CmdRangeFromAscEx(0x0210, b"world".to_vec());
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [
                packet::CMD_RANGE_FROM_ASC_EX,
                2,
                16,
                0,
                0,
                0,
                5,
                b'w',
                b'o',
                b'r',
                b'l',
                b'd'
            ],
        );
    }

    #[test]
    fn test_cmd_range_from_desc() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = packet::Packet::CmdRangeFromDesc(0x0210, b"world".to_vec());
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [
                packet::CMD_RANGE_FROM_DESC,
                2,
                16,
                0,
                0,
                0,
                5,
                b'w',
                b'o',
                b'r',
                b'l',
                b'd'
            ],
        );
    }

    #[test]
    fn test_cmd_range_from_desc_ex() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = packet::Packet::CmdRangeFromDescEx(0x0210, b"world".to_vec());
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [
                packet::CMD_RANGE_FROM_DESC_EX,
                2,
                16,
                0,
                0,
                0,
                5,
                b'w',
                b'o',
                b'r',
                b'l',
                b'd'
            ],
        );
    }

    #[test]
    fn test_resp_ok() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = packet::Packet::RespOk("world".to_string());
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [packet::RESP_OK, 0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd'],
        );
    }

    #[test]
    fn test_resp_error() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = packet::Packet::RespError("world".to_string());
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [packet::RESP_ERROR, 0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd'],
        );
    }

    #[test]
    fn test_resp_token() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = packet::Packet::RespToken(b"world".to_vec());
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [packet::RESP_TOKEN, 0, 0, 0, 5, b'w', b'o', b'r', b'l', b'd'],
        );
    }

    #[test]
    fn test_resp_tokens() {
        let mut writer = Vec::new();
        let mut packer = PacketWriter::new(&mut writer);
        let packet = packet::Packet::RespTokens(vec![
            "hello".as_bytes().to_vec(),
            "world".as_bytes().to_vec(),
            vec![],
            "rust".as_bytes().to_vec(),
        ]);
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [
                packet::RESP_TOKENS,
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
        let packet = packet::Packet::RespPairs(vec![
            "hello".as_bytes().to_vec(),
            "world".as_bytes().to_vec(),
        ]);
        packer.write_packet(&packet);
        assert_eq!(
            writer,
            [
                packet::RESP_PAIRS,
                0,
                1,
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
}
