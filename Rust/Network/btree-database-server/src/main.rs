extern crate tokio;

// Imports for database reference counter, mutex and lock
use std::sync::Arc;
use std::sync::Mutex;

// For parsing client input
use tokio::codec::Decoder;
use tokio::codec::LinesCodec;

// For Tokio server
use tokio::net::TcpListener;
use tokio::prelude::*;

// import BTreeMap
use std::collections::BTreeMap;
use std::collections::btree_map::Entry::*;

// Imports for file writing
use std::fs::OpenOptions;

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

fn get_value(database_arc: &Arc<Mutex<BTreeMap<String, Data>>>, parameters: Vec<String>) -> Result<Data, String> {

    let db = database_arc.lock().unwrap();
    let mut sub_db = &(*db);
    if parameters.len() > 0{
        for i in 0..(parameters.len() -1){
            let key = &(parameters[i]);
            let result = (*sub_db).get(key);
            match result{
                None => {
                    return Err(format!("Error: the tree for key {}, does not exist.", key));
                }
                Some(data) => {
                    match data {
                        Data::Value(_) => {
                            return Err(format!("Error: the key {}, is a value and not a tree!", key));
                        }
                        Data::Map(map) => {
                            sub_db = map;
                            continue;
                        }
                    }
                }
            }
        }
    }

    let key = &parameters[parameters.len()-1];

    match (*sub_db).get(key) {
        None => {
            return Err(format!("Error: the key {}, does not have a value.", key));
        }
        Some(data) => {
            return Ok(data.clone());
        }
    }
}

fn set_value(database_arc: &mut Arc<Mutex<BTreeMap<String, Data>>>, parameters: SetParameters) -> String {
    let mut db = database_arc.lock().unwrap();
    let mut sub_db = &mut (*db); // BTree variable for loop where we borrow the root as mutable
    if parameters.btrees.len() > 0 {
        for tree in parameters.btrees{ // Loop through all the BTree keys
            match (*sub_db).entry(tree){ // Use Entry API Pattern
                Occupied(entry) => { //  If the BTree exists set the loop variable to it
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
                Vacant(entry) => { // If the BTree does not exist create a new one and point the loop variable to it
                    match entry.insert(Data::Map(BTreeMap::<String, Data>::new())){
                        Data::Map(map) => {
                            sub_db = map;
                            continue;
                        }
                        Data::Value(_) => {
                            return format!("Error: Got value when expecting tree.\n");
                        }
                    }
                }
            }
        }
    }

    // Finally, add the key Value pair the the desired BTree represented by sub_db
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

    let mut db = database_arc.lock().unwrap();
    let mut sub_db = &mut (*db);
    if parameters.len() > 0{
        for i in 0..(parameters.len() -1){
            let key = &(parameters[i]);
            let result = (*sub_db).get_mut(key);
            match result{
                None => {
                    return format!("Error: the tree for key {}, does not exist.", key);
                }
                Some(data) => {
                    match data {
                        Data::Value(_) => {
                            return format!("Error: the key {}, is a value and not a tree!", key);
                        }
                        Data::Map(map) => {
                            sub_db = map;
                            continue;
                        }
                    }
                }
            }
        }
    }

    let key = &parameters[parameters.len()-1];

    println!("Key: {}, TreeKeys: {:?}", key, (*sub_db).keys());

    match (*sub_db).remove(key) {
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
            let key: String;
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
                return Command::Error("Error: Remove receives 1 parameter!".to_string());
            }
            let param_keys = input_vec[1]
                .split("/") // Split by slash
                .map(ToString::to_string)
                .collect::<Vec<String>>();
            return Command::Remove(param_keys);
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

/*                                      FILE WRITER                                              */

fn save_to_file(string_to_save: &String, name_of_file: &str) {
    let mut file = OpenOptions::new()
    .append(true)
    .create(true)
    .open(name_of_file)
    .unwrap();

    if let Err(e) = writeln!(file, "{}", string_to_save) {
        eprintln!("Couldn't write to log-file: {}", e);
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
                let command = parse_string(incomming_message.clone());
                match command {
                    Command::Error(msg) => {
                        return msg;
                    }
                    Command::Keys => {
                        save_to_file(&incomming_message, "log.txt");

                        let keys = get_keys(&database_arc);
                        return format!("The database keys are: {:?}\n", keys);
                    }
                    Command::Get(parameters) => {
                        save_to_file(&incomming_message, "log.txt");

                        let result = get_value(&database_arc, parameters.clone());
                        match result {
                            Err(error_string) => {
                                return format!("{}", error_string);
                            },
                            Ok(data) => {
                                match data {
                                    Data::Value(val) => {
                                        return format!("{}\n", val);
                                    }
                                    Data::Map(map) => {
                                        let keys: Vec<_> = map.keys().cloned().collect();
                                        return format!("The keys of the requested tree are: {:?}\n", keys);
                                    }
                                }
                            }
                        }
                    },
                    Command::SetValue(parameters) => {
                        save_to_file(&incomming_message, "log.txt");

                        let set_result = set_value(&mut database_arc, parameters);
                        return format!("{}", set_result);
                    }
                    Command::Remove(parameters) => {
                        save_to_file(&incomming_message, "log.txt");

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
