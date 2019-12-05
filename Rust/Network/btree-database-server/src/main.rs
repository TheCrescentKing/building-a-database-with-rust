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
    SetVal,
    NewDir,
    Remove,
    Error(String),
}

fn parse_string(mut input: String) -> (Command, Option<Vec<String>>) {
    trim_newline(&mut input);
    let input_vec = input
        .split(" ") // Split by spaces
        .map(ToString::to_string)
        .collect::<Vec<String>>();

    if input_vec.len() <= 0 {
        return (Command::Error("Please input a command!".to_string()), None);
    }

    match input_vec[0].to_lowercase().as_str() {
        "keys" => {
            return (Command::Keys, None);
        }
        "get" => {
            if input_vec.len() != 2 {
                return (
                    Command::Error("Error: Get receives 1 parameter!".to_string()),
                    None,
                );
            }
            let param_keys = input_vec[1]
                .split("/") // Split by slash
                .map(ToString::to_string)
                .collect::<Vec<String>>();
            return (Command::Get, Some(param_keys));
        }
        "setval" => {
            if input_vec.len() != 3 {
                return (
                    Command::Error("Error: Setval receives 2 parameters!".to_string()),
                    None,
                );
            }
            let param_k_v = input_vec[1..2].to_vec(); // TODO Process multiple keys
            return (Command::SetVal, Some(param_k_v));
        }
        "newdir" => {
            if input_vec.len() != 2 {
                return (
                    Command::Error("Error: NewDir receives 1 parameter!".to_string()),
                    None,
                );
            }
            let param_k_v = input_vec[1..2].to_vec(); // TODO Process multiple keys
            return (Command::NewDir, Some(param_k_v));
        }
        "remove" => {
            if input_vec.len() != 2 {
                return (
                    Command::Error("Error: Remove receives 1 parameter!".to_string()),
                    None,
                );
            }
            let param_remove_key = vec![input_vec[1].to_string()];
            return (Command::Remove, Some(param_remove_key));
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
                let (command, parameters) = parse_string(incomming_message);
                match command {
                    Command::Error(msg) => {
                        return msg;
                    }
                    Command::Keys => {
                        let db = database_arc.lock().unwrap();
                        let keys: Vec<_> = (*db).keys().cloned().collect();
                        return format!("The database keys are: {:?}\n", keys);
                    }
                    Command::Get => {
                        let key = parameters.unwrap(); //Can safely do this as fisrt match is Error
                        if key.len() == 1 {
                            let db = database_arc.lock().unwrap();
                            let result = (*db).get(&(key[0])).unwrap(); // Error check for n/a val
                            match result {
                                Data::Value(val) => {
                                    return format!("{}\n", val);
                                }
                                Data::Map(map) => {
                                    let keys: Vec<_> = map.keys().cloned().collect();
                                    return format!(
                                        "The values stored under {} are: {:?}\n",
                                        &(key[0]),
                                        keys
                                    );
                                }
                            }
                        } else {
                            // TODO hanlde case when multiple tree keys
                        }
                    }
                    Command::SetVal => {
                        let key = parameters.unwrap(); //Can safely do this as fisrt match is Error
                        let mut db = database_arc.lock().unwrap();
                        // TODO Reformat to take into account multiple btree keys
                        (*db).insert((*key[0]).to_string(), Data::Value((*key[1]).to_string()));
                        return format!("Set done.");
                    }
                    Command::NewDir => {
                        let key = parameters.unwrap();
                        let mut db = database_arc.lock().unwrap();
                        let new_tree: BTreeMap<String, Data> = BTreeMap::new();
                        // TODO Reformat to take into account multiple btree keys
                        (*db).insert((*key[0]).to_string(), Data::Map(new_tree));
                        return format!("Newdir done.");
                    }
                    Command::Remove => {
                        let key = parameters.unwrap(); //Can safely do this as fisrt match is Error
                        if key.len() == 1 {
                            let mut db = database_arc.lock().unwrap();
                            let result = (*db).remove(&(key[0])).unwrap(); // Error check for n/a val
                            match result {
                                Data::Value(val) => {
                                    return format!("Removed value: {}\n", val);
                                }
                                Data::Map(_) => {
                                    return format!("Removed directory under: {}\n", &(key[0]));
                                }
                            }
                        }
                    }
                }
                return format!("ERROR: Program reached en of command match without returning!");
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
