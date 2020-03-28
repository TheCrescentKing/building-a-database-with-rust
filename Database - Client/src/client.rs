#![warn(rust_2018_idioms)]

use std::str;
use std::thread;
use std::io::Error;

use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use std::time::{Instant};

use rand::{thread_rng, Rng};
use rand::distributions::{Alphanumeric, Uniform};

use std::{
    fs,
    fs::OpenOptions
};
use std::io::Write;


pub async fn main(addr: String, command: u8) -> Result<(), Box<Error>> {

    let _result : std::io::Result<()> = fs::remove_file("output.txt");

    // let addr = env::args()
    //     .nth(1)
    //     .unwrap_or_else(|| "127.0.0.1:6142".to_string());

    // let addr = "127.0.0.1:6142".to_string();

    let mut socket;
    match TcpStream::connect(&addr).await{
        Ok(s)=>{
            socket = s;
        }
        Err(error)=>{
            return Err(Box::new(error));
        }
    }

    // socket.write_all("ç".as_bytes()).await.expect("failed to write data to socket");

    // socket.write_all("resetlog;\n".as_bytes()).await.expect("failed tso write data to socket");
    //
    // let response: String = read_from_socket(&mut socket).await;
    // assert_eq!(true, response.contains("Ok"));

    match command{
        1 => {
            short_test_all_commands(&mut socket).await;
        },
        2 => {
            multiple_set_commands(&mut socket, 100).await;
        },
        3 =>{
            change_name(&mut socket, "Name", "Bob").await;
        },
        4 =>{
            change_name(&mut socket, "Name", "Alice").await;
        }
        _ =>{}
    }

    Ok(())
}

async fn multiple_set_commands(mut socket: &mut TcpStream, number: u128){
    let mut sum_of_times = 0;

    let start_time = Instant::now();

    for _i in 0..number{

        let first_time = Instant::now();

        let range = Uniform::new(5, 10);

        let x = thread_rng().sample(range);
        let key: String = thread_rng().sample_iter(Alphanumeric).take(x).collect();
        let x = thread_rng().sample(range);
        let value: String = thread_rng().sample_iter(Alphanumeric).take(x).collect();

        let command = format!("set {} {};\n", key, value);
        // println!("{:?}", command);
        save_to_file(&command);

        socket.write_all(command.as_bytes()).await.expect("failed to write data to socket");
        let response: String = read_from_socket(&mut socket).await;
        // println!("Client received {:?}", response);
        save_to_file(&response);

        let second_time = Instant::now();
        let time_taken = second_time.duration_since(first_time).as_millis();
        // println!("Iteration {}, time taken {}ms", i, time_taken);
        sum_of_times += time_taken;
    }

    let end_time = Instant::now();

    let total_time = end_time.duration_since(start_time).as_secs();

    println!("Client report: Time taken to complete the average test was {}ms. Total time taken {}s", (sum_of_times/number), total_time);

}

async fn short_test_all_commands(mut socket: &mut TcpStream){
    let mut sum_of_times = 0;

    for _i in 0..10{

        let start_time = Instant::now();

        socket.write_all("set Name Rustacean;\n".as_bytes()).await.expect("failed tso write data to socket");

        let response: String = read_from_socket(&mut socket).await;
        assert_eq!(true, response.contains("Ok"));

        socket.write_all("get Name;\n".as_bytes()).await.expect("failed to read data from socket");
        let response: String = read_from_socket(&mut socket).await;
        assert_eq!(true, response.contains("Rustacean"));

        socket.write_all("remove Name;\n".as_bytes()).await.expect("failed to write data to socket");
        let response: String = read_from_socket(&mut socket).await;
        // println!("line 102 response {:?}", response);
        assert_eq!(true, response.contains("Rustacean"));

        socket.write_all("get Name;\n".as_bytes()).await.expect("failed to read data from socket");
        let response: String = read_from_socket(&mut socket).await;
        assert_eq!(true, (response.contains("not found") && response.contains("Name")));

        let end_time = Instant::now();

        // println!("Loop {} time {:?}", i, end_time.duration_since(start_time).as_millis());

        sum_of_times += end_time.duration_since(start_time).as_millis();
    }

    println!("Client report: Time taken to complete tests is {}", (sum_of_times/10));
}

async fn change_name(mut socket: &mut TcpStream, key: &str, value: &str){
    for _i in 0..10{
        let command = format!("set {} {};\n", key, value);
        socket.write_all(command.as_bytes()).await.expect("failed to write data to socket");
        let _response: String = read_from_socket(&mut socket).await;
        let command = "get Name;\n";
        thread::sleep_ms(100);
        socket.write_all(command.as_bytes()).await.expect("failed to write data to socket");
        let response: String = read_from_socket(&mut socket).await;
        println!("Value: {}, response: {}", value, response);
        assert_eq!(true, response.contains(value));
    }
}

async fn read_from_socket(socket: &mut TcpStream) -> String{
    let mut read_buf = [0; 1024];
    socket
        .read(&mut read_buf)
        .await
        .expect("failed to read data from socket");
    let response = str::from_utf8(&read_buf).unwrap();
    let response =  response.chars()
        .filter(|c| *c != '\u{0}')
        .collect::<String>();
    response
}

fn save_to_file(string_to_save: &String) {
    let mut output_file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open("output.txt")
        .unwrap();
    if let Err(e) = write!(output_file, "{}", string_to_save) { // Re create file if deleted while running
        eprintln!("Couldn't write to log-file: {}", e);
    }else{
        // log_file.flush().unwrap();
        // println!("{:?}", result);
        // output_file.sync_data().unwrap();
    }
}
