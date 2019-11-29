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

/*                                     DATABASE RESOURCES                                        */

enum Data {
    Map(BTreeMap<String, Data>),
    Value(String),
}

/*                                     SERVER RESOURCES                                          */

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
