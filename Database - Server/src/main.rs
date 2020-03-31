#![warn(rust_2018_idioms)]

use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use std::env;
use std::error::Error;

mod database;
use database::Database;
use database::SetParameters;
use database::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {


    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6142".to_string());

    // let addr = "127.0.0.1:6142".to_string();

    let mut listener = TcpListener::bind(&addr).await?;

    println!("Listening on: {}", addr);

    let db = Database::new(false, "./");

    loop {
        // Asynchronously wait for an inbound socket.
        let (mut socket, _) = listener.accept().await?;

        let mut db = db.clone();

        tokio::spawn(async move {

            let mut incoming_message: String = "".to_string();
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {

                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    return;
                }

                let mut buf_vec = buf.to_vec();
                buf_vec.truncate(n);

                // println!("{:?}", buf_vec);

                // match &String::from_utf8(buf_vec){
                //     Err(utf8_error) => {
                //         let query_result = format!("Error: The command contained invalid text data. Details: {:?}", utf8_error.utf8_error());
                //         socket
                //             .write_all(query_result.as_bytes())
                //             .await
                //             .expect("failed to write data to socket");
                //     },
                //     Ok(result) =>{
                //         // println!("Before filtering buffer: {}", result);
                //         parse_buffer(&result, &mut incoming_message, &mut socket, &mut db).await;
                //     }
                // }

                let result = String::from_utf8_lossy(&buf_vec).into_owned();
                // println!("Before filtering buffer: {}", result);
                parse_buffer(&result, &mut incoming_message, &mut socket, &mut db).await;

            }
        });
    }
}

/*                                      STRING PARSER                                        */

async fn parse_buffer(result : &String, incoming_message: &mut String, socket: &mut TcpStream, db: &mut Database){
    let mut result = result.clone();
    trim_newline(&mut result);
    // println!("After filtering buffer: {:?}", result);

    if result.ends_with(";/"){ // Meaning we have transaction chain so must wait until last command
        result.pop();
        incoming_message.push_str(&result);
        socket
            .write_all("Ok".as_bytes())
            .await
            .expect("failed to write data to socket");
    }else if result.ends_with(";"){ // We can process the command(s)
        let mut commands: Vec<String> = vec![];
        if result.chars().filter(|c| *c == ';').count() > 1{ // If we have multiple commmands
            // Separate by semicolon if there are multiple commands
            commands = result.split(";").map(ToString::to_string).collect::<Vec<String>>();
        }else{
            result.pop(); // Remove end semicolon before passing to the string_to_command function
            commands.push(result);
        }
        // Send command(s) to database
        for command in commands{
            println!("Intpreted command in loop: {:?}", command);
            let query_result = format!("{}", db.send_db_command_get_reponse(string_to_command(&command), true));
            socket
                .write_all(query_result.as_bytes())
                .await
                .expect("failed to write data to socket");
        }
        incoming_message.clear();
    }else{ // Buffer was too small so rest of command in next buffer
        incoming_message.push_str(&result);
    }

    println!("Incoming message= {:?}", incoming_message);

    // if result.contains(";"){ // Check if we have semicolon to signify end of commands
    //     let commands: Vec<String>;
    //     if result.chars().filter(|c| *c == ';').count() > 1{ // If we have multiple commmands
    //         result = result.chars()
    //             .filter(|c| *c != '\n') // Separate by newlines if there are multiple commands
    //             .collect::<String>();
    //         commands = result.split(";").map(ToString::to_string).collect::<Vec<String>>();
    //     }else{
    //         match result.find(';'){
    //             Some(index) => {
    //                 result.split_off(index);
    //             }
    //             None => {
    //                 return;
    //             }
    //         }
    //         commands = result.split("\n").map(ToString::to_string).collect::<Vec<String>>();
    //     }
    //     incoming_message.push_str(&result); // why is this here, should it be above in the else close for non?
    //     for command in commands{
    //         let query_result = format!("{}", db.send_db_command_get_reponse(string_to_command(&command), true));
    //         socket
    //             .write_all(query_result.as_bytes())
    //             .await
    //             .expect("failed to write data to socket");
    //     }
    //     incoming_message.clear();
    // }else{
    //     incoming_message.push_str(&result);
    // }
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

fn string_to_command(input_string: &String) -> Command {
    // println!("Parsing command: {:?}", input_string);
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
        "resetlog" => {
            return Command::ResetLog;
        }
        "exit" => {
            return Command::Exit;
        }
        _ => {
            return Command::Error("Error: Command does not exist.".to_string());
        }
    }
}
