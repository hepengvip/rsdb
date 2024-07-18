use std::io::{stdout, Write};

use clap::Parser;

extern crate rustyline;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use rsdbrs::{Direction, IteratorMode, RsDBClient};

#[derive(Parser, Debug)]
#[command(version, about, long_about = "RSDB client utility")]
struct Args {
    #[arg(short, long, default_value_t=String::from("127.0.0.1:10110"))]
    addr: String,
}

fn main() {
    let args = Args::parse();

    let mut rsdb_cli = RsDBClient::new();
    rsdb_cli.connect(&args.addr).unwrap();

    let mut rl = DefaultEditor::new().unwrap();

    #[cfg(feature = "with-file-history")]
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    let pkg_name = env!("CARGO_PKG_NAME");
    let pkg_version = env!("CARGO_PKG_VERSION");

    println!("\n\t{pkg_name} {pkg_version}\n");
    println!("    > Type `help` for a list of commands.\n");

    let mut db_name = String::from("(none) ");

    loop {
        if let Some(name) = rsdb_cli.get_db_name() {
            db_name = format!("({}) ", name);
        }

        println!("");
        let readline = rl.readline(db_name.as_str());
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                // println!("Line: {}", line);

                let valid_part = line.trim();
                if valid_part == "quit" {
                    println!("Bye.");
                    break;
                }
                let parts: Vec<&str> = valid_part.split_ascii_whitespace().collect();
                if parts.len() == 0 {
                    continue;
                }

                match parts[0] {
                    "help" => {
                        println!(" Commands currently supported:");
                        println!("                 set - Set key:value pair");
                        println!("                 get - Get value by key");
                        println!("              delete - Delete by key");
                        println!("                 use - Select/Attached a database");
                        println!("          current_db - Get current database");
                        println!(
                            "             list_db - List all the databases currently attached"
                        );
                        println!("              detach - Detach a database\n");
                        println!("         range_begin - Range pairs from begin");
                        println!("           range_end - Range pairs from a key");
                        println!(
                            "      range_from_asc - Range pairs from a key (inluding the current key)"
                        );
                        println!("    range_end_asc_ex - Range pairs from end");
                        println!(
                            "     range_from_desc - Range pairs from a key (inluding the current key)"
                        );
                        println!("  range_from_desc_ex - Range pairs from a key");
                        continue;
                    }
                    "set" => {
                        if parts.len() != 3 {
                            println!("Error: invalid parameter for set");
                            continue;
                        }
                        let rs = rsdb_cli.set(parts[1].as_bytes(), parts[2].as_bytes());
                        if let Err(e) = rs {
                            println!("Error: {}", e);
                        } else {
                            println!("Info: Ok.")
                        }
                    }
                    "get" => {
                        if parts.len() != 2 {
                            println!("Error: invalid parameter for get");
                            continue;
                        }
                        let rs = rsdb_cli.get(parts[1].as_bytes());
                        match rs {
                            Err(e) => println!("Error: {}", e),
                            Ok(val) => {
                                if let Some(val) = val {
                                    let m = String::from_utf8(val).unwrap();
                                    println!("Value: {m}");
                                } else {
                                    println!("Value: <none>")
                                }
                            }
                        }
                    }
                    "delete" => {
                        if parts.len() != 2 {
                            println!("Error: invalid parameter for delete");
                            continue;
                        }
                        let rs = rsdb_cli.delete(parts[1].as_bytes());
                        if let Err(e) = rs {
                            println!("Error: {}", e);
                        } else {
                            println!("Info: Ok.")
                        }
                    }
                    "use" => {
                        if parts.len() != 2 {
                            println!("Error: invalid parameter for use");
                            continue;
                        }

                        let rs = rsdb_cli.use_db(parts[1]);
                        if let Err(e) = rs {
                            println!("Error: {}", e);
                        } else {
                            println!("Info: Ok.")
                        }
                    }
                    "current_db" => {
                        if parts.len() != 1 {
                            println!("Error: invalid parameter for current_db");
                            continue;
                        }

                        let rs = rsdb_cli.get_current_db();
                        match rs {
                            Err(e) => println!("Error: {}", e),
                            Ok(val) => {
                                if let Some(m) = val {
                                    println!("Value: {m}");
                                } else {
                                    println!("Value: <none>")
                                }
                            }
                        }
                    }
                    "list_db" => {
                        if parts.len() != 1 {
                            println!("Error: invalid parameter for list_db");
                            continue;
                        }

                        let rs = rsdb_cli.list_db();
                        match rs {
                            Err(e) => println!("Error: {}", e),
                            Ok(val) => {
                                println!("Activited databases:");
                                for db_name in val {
                                    println!("  - {db_name}");
                                }
                            }
                        }
                    }
                    "detach" => {
                        if parts.len() != 2 {
                            println!("Error: invalid parameter for detach");
                            continue;
                        }

                        let rs = rsdb_cli.detach_db(parts[1]);
                        if let Err(e) = rs {
                            println!("Error: {}", e);
                        } else {
                            println!("Info: Ok.")
                        }
                    }
                    "range_begin" => {
                        if parts.len() != 2 {
                            println!("Error: invalid parameter for range_begin");
                            continue;
                        }

                        let page_size = parts[1].parse::<u16>().unwrap();
                        let iter_mode = IteratorMode::Start;
                        let rs = rsdb_cli.range(iter_mode, page_size, false);
                        match rs {
                            Err(e) => println!("Error: {}", e),
                            Ok(val) => {
                                println!("Item pairs:");
                                for (key, val) in val {
                                    let key = String::from_utf8(key).unwrap();
                                    let val = String::from_utf8(val).unwrap();
                                    println!("  {}: {}", key, val);
                                }
                            }
                        }
                    }
                    "range_end" => {
                        if parts.len() != 2 {
                            println!("Error: invalid parameter for range_end");
                            continue;
                        }

                        let page_size = parts[1].parse::<u16>().unwrap();
                        let iter_mode = IteratorMode::End;
                        let rs = rsdb_cli.range(iter_mode, page_size, false);
                        match rs {
                            Err(e) => println!("Error: {}", e),
                            Ok(val) => {
                                println!("Item pairs:");
                                for (key, val) in val {
                                    let key = String::from_utf8(key).unwrap();
                                    let val = String::from_utf8(val).unwrap();
                                    println!("  {}: {}", key, val);
                                }
                            }
                        }
                    }
                    "range_from_asc" => {
                        if parts.len() != 3 {
                            println!("Error: invalid parameter for range_from_asc");
                            continue;
                        }

                        let page_size_rs = parts[1].parse::<u16>();
                        let page_size = if let Ok(page_size) = page_size_rs {
                            page_size
                        } else {
                            continue;
                        };
                        let iter_mode = IteratorMode::From(parts[2].as_bytes(), Direction::Forward);
                        let rs = rsdb_cli.range(iter_mode, page_size, false);
                        match rs {
                            Err(e) => println!("Error: {}", e),
                            Ok(val) => {
                                println!("Item pairs:");
                                for (key, val) in val {
                                    let key = String::from_utf8(key).unwrap();
                                    let val = String::from_utf8(val).unwrap();
                                    println!("  {}: {}", key, val);
                                }
                            }
                        }
                    }
                    "range_from_asc_ex" => {
                        if parts.len() != 3 {
                            println!("Error: invalid parameter for range_from_asc");
                            continue;
                        }

                        let page_size_rs = parts[1].parse::<u16>();
                        let page_size = if let Ok(page_size) = page_size_rs {
                            page_size
                        } else {
                            continue;
                        };
                        let iter_mode = IteratorMode::From(parts[2].as_bytes(), Direction::Forward);
                        let rs = rsdb_cli.range(iter_mode, page_size, true);
                        match rs {
                            Err(e) => println!("Error: {}", e),
                            Ok(val) => {
                                println!("Item pairs:");
                                for (key, val) in val {
                                    let key = String::from_utf8(key).unwrap();
                                    let val = String::from_utf8(val).unwrap();
                                    println!("  {}: {}", key, val);
                                }
                            }
                        }
                    }
                    "range_from_desc" => {
                        if parts.len() != 3 {
                            println!("Error: invalid parameter for range_from_asc");
                            continue;
                        }

                        let page_size_rs = parts[1].parse::<u16>();
                        let page_size = if let Ok(page_size) = page_size_rs {
                            page_size
                        } else {
                            continue;
                        };
                        let iter_mode = IteratorMode::From(parts[2].as_bytes(), Direction::Reverse);
                        let rs = rsdb_cli.range(iter_mode, page_size, false);
                        match rs {
                            Err(e) => println!("Error: {}", e),
                            Ok(val) => {
                                println!("Item pairs:");
                                for (key, val) in val {
                                    let key = String::from_utf8(key).unwrap();
                                    let val = String::from_utf8(val).unwrap();
                                    println!("  {}: {}", key, val);
                                }
                            }
                        }
                    }
                    "range_from_desc_ex" => {
                        if parts.len() != 3 {
                            println!("Error: invalid parameter for range_from_asc");
                            continue;
                        }

                        let page_size_rs = parts[1].parse::<u16>();
                        let page_size = if let Ok(page_size) = page_size_rs {
                            page_size
                        } else {
                            continue;
                        };
                        let iter_mode = IteratorMode::From(parts[2].as_bytes(), Direction::Reverse);
                        let rs = rsdb_cli.range(iter_mode, page_size, true);
                        match rs {
                            Err(e) => println!("Error: {}", e),
                            Ok(val) => {
                                println!("Item pairs:");
                                for (key, val) in val {
                                    let key = String::from_utf8(key).unwrap();
                                    let val = String::from_utf8(val).unwrap();
                                    println!("  {}: {}", key, val);
                                }
                            }
                        }
                    }
                    _ => {
                        println!("Error: unknown command `{}`", parts[0]);
                        continue;
                    }
                }

                stdout().flush().unwrap();
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
