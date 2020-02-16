extern crate tokio;

mod database;
use database::Database;
use database::SetParameters;
use database::Command;

// For parsing client input
use tokio::codec::Decoder;
use tokio::codec::LinesCodec;

// For Tokio server
use tokio::net::TcpListener;
use tokio::prelude::*;

/*                                      SERVER RESOURCES                                         */

fn main() {
    let addr = "127.0.0.1:6142".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    let db = Database::new();

    let server = listener
        .incoming()
        .map_err(|e| println!("failed to accept socket; error = {:?}", e))
        .for_each(move |socket| {
            let mut db = db.clone();

            let (lines_tx, lines_rx) = LinesCodec::new().framed(socket).split();

            let responses = lines_rx.map(move |incomming_message| {
                let query_result = db.send_db_command_get_reponse(parse_string(&incomming_message), true);
                return query_result;
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

/*                                      STRING PARSER                                        */

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

fn parse_string(input_string: &String) -> Command {
    let mut input = input_string.clone();
    trim_newline(&mut input);
    let input = input.chars().filter(|char| !char.is_control()).collect::<String>();
    let mut input_vec = input
        .split(" ") // Split by spaces
        .map(ToString::to_string)
        .collect::<Vec<String>>();

    if input_vec.len() <= 0 {
        return Command::Error("Please input a command!".to_string());
    }

    match input_vec[0].to_lowercase().as_str() {
        "get" => {
            if input_vec.len() == 1{
                return Command::Keys;
            }
            if input_vec.len() != 2 {
                return Command::Error("Error: Get receives 1 parameter!".to_string());
            }
            let param_keys = input_vec[1]
                .split("/") // Split by slash
                .map(ToString::to_string)
                .collect::<Vec<String>>();
            return Command::Get(param_keys);
        }
        "set" => {
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
            let parsed_values = SetParameters::new(key, value, btrees);
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
