use rsdbrs::{Direction, IteratorMode, RsDBClient};

fn main() {
    let mut rsdb_cli = RsDBClient::new();
    rsdb_cli.connect("127.0.0.1:10110").unwrap();

    rsdb_cli.use_db("ts.ru2501").unwrap();

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
            for (k, v) in rs {
                cnt += 1;
                let ks = String::from_utf8((&k).to_vec()).unwrap();
                let vs = String::from_utf8(v.to_vec()).unwrap();
                println!("{}: {} => {}", cnt, ks, vs);
                last_key = k;
            }
        } else {
            println!("Error: {}", rs.unwrap_err());
            break;
        }
    }
    println!("All good.");
}
