use std::{net::TcpStream, time::Duration};
use std::thread;

extern crate packet;

use packet::{Packet, PacketReaderWriter};


fn main() {
    let stream = TcpStream::connect("127.0.0.1:1935").unwrap();

    let mut cnt = 0;
    let mut rw = PacketReaderWriter::new(stream);
    loop {
        let p = Packet::CmdWrite(vec![
            b"hello".to_vec(),
            b"world".to_vec(),
        ]);
        rw.write_packet(&p);
        let p = rw.read_packet();
        cnt += 1;
        println!("{cnt} Received response: {:?}", p);

        if cnt >= 10 {
            break;
        }

        thread::sleep(Duration::from_secs(1));
    }
}
