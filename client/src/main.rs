use std::io::{stdin, stdout, Write};
use std::net::TcpStream;

extern crate packet;

use clap::Parser;
use packet::{Packet, PacketReaderWriter};

#[derive(Parser, Debug)]
#[command(version, about, long_about = "RSDB server")]
struct Args {
    #[arg(short, long, default_value_t=String::from("127.0.0.1:1935"))]
    addr: String,
}

fn main() {
    let args = Args::parse();

    let mut buf = String::new();
    let stream = TcpStream::connect(args.addr).unwrap();
    let mut rw = PacketReaderWriter::new(stream);

    let mut current_db_name = String::new();

    loop {
        if current_db_name.len() == 0 {
            print!("(none) ");
        } else {
            print!("({}) ", &current_db_name);
        }

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
            "help" => {
                println!(" Commands currently supported:");
                println!("                 set - Set key:value pairs");
                println!("                 get - Get value by keys");
                println!("              delete - Delete by keys");
                println!("                 use - Select/Attached a database");
                println!("          current_db - Get current database");
                println!("             list_db - List all the databases currently attached");
                println!("              detach - Detach a database\n");
                println!("         range_begin - Range pairs from begin");
                println!("           range_end - Range pairs from a key");
                println!(
                    "      range_from_asc - Range pairs from a key (inluding the current key)"
                );
                println!("    range_end_asc_ex - Range pairs from end");
                println!("     range_from_desc - Range pairs from a key");
                println!(
                    "  range_from_desc_ex - Range pairs from a key (inluding the current key)"
                );
                continue;
            }
            "set" => {
                if !validate_dbname(&current_db_name) {
                    continue;
                }

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
                if !validate_dbname(&current_db_name) {
                    continue;
                }

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
                if !validate_dbname(&current_db_name) {
                    continue;
                }

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
            "use" => {
                if parts.len() != 2 {
                    println!("Error: invalid parameter for use");
                    continue;
                }

                let bytes = parts[1].as_bytes().to_vec();
                let packet = Packet::CmdUse(bytes);
                rw.write_packet(&packet);
            }
            "current_db" => {
                if parts.len() != 1 {
                    println!("Error: invalid parameter for current_db");
                    continue;
                }

                let packet = Packet::CmdCurrentDB();
                rw.write_packet(&packet);
            }
            "list_db" => {
                if parts.len() != 1 {
                    println!("Error: invalid parameter for list_db");
                    continue;
                }

                let packet = Packet::CmdListDb();
                rw.write_packet(&packet);
            }
            "detach" => {
                if parts.len() != 2 {
                    println!("Error: invalid parameter for detach");
                    continue;
                }

                let bytes = parts[1].as_bytes().to_vec();
                let packet = Packet::CmdDetach(bytes);
                rw.write_packet(&packet);
            }
            "range_begin" => {
                if parts.len() != 2 {
                    println!("Error: invalid parameter for range_begin");
                    continue;
                }

                let page_size = parts[1].parse::<u16>().unwrap();
                let packet = Packet::CmdRangeBegin(page_size);
                rw.write_packet(&packet);
            }
            "range_end" => {
                if parts.len() != 2 {
                    println!("Error: invalid parameter for range_end");
                    continue;
                }

                let page_size = parts[1].parse::<u16>().unwrap();
                let packet = Packet::CmdRangeEnd(page_size);
                rw.write_packet(&packet);
            }
            "range_from_asc" => {
                if parts.len() != 3 {
                    println!("Error: invalid parameter for range_from_asc");
                    continue;
                }

                let page_size = parts[1].parse::<u16>().unwrap();
                let packet = Packet::CmdRangeFromAsc(page_size, parts[2].as_bytes().to_vec());
                rw.write_packet(&packet);
            }
            "range_from_asc_ex" => {
                if parts.len() != 3 {
                    println!("Error: invalid parameter for range_from_asc");
                    continue;
                }

                let page_size = parts[1].parse::<u16>().unwrap();
                let packet = Packet::CmdRangeFromAscEx(page_size, parts[2].as_bytes().to_vec());
                rw.write_packet(&packet);
            }
            "range_from_desc" => {
                if parts.len() != 3 {
                    println!("Error: invalid parameter for range_from_asc");
                    continue;
                }

                let page_size = parts[1].parse::<u16>().unwrap();
                let packet = Packet::CmdRangeFromDesc(page_size, parts[2].as_bytes().to_vec());
                rw.write_packet(&packet);
            }
            "range_from_desc_ex" => {
                if parts.len() != 3 {
                    println!("Error: invalid parameter for range_from_asc");
                    continue;
                }

                let page_size = parts[1].parse::<u16>().unwrap();
                let packet = Packet::CmdRangeFromDescEx(page_size, parts[2].as_bytes().to_vec());
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

                // current db
                if parts[0] == "use" {
                    current_db_name = parts[1].to_string();
                }

                // detach
                if parts[0] == "detach" && current_db_name == parts[1].to_string() {
                    current_db_name.clear();
                }
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
                    if k % 2 == 0 {
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

fn validate_dbname(name: &str) -> bool {
    if name.len() == 0 {
        println!("Error: no db selected");
        return false;
    }
    true
}
