use std::time::Instant;
use uuid::Uuid;

use rsdb_rs::RsDBClient;

fn main() {
    let mut rsdb_cli = RsDBClient::new();
    rsdb_cli.connect("127.0.0.1:10110").unwrap();

    rsdb_cli.use_db("test").unwrap();

    let n: usize = 10000;
    let mut pairs = Vec::with_capacity(n);

    for _ in 0..n {
        let uuid = Uuid::new_v4();
        let key: String = uuid.into();
        let uuid = Uuid::new_v4();
        let value: String = uuid.into();

        pairs.push((key, value));
    }

    set_pairs(&pairs, &mut rsdb_cli)
}

fn set_pairs(pairs: &Vec<(String, String)>, rsdb_cli: &mut RsDBClient) {
    let start = Instant::now();
    for (key, value) in pairs {
        rsdb_cli.set(key.as_bytes(), value.as_bytes()).unwrap();
        // println!("set {} {}", key, value);
    }
    println!(
        "set {} pairs in {:.2?}ms",
        pairs.len(),
        start.elapsed().as_millis()
    );
    println!("All good.")
}
