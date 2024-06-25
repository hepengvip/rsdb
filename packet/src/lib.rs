use std::io::{BufRead};


const CMD_LENGTH: usize = 1;
const LEN_LENGTH: usize = 2;
const TOKEN_LENGTH: usize = 4;

const CMD_IDCODE : u8 = 0x01;
const CMD_WRITE : u8 = 0x02;
const CMD_READ  : u8 = 0x03;
const CMD_DELETE: u8 = 0x04;
const CMD_RESP_OK: u8 = 0x55;
const CMD_RESP_ERROR: u8 = 0x56;
const CMD_RESP_LIST: u8 = 0x57;


pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub enum Packet {
    Write(Vec<Vec<u8>>),
    Read(Vec<Vec<u8>>),
    Delete(Vec<Vec<u8>>),
    RespOk,
    RespError(String),
    RespList(Vec<Vec<u8>>),
}

pub struct Packer<T: BufRead> {
    reader: T,
}

impl<T: BufRead> Packer<T> {
    pub fn new(reader: T) -> Self {
        Self { reader }
    }

    // pub fn read_header(&mut self) -> Option<u32> {
    //     let mut header = [0; HEADER_LENGTH as usize];
    //     if self.reader.read_exact(&mut header).is_err() {
    //         return None;
    //     }
    //     Some(u32::from_be_bytes(header))
    // }

    // pub fn read_key(&mut self, length: u32) -> Option<Vec<u8>> {
    //     let mut key = vec![0; length as usize];
    //     if self.reader.read_exact(&mut key).is_err() {
    //         return None;
    //     }
    //     Some(key)
    // }

    // pub fn read_value(&mut self, length: u32) -> Option<Vec<u8>> {
    //     let mut value = vec![0; length as usize];
    //     if self.reader.read_exact(&mut value).is_err() {
    //         return None;
    //     }
    //     Some(value)
    // }

    // pub fn read_record(&mut self) -> Option<Record> {
    //     let length = self.read_header()?;
    //     let key = self.read_key(length)?;
    //     let value = self.read_value(length)?;
    //     Some(Record::new(&key, &value))
    // }
}






#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
