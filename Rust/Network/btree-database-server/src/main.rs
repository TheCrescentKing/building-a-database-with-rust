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
use std::collections::btree_map::Entry::*;

/*                                      DATABASE RESOURCES                                       */
#[derive(Clone)]
enum Data {
    Map(BTreeMap<String, Data>),
    Value(String),
}

fn get_keys(database_arc: &Arc<Mutex<BTreeMap<String, Data>>>) -> Vec<String> {
    let db = (*database_arc).lock().unwrap();
    return (*db).keys().cloned().collect();
}

fn get_value(database_arc: &Arc<Mutex<BTreeMap<String, Data>>>, parameters: Vec<String>) -> Data {
    // Replace with for loop to handle multiple keys
    if parameters.len() == 1 {
        let db = database_arc.lock().unwrap();
        let result = (*db).get(&(parameters[0])); // Error check for n/a val
        match result {
            None => {
                return Data::Value("".to_string());
            }
            Some(_) => {
                return (*(result.unwrap())).clone();
            }
        }
    } else {
        // TODO hanlde case when multiple tree keys
        return Data::Value("subsequent btree access not developed.".to_string());
    }
}

fn set_value(database_arc: &mut Arc<Mutex<BTreeMap<String, Data>>>, parameters: SetParameters) -> String {
    let mut db = database_arc.lock().unwrap();
    let mut sub_db = &mut (*db);
    if parameters.btrees.len() > 0 {
        for tree in parameters.btrees{
            match (*sub_db).entry(tree.clone()){
                Occupied(entry) => {
                    match entry.into_mut() {
                        Data::Map(map) => {
                            sub_db = map;
                            continue;
                        }
                        Data::Value(_) => {
                            return format!("Error: Got value when expecting tree.\n");
                        }
                    }
                },
                Vacant(entry) => {
                    sub_db = &mut BTreeMap::<String, Data>::new();
                    entry.insert(Data::Map(*sub_db));
                    continue;
                }
            }
        }
    }
    // if parameters.btrees.len() > 0 {
    //     for i in 0..(parameters.btrees.len() - 1) {
    //         let keys: Vec<_> = (*sub_db).keys().cloned().collect();
    //         if keys.contains(&parameters.btrees[i]) {
    //             let result = (*sub_db).get(&(parameters.btrees[i])).unwrap(); // Error check for n/a val
    //             match result {
    //                 Data::Map(map) => {
    //                     sub_db = map;
    //                     continue;
    //                 }
    //                 Data::Value(_) => {
    //                     return format!("Error: Got value when expecting tree.\n");
    //                 }
    //             }
    //         } else {
    //             for j in i..(parameters.btrees.len() - 1) {
    //                 let new_tree: BTreeMap<String, Data> = BTreeMap::new();
    //                 (*sub_db).insert(
    //                     (*parameters.btrees[j]).to_string(),
    //                     Data::Map(new_tree),
    //                 );
    //             }
    //         }
    //     }
    // }
    // TODO Reformat to take into account multiple btree keys
    //println!("key: {}, value: {}, btrees: {:?}", parameters.key, parameters.value, parameters.btrees);
    let insert_result = (*sub_db).insert(parameters.key, Data::Value(parameters.value)); // TODO Add safety check for unwrap even though it should NEVER be None
    match insert_result {
        None => {
            return format!("Value set.");
        }
        Some(_) => {
            return format!("Value updated.");
        }
    }
}

fn remove_value(database_arc: &Arc<Mutex<BTreeMap<String, Data>>>, parameters: Vec<String>) -> String {
    if parameters.len() == 1 {
        let mut db = database_arc.lock().unwrap();
        let result = (*db).remove(&(parameters[0]));
        match result {
            None => {
                return format!("That key does not seem to exist!"); // Make error message more explicit.
            }
            Some(removed_data) => {
                match removed_data {
                    Data::Value(val) => {
                        return format!("Removed value: {}\n", val);
                    }
                    Data::Map(_) => {
                        return format!("Removed directory under: {}\n", &(parameters[0]));
                    }
                }
            }
        }
    }else{ // TODO: Fix to remove with multiple keys i.e. trees
        return format!("Error: Remove has more than 1 parameter!. Feature not implemented.")
    }
}

/*                                      STRING PARSER                                            */

enum Command {
    Keys,
    Get(Vec<String>),
    SetValue(SetParameters), // TODO make into a struct
    Remove(Vec<String>),
    Exit,
    Error(String),
}

struct SetParameters {
    key: String,
    value: String,
    btrees: Vec<String>,
}

fn parse_string(mut input: String) -> Command {
    trim_newline(&mut input);
    let mut input_vec = input
        .split(" ") // Split by spaces
        .map(ToString::to_string)
        .collect::<Vec<String>>();

    if input_vec.len() <= 0 {
        return Command::Error("Please input a command!".to_string());
    }

    match input_vec[0].to_lowercase().as_str() {
        "keys" => {
            return Command::Keys;
        }
        "get" => {
            if input_vec.len() != 2 {
                return Command::Error("Error: Get receives 1 parameter!".to_string());
            }
            let param_keys = input_vec[1]
                .split("/") // Split by slash
                .map(ToString::to_string)
                .collect::<Vec<String>>();
            return Command::Get(param_keys);
        }
        "setvalue" => {
            if input_vec.len() != 3 {
                return Command::Error("Error: SetValue receives 2 parameters!".to_string());
            }
            let mut key = "".to_string();
            let mut btrees = vec!();
            let value = (*input_vec[2]).to_string();
            if (input_vec[1]).contains("/") {
                match (input_vec[1]).pop() {
                    Some('/') => {
                        let param_directories = input_vec[1]
                            .split("/") // Split by slash
                            .map(ToString::to_string)
                            .collect::<Vec<String>>();
                        btrees = param_directories;
                        key = "".to_string();
                    },
                    Some(char) => {
                        input_vec[1].push(char);
                        let mut param_directories = input_vec[1]
                            .split("/") // Split by slash
                            .map(ToString::to_string)
                            .collect::<Vec<String>>();
                        key = param_directories.pop().unwrap();
                        btrees = param_directories;
                    },
                    None => {
                        return Command::Error("Error: Set command should have directory or value data!!".to_string());
                    }
                }
            }else{
                key = input_vec[1].clone();
            }
            let parsed_values = SetParameters {key, value, btrees};
            return Command::SetValue(parsed_values);
        }
        "remove" => {
            if input_vec.len() != 2 {
                return Command::Error("Error: Remove receives at least 1 parameter!".to_string());
            }
            let param_remove_key = vec![input_vec[1].to_string()];
            return Command::Remove(param_remove_key);
        }
        "exit" => {
            return Command::Exit;
        }
        _ => {
            return Command::Error("Error: Command does not exist.".to_string());
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
            let mut database_arc = Arc::clone(&database_arc);
            let (lines_tx, lines_rx) = LinesCodec::new().framed(socket).split();

            let responses = lines_rx.map(move |incomming_message| {
                let command = parse_string(incomming_message);
                match command {
                    Command::Error(msg) => {
                        return msg;
                    }
                    Command::Keys => {
                        let keys = get_keys(&database_arc);
                        return format!("The database keys are: {:?}\n", keys);
                    }
                    Command::Get(parameters) => {
                        let result = get_value(&database_arc, parameters.clone());
                        match result {
                            Data::Value(val) => {
                                if val.is_empty(){
                                    return format!("No values are stored for '{}'.\n", parameters[0]);
                                }
                                return format!("{}\n", val);
                            }
                            Data::Map(map) => {
                                let keys: Vec<_> = map.keys().cloned().collect();
                                return format!(
                                    "The values stored under {} are: {:?}\n",
                                    &(parameters[0]),
                                    keys
                                );
                            }
                        }
                    },
                    Command::SetValue(parameters) => {
                        let set_result = set_value(&mut database_arc, parameters);
                        return format!("{}", set_result);
                    }
                    // Command::NewDir(parameters) => {
                    //     let key = parameters.unwrap();
                    //     let mut db = database_arc.lock().unwrap();
                    //     let new_tree: BTreeMap<String, Data> = BTreeMap::new();
                    //     // TODO Reformat to take into account multiple btree keys
                    //     (*db).insert((*key[0]).to_string(), Data::Map(new_tree));
                    //     return format!("Newdir done.");
                    // }
                    Command::Remove(parameters) => {
                            let remove_result = remove_value(&database_arc, parameters);
                            return format!("{}", remove_result);
                    }
                    Command::Exit => {
                        std::process::exit(0);
                    }
                }
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
