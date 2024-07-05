use rsdb_rs::{RsDBClient, IteratorMode, Direction};


fn main() {
    let mut rsdb_cli = RsDBClient::new();
    rsdb_cli.connect("127.0.0.1:10110").unwrap();

    rsdb_cli.use_db("test").unwrap();

    let mut cnt: usize = 0;
    let page_size = 100;
    let mut last_key = vec![];
    loop {
        let rs = if cnt == 0 {
            rsdb_cli.range(IteratorMode::Start, page_size, true)
        } else {
            let iter_mode = IteratorMode::From(&last_key, Direction::Forward);
            rsdb_cli.range(iter_mode, page_size, true)
        };
        cnt += 1;
        if let Ok(rs) = rs {
            if rs.len() == 0 {
                break;
            }
            for (k, v) in rs {
                let ks = String::from_utf8((&k).to_vec()).unwrap();
                let vs = String::from_utf8(v.to_vec()).unwrap();
                println!("{}: {}", ks, vs);
                last_key = k;
            }
        } else {
            println!("Error: {}", rs.unwrap_err());
            break;
        }
    }
    println!("All good.");
}
