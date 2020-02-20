//! A "hello world" echo server with Tokio
//!
//! This server will create a TCP listener, accept connections in a loop, and
//! write back everything that's read off of each TCP connection.
//!
//! Because the Tokio runtime uses a thread pool, each TCP connection is
//! processed concurrently with all other TCP connections across multiple
//! threads.
//!
//! To see this server in action, you can run this in one terminal:
//!
//!     cargo run --example echo
//!
//! and in another terminal you can run:
//!
//!     cargo run --example connect 127.0.0.1:8080
//!
//! Each line you type in to the `connect` terminal should be echo'd back to
//! you! If you open up multiple terminals running the `connect` example you
//! should be able to see them all make progress simultaneously.

#![warn(rust_2018_idioms)]

use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use std::env;
use std::error::Error;

mod database;
use database::Database;
use database::SetParameters;
use database::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Allow passing an address to listen on as the first argument of this
    // program, but otherwise we'll just set up our TCP listener on
    // 127.0.0.1:8080 for connections.
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6142".to_string());

    // Next up we create a TCP listener which will listen for incoming
    // connections. This TCP listener is bound to the address we determined
    // above and must be associated with an event loop.
    let mut listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    let db = Database::new();

    loop {
        // Asynchronously wait for an inbound socket.
        let (mut socket, _) = listener.accept().await?;

        // And this is where much of the magic of this server happens. We
        // crucially want all clients to make progress concurrently, rather than
        // blocking one on completion of another. To achieve this we use the
        // `tokio::spawn` function to execute the work in the background.
        //
        // Essentially here we're executing a new task to run concurrently,
        // which will allow all of our clients to be processed concurrently.

        let mut db = db.clone();

        tokio::spawn(async move {

            let mut incoming_message: String = "".to_string();

            // let size = 1024;
            // let mut buf: Box<[u8]> = vec![0; size].into_boxed_slice();

            // In a loop, read data from the socket and write the data back.
            loop {
                let mut buf = [0; 1024];

                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    return;
                }

                let mut query_result: String = "".to_string();

                match &String::from_utf8(buf.to_vec()){
                    Err(utf8_error) => {
                        query_result = format!("Error: The command contained invalid text data. Details: {:?}", utf8_error.utf8_error());
                    },
                    Ok(result) =>{
                        let result = result
                                        .chars()
                                        .filter(|c| *c != '\u{0}')
                                        .collect::<String>();
                        incoming_message.push_str(&result);
                        match incoming_message.chars().last(){
                            None => {
                                query_result = format!("Error: No data was received from the socket.");
                            },
                            Some(char) =>{
                                if char == '\n'{
                                    query_result = format!("{}\n", db.send_db_command_get_reponse(parse_string(&incoming_message), true));
                                    incoming_message.clear();
                                }
                            }
                        }
                    }
                }

                socket
                    // .write_all(&buf[0..n])
                    .write_all(query_result.as_bytes())
                    .await
                    .expect("failed to write data to socket");
            }
        });
    }
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
    println!("{:?}", input_string);
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
