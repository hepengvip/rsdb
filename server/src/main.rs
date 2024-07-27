extern crate packet;
extern crate storage;

use clap::Parser;

mod errors;
mod logic;

#[derive(Parser, Debug)]
#[command(version, about, long_about = "RSDB server")]
struct Args {
    #[arg(short, long)]
    addr: Option<String>,

    #[arg(short, long)]
    root: String,

    #[arg(short, long)]
    unix_addr: Option<String>,
}

fn main() {
    let args = Args::parse();

    let pkg_name = env!("CARGO_PKG_NAME");
    let pkg_version = env!("CARGO_PKG_VERSION");

    println!("\n\t{pkg_name} {pkg_version}\n");

    let s = logic::Server::new(args.addr, &args.root, args.unix_addr);
    match s {
        Err(e) => eprintln!("Error: {}", e),
        Ok(s) => s.listen_and_serve().unwrap(),
    }
}
