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

fn set_value(database_arc: &Arc<Mutex<BTreeMap<String, Data>>>, parameters: Vec<String>) -> String {
    let mut db = database_arc.lock().unwrap();
    let mut sub_db: &BTreeMap<String, Data>;
    if parameters.len() > 2{
        for i in 0..(parameters.len() - 1) {
            let keys: Vec<_> = (*db).keys().cloned().collect();
            if keys.contains(&parameters[i]) {
                let result = (*db).get(&(parameters[i])).unwrap(); // Error check for n/a val
                match result {
                    Data::Map(map) => {
                        sub_db = map;
                        continue;
                    }
                    Data::Value(_) => {
                        return format!("Error: Got value when expecting tree.\n");
                    }
                }
            } else {
                for j in i..(parameters.len() - 1) {
                    let new_tree: BTreeMap<String, Data> = BTreeMap::new();
                    (*db).insert(
                        (*parameters[j]).to_string(),
                        Data::Map(new_tree),
                    );
                }
            }
        }
    }
    // TODO Reformat to take into account multiple btree keys
    let insert_result = (*db).insert((*parameters[0]).to_string(), Data::Value((*parameters[1]).to_string()));
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
    Get(Option<Vec<String>>),
    SetValue(Option<Vec<String>>),
    Remove(Option<Vec<String>>),
    Exit,
    Error(String),
}

fn parse_string(mut input: String) -> Command {
    trim_newline(&mut input);
    let input_vec = input
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
            return Command::Get(Some(param_keys));
        }
        "setvalue" => {
            if input_vec.len() != 3 {
                return Command::Error("Error: SetValue receives 2 parameters!".to_string());
            }
            let mut param_k_v = input_vec[1]
                .split("/") // Split by slash
                .map(ToString::to_string)
                .collect::<Vec<String>>();
            param_k_v.push((*input_vec[2]).to_string()); // TODO Process multiple keys
            println!("{:?}", param_k_v);
            return Command::SetValue(Some(param_k_v));
        }
        "remove" => {
            if input_vec.len() != 2 {
                return Command::Error("Error: Remove receives at least 1 parameter!".to_string());
            }
            let param_remove_key = vec![input_vec[1].to_string()];
            return Command::Remove(Some(param_remove_key));
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
            let database_arc = Arc::clone(&database_arc);
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
                    Command::Get(parameters) => match parameters {
                        None => {
                            return format!("That value does not exist!\n");
                        }
                        Some(param_unwrapped) => {
                            let result = get_value(&database_arc, param_unwrapped.clone());
                            match result {
                                Data::Value(val) => {
                                    if val.is_empty(){
                                        return format!("No values are stored for '{}'.\n", param_unwrapped[0]);
                                    }
                                    return format!("{}\n", val);
                                }
                                Data::Map(map) => {
                                    let keys: Vec<_> = map.keys().cloned().collect();
                                    return format!(
                                        "The values stored under {} are: {:?}\n",
                                        &(param_unwrapped[0]),
                                        keys
                                    );
                                }
                            }
                        }
                    },
                    Command::SetValue(parameters) => match parameters {
                        None => {
                            return format!("No parameters entered for setvalue!\n");
                        }
                        Some(param_unwrapped) => {
                            let set_result = set_value(&database_arc, param_unwrapped.clone());
                            return format!("{}", set_result);
                        }
                    }
                    // Command::NewDir(parameters) => {
                    //     let key = parameters.unwrap();
                    //     let mut db = database_arc.lock().unwrap();
                    //     let new_tree: BTreeMap<String, Data> = BTreeMap::new();
                    //     // TODO Reformat to take into account multiple btree keys
                    //     (*db).insert((*key[0]).to_string(), Data::Map(new_tree));
                    //     return format!("Newdir done.");
                    // }
                    Command::Remove(parameters) => match parameters {
                        None => {
                            return format!("No parameters entered for setvalue!\n");
                        }
                        Some(param_unwrapped) => {
                            let remove_result = remove_value(&database_arc, param_unwrapped.clone());
                            return format!("{}", remove_result);
                        }
                    }
                    Command::Exit => {
                        std::process::exit(0);
                    }
                }
                return format!("ERROR: Program reached end of command match without returning!"); // For debugging.
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
