extern crate tokio;

use std::sync::Arc;
use std::sync::Mutex;

use tokio::codec::Decoder;
use tokio::codec::LinesCodec;

//use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::*;

// import BTreeMap
use std::collections::BTreeMap;

/*                                      DATABASE RESOURCES                                       */

enum Data {
    Map(BTreeMap<String, Data>),
    Value(String),
}

/*                                      STRING PARSER                                            */

enum Command {
    Keys,
    Get,
    Set,
    Remove,
    Error(String),
}

fn parse_string(mut input: String) -> (Command, Option<Vec<String>>) {
    trim_newline(&mut input);
    let input_vec = input
        .split(" ") // Split by spaces
        .map(ToString::to_string)
        .collect::<Vec<String>>();

    // TODO Check for empty result and retuen error

    match input_vec[0].to_lowercase().as_str() {
        "keys" => {
            return (Command::Keys, None);
        }
        "get" => {
            if input_vec.len() != 2 {
                return (
                    Command::Error("Error: Get receives 2 parameters!".to_string()),
                    None,
                );
            }
            let param_keys = input_vec[1]
                .split("/") // Split by slash
                .map(ToString::to_string)
                .collect::<Vec<String>>();
            return (Command::Get, Some(param_keys));
        }
        "set" => {
            if input_vec.len() != 3 {
                return (
                    Command::Error("Error: Set receives 3 parameters!".to_string()),
                    None,
                );
            }
            let param_k_v = input_vec[1..2].to_vec();
            return (Command::Set, Some(param_k_v));
        }
        "remove" => {
            // TODO finish remove parsing
            return (Command::Remove, None);
        }
        _ => {
            return (
                Command::Error("Error: Command does not exist.".to_string()),
                None,
            )
        }
    }
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

/*                                      SERVER RESOURCES                                         */

fn main() {
    let addr = "127.0.0.1:6142".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    let map: BTreeMap<String, Data> = BTreeMap::new();

    let database_arc = Arc::new(Mutex::new(map));

    let server = listener
        .incoming()
        .map_err(|e| println!("failed to accept socket; error = {:?}", e))
        .for_each(move |socket| {
            let database_arc = Arc::clone(&database_arc);
            let (lines_tx, lines_rx) = LinesCodec::new().framed(socket).split();

            let responses = lines_rx.map(move |incomming_message| {
                parse_string(incomming_message);
                return "TODO: Return proper message.".to_string();
                /*
                match incomming_message.as_ref() {
                    "keys" => {
                        let db = database_arc.lock().unwrap();
                        let keys: Vec<_> = (*db).keys().cloned().collect();
                        return format!("The database keys are: {:?}\n", keys);
                    }
                    "insert" => {
                        let mut db = database_arc.lock().unwrap();
                        (*db).insert("key1".to_string(), Data::Value("value1".to_string()));
                        return format!("Done?");
                    }
                    "get" => {
                        // fix printing
                        let db = database_arc.lock().unwrap();
                        let result = (*db).get(&("key1".to_string())).unwrap();
                        match result {
                            Data::Value(val) => {
                                return format!("{}\n", val);
                            }
                            Data::Map(_) => {
                                return format!("Map"); // Fix proper printing
                            }
                        }
                    }
                    "insertbtree" => {
                        let mut db = database_arc.lock().unwrap();
                        let new_tree: BTreeMap<String, Data> = BTreeMap::new();
                        (*db).insert("Employees".to_string(), Data::Map(new_tree));
                        return format!("Done?");
                    }
                    "getbtree" => {
                        let db = database_arc.lock().unwrap();
                        let result = (*db).get(&("Employees".to_string())).unwrap();
                        match result {
                            Data::Value(val) => {
                                return format!("{}\n", val);
                            }
                            Data::Map(map) => {
                                let keys: Vec<_> = map.keys().cloned().collect();
                                return format!(
                                    "The values stored under {} are: {:?}\n",
                                    "Employees", keys
                                );
                            }
                        }
                    }
                    _ => {
                        return format!("The commands are: insert, get, remove, keys.\n");
                    }
                }
                */
                //return incomming_message;
            });

            let writes = responses.fold(lines_tx, |writer, response| {
                //Return the future that handles to send the response to the socket
                writer.send(response)
            });

            tokio::spawn(writes.then(move |_| Ok(())));

            Ok(())
        });

    println!("Server running on {}", addr);

    tokio::run(server);
}
