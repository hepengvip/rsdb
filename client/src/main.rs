use std::io::{stdin, stdout, Write};
use std::net::TcpStream;

extern crate packet;

use packet::{Packet, PacketReaderWriter};

fn main() {
    let mut buf = String::new();
    let stream = TcpStream::connect("127.0.0.1:1935").unwrap();
    let mut rw = PacketReaderWriter::new(stream);

    loop {
        print!(">> ");
        stdout().flush().unwrap();
        buf.clear();

        stdin().read_line(&mut buf).unwrap();
        let valid_part = buf.trim();
        if valid_part == "quit" {
            println!("Bye.");
            break;
        }

        // println!("Received: `{}`", valid_part);
        let parts: Vec<&str> = valid_part.split_ascii_whitespace().collect();
        if parts.len() == 0 {
            continue;
        }

        let mut bytes_parts = vec![];
        match parts[0] {
            "set" => {
                if parts.len() < 3 || parts.len() % 2 == 0 {
                    println!("Error: invalid parameter for set");
                    continue;
                }
                let pairs = parts.len() / 2;

                for idx in 0..pairs {
                    let begin = idx * 2;
                    // println!("    - SET `{}` `{}`", parts[begin + 1], parts[begin + 2]);
                    bytes_parts.push(parts[begin + 1].as_bytes().to_vec());
                    bytes_parts.push(parts[begin + 2].as_bytes().to_vec());
                }
                let packet = Packet::CmdWrite(bytes_parts);
                rw.write_packet(&packet);
            }
            "get" => {
                if parts.len() < 2 {
                    println!("Error: invalid parameter for get");
                    continue;
                }

                for idx in 1..parts.len() {
                    // println!("    - GET `{}`", parts[idx]);
                    bytes_parts.push(parts[idx].as_bytes().to_vec());
                }
                let packet = Packet::CmdRead(bytes_parts);
                rw.write_packet(&packet);
            }
            "delete" => {
                if parts.len() < 2 {
                    println!("Error: invalid parameter for delete");
                    continue;
                }

                for idx in 1..parts.len() {
                    // println!("    - DELETE `{}`", parts[idx]);
                    bytes_parts.push(parts[idx].as_bytes().to_vec());
                }
                let packet = Packet::CmdDelete(bytes_parts);
                rw.write_packet(&packet);
            }
            _ => {
                println!("Error: unknown command `{}`", parts[0]);
                continue;
            }
        }

        let resp = rw.read_packet().unwrap();
        match resp {
            Packet::RespOk(ref msg) => {
                println!("MessageOk: {}", msg);
            }
            Packet::RespError(ref msg) => {
                println!("MessageError: {}", msg);
            }
            Packet::RespToken(ref data) => {
                let msg = String::from_utf8_lossy(data.as_slice());
                println!("Token: {}", msg);
            }
            Packet::RespTokens(ref data) => {
                println!("Tokens:");
                for part in data {
                    // let msg = String::from_utf8_lossy(part.as_slice());
                    let token = if part.len() == 0 {
                        String::from("<None>")
                    } else {
                        String::from_utf8(part.to_vec()).unwrap()
                    };
                    println!("    - {}", token);
                }
            }
            Packet::RespPairs(ref data) => {
                println!("Pairs:");
                for (k, v) in data.iter().enumerate() {
                    let token = if v.len() == 0 {
                        String::from("<None>")
                    } else {
                        String::from_utf8(v.to_vec()).unwrap()
                    };
                    if k % 2 == 1 {
                        print!("    - {} => ", token);
                    } else {
                        println!("{}", token);
                    }
                }
            }
            _ => {
                println!("Error: unknown response {:?}", resp);
            }
        }

        stdout().flush().unwrap();
    }
}
