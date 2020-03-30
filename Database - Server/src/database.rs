use std;

// Imports for database reference counter, mutex and lock
use std::sync::Arc;
use std::sync::RwLock;

// import BTreeMap
use std::collections::BTreeMap;
use std::collections::btree_map::Entry::*;

// Imports for file reading/writing
use std::{
    fs,
    fs::File,
    io::{prelude::*, BufReader, Error},
    path::{Path, PathBuf},
};

use std::fs::OpenOptions;

/*                                      CONSTANTS                                                */
static LOG_FILE: &str = "log.txt";


#[derive(Clone)]
pub enum Data {
    Map(BTreeMap<String, Data>),
    Value(String),
}

type DBSignature = BTreeMap<String, Data>;

pub struct SetParameters {
    key: String,
    value: String,
    btrees: Vec<String>,
}

pub enum Command {
    Keys,
    Get(Vec<String>),
    SetValue(SetParameters),
    Remove(Vec<String>),
    Exit,
    ResetLog,
    Error(String),
}

impl SetParameters {

    pub fn new(key: String, value: String, btrees: Vec<String>) -> SetParameters{
        SetParameters{
            key: key,
            value: value,
            btrees: btrees,
        }
    }

    pub fn get_key(&self) -> String{
        self.key.clone()
    }
    pub fn get_value(&self) -> String{
        self.value.clone()
    }
    pub fn get_btrees(&self) -> Vec<String>{
        self.btrees.clone()
    }
}

pub struct Database {
    database_arc: Arc<RwLock<DBSignature>>,
    log_file_arc: Arc<RwLock<File>>,
    log_path: PathBuf,
}

impl Database {

    pub fn new(reset_log: bool, log_dir: &str) -> Database{
        let log_path = Path::new(log_dir);
        if !log_path.is_dir(){
            println!("Error: The directory for storing the log '{}' does not exist.", log_dir);
            std::process::exit(-1);
        }
        let log_path_buf = log_path.join(LOG_FILE);
        let log_path = log_path_buf.as_path();
        if reset_log && log_path.is_file(){
            let result : std::io::Result<()> = fs::remove_file(log_path);
            match result{
                Err(_)=>{
                    println!("Error: Removing the log file failed.");
                    std::process::exit(-1);
                }
                _ =>{
                }
            }
        }
        let mut db = Database{
            database_arc: Arc::new(RwLock::new(DBSignature::new())),
            log_file_arc: Arc::new(RwLock::new(OpenOptions::new()
                .read(true)
                .append(true)
                .create(true)
                .open(log_path)
                .unwrap())),
            log_path: log_path.to_owned(),
            };
        db.restore_from_log();
        db
    }

    fn save_to_log(&mut self, string_to_save: &String) {
        let mut log_file = (*self.log_file_arc).write().unwrap();
        if let Err(e) = writeln!(log_file, "{}", string_to_save) { // Re create file if deleted while running
            eprintln!("Couldn't write to log-file: {}", e);
        }else{
            // log_file.flush().unwrap();
            // println!("{:?}", result);
            log_file.sync_data().unwrap();
        }
    }

    pub fn clone(&self) -> Database{
        Database{
            database_arc: self.database_arc.clone(),
            log_file_arc: self.log_file_arc.clone(),
            log_path: self.log_path.clone(),
        }
    }

    // Accessors & Mutators

    fn get_keys(&self) -> Vec<String> {
        let db = (*self.database_arc).read().unwrap();
        return (*db).keys().cloned().collect();
    }

    fn get_value(&self, parameters: Vec<String>) -> Result<Data, String> {

        let db = (*self.database_arc).read().unwrap();
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
                return Err(format!("Error: Value not found under {}", key));
            }
            Some(data) => {
                return Ok(data.clone());
            }
        }
    }

    fn set_value(&mut self, parameters: SetParameters, save_log_flag: bool) -> String {

        if save_log_flag{
            let log_string;
            if parameters.get_btrees().len() > 0{
                log_string = format!("SET {} {} {}", parameters.get_btrees().join("/"), parameters.get_key(), parameters.get_value());
            }else{
                log_string = format!("SET {} {}", parameters.get_key(), parameters.get_value());
            }
            self.save_to_log(&log_string);
        }


        let mut db = (*self.database_arc).write().unwrap();
        let mut sub_db = &mut (*db); // BTree variable for loop where we borrow the root as mutable
        let btree_vec = parameters.get_btrees();
        if btree_vec.len() > 0 {
            for tree in btree_vec{ // Loop through all the BTree keys
                match (*sub_db).entry(tree){ // Use Entry API Pattern
                    Occupied(entry) => { //  If the BTree exists set the loop variable to it
                        match entry.into_mut() {
                            Data::Map(map) => {
                                sub_db = map;
                                continue;
                            }
                            Data::Value(_) => {
                                return format!("Error: Got value when expecting tree.");
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
                                return format!("Error: Got value when expecting tree.");
                            }
                        }
                    }
                }
            }
        }

        // Finally, add the key Value pair the the desired BTree represented by sub_db
        /*let insert_result = */(*sub_db).insert(parameters.get_key(), Data::Value(parameters.get_value())); // TODO Add safety check for unwrap even though it should NEVER be None
        format!("Ok")
    }

    fn remove_value(&mut self, parameters: Vec<String>, save_log_flag: bool) -> String {

        if save_log_flag{
            let log_string = format!("REM {}", parameters.join("/"));
            self.save_to_log(&log_string);
        }

        let mut db = (*self.database_arc).write().unwrap();
        let mut sub_db = &mut (*db);
        if parameters.len() > 0{
            for i in 0..(parameters.len() -1){
                let key = &(parameters[i]);
                let result = (*sub_db).get_mut(key);
                match result{
                    None => {
                        return format!("Error: the tree for key {}, does not exist.", key); // Maybe remove last log line if this happens
                    }
                    Some(data) => {
                        match data {
                            Data::Value(_) => {
                                return format!("Error: the key {}, is a value and not a tree!", key); // Maybe remove last log line if this happens
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

        match (*sub_db).remove(key) {
            None => {
                return format!("That key does not seem to exist!"); // Make error message more explicit. And remove last log line
            }
            Some(removed_data) => {
                match removed_data {
                    Data::Value(val) => {
                        return format!("{}", val);
                    }
                    Data::Map(_) => {
                        return format!("{}", &(parameters[0]));
                    }
                }
            }
        }
    }

    fn remove_log(&self) -> Result<(), Error>{
         return fs::remove_file(&self.log_path)
    }

    fn remove_empty_values(parameters: Vec<String>) -> Vec<String>{
        parameters.into_iter().filter(|string| !string.is_empty())
                    .collect::<Vec<String>>()
    }

    pub fn send_db_command_get_reponse(&mut self, command: Command,  save_log_flag: bool) -> String{
        let query_result: String;

        match command {
            Command::Error(msg) => {
                query_result = msg;
            }
            Command::Keys => {
                let keys = self.get_keys();
                query_result = format!("The database keys are: {:?}", keys);
            }
            Command::Get(parameters) => {
                // println!("{:?}", parameters);
                let parameters = Self::remove_empty_values(parameters.clone());
                let result = self.get_value(parameters.clone());
                match result {
                    Err(error_string) => {
                        query_result = format!("{}", error_string);
                    },
                    Ok(data) => {
                        match data {
                            Data::Value(val) => {
                                query_result = format!("{}", val);
                            }
                            Data::Map(map) => {
                                let keys: Vec<_> = map.keys().cloned().collect();
                                query_result = format!("The keys of the requested tree are: {:?}", keys);
                            }
                        }
                    }
                }
            },
            Command::SetValue(parameters) => {
                let set_result = self.set_value(parameters, save_log_flag);
                query_result = format!("{}", set_result);
            }
            Command::Remove(parameters) => {
                let remove_result = self.remove_value(parameters, save_log_flag);
                query_result = format!("{}", remove_result);
            }
            Command::ResetLog =>{
                let result = self.remove_log();
                match result{
                    Err(err)=>{
                        query_result = format!("Error: Removing the log file failed, reason: {}", err);
                    }
                    Ok(()) =>{
                        query_result = format!("Ok, log cleaned.");
                    }
                }
            }
            Command::Exit => {
                std::process::exit(0);
            }
        }

        return query_result;
    }

    // Restore from log
    fn restore_from_log(&mut self){
        let log_file = self.log_file_arc.read().unwrap();
        let file_lines = lines_from_file(&log_file);
        drop(log_file);
        if !file_lines.is_empty() {
            for line in file_lines{
                let command = parse_log_line(line.clone());
                self.send_db_command_get_reponse(command, false);
            }
        }
    }
}

/*                                      FILE READER                                              */

fn lines_from_file(file: &File) -> Vec<String> {
    let buf = BufReader::new(file);
    buf.lines()
        .map(|l| l.expect("Could not parse line"))
        .collect()
}

fn parse_log_line(mut log_line: String) -> Command{
    // Remove trailing white space
    if log_line.ends_with('\n') {
        log_line.pop();
        if log_line.ends_with('\r') {
            log_line.pop();
        }
    }

    let mut input_vec = log_line
        .split(" ") // Split by spaces
        .map(ToString::to_string)
        .collect::<Vec<String>>();

    match input_vec.remove(0).as_str(){
        "SET" =>{
            let key: String;
            let mut btrees = vec!();
            let value: String = input_vec.pop().unwrap();
            if input_vec[0].contains("/"){
                let param_directories = input_vec[0]
                    .split("/") // Split by slash
                    .map(ToString::to_string)
                    .collect::<Vec<String>>();
                btrees = param_directories;
                key = input_vec.remove(1);
            } else{
                key = input_vec.remove(0);
            }
            let parsed_values = SetParameters::new(key, value, btrees);
            return Command::SetValue(parsed_values);
        },
        "REM" => {
            let param_keys = input_vec[0]
                .split("/") // Split by slash
                .map(ToString::to_string)
                .collect::<Vec<String>>();
            return Command::Remove(param_keys);
        },
        _ => {
            return Command::Error("Error: Command not found while parsing log!".to_string());
        }
    }
}
