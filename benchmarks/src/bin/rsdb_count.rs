use std::env::args;

use rsdbrs::{Direction, IteratorMode, RsDBClient};

fn main() {
    let mut rsdb_cli = RsDBClient::new();
    rsdb_cli.connect("127.0.0.1:10110").unwrap();
    // rsdb_cli.connect("/tmp/rsdb.sock").unwrap();

    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        println!("Usage: {} <db_name>", args[0]);
        return;
    }

    println!("Counting {}...", &args[1]);

    rsdb_cli.use_db(&args[1]).unwrap();

    let mut cnt: usize = 0;
    let page_size = 1000;
    let mut last_key = vec![];
    loop {
        let rs = if cnt == 0 {
            rsdb_cli.range(IteratorMode::Start, page_size, true)
        } else {
            let iter_mode = IteratorMode::From(&last_key, Direction::Forward);
            rsdb_cli.range(iter_mode, page_size, true)
        };
        if let Ok(rs) = rs {
            if rs.len() == 0 {
                break;
            }
            for (k, _v) in rs {
                cnt += 1;
                // let ks = String::from_utf8((&k).to_vec()).unwrap();
                // let vs = String::from_utf8(v.to_vec()).unwrap();
                last_key = k;
            }
        } else {
            println!("Error: {}", rs.unwrap_err());
            break;
        }
    }
    println!("pairs {}", cnt);
}
