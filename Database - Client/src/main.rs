#![warn(rust_2018_idioms)]

use std::str;
use std::io::Error;

use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use std::env;
use std::time::{Instant};

use rand::{thread_rng, Rng};
use rand::distributions::{Alphanumeric, Uniform};


#[tokio::main]
async fn main() -> Result<(), Box<Error>> {

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6142".to_string());

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

    short_test_all_commands(&mut socket).await;
    // multiple_set_commands(&mut socket, 1000).await;

    Ok(())
}

async fn multiple_set_commands(mut socket: &mut TcpStream, number: u128){

    let mut sum_of_times = 0;

    let start_time = Instant::now();

    for _i in 0..number{

        let first_time = Instant::now();

        let range = Uniform::new(5, 100);

        let x = thread_rng().sample(range);
        let key: String = thread_rng().sample_iter(Alphanumeric).take(x).collect();
        let x = thread_rng().sample(range);
        let value: String = thread_rng().sample_iter(Alphanumeric).take(x).collect();

        let command = format!("set {} {};\n", key, value);


        socket.write_all(command.as_bytes()).await.expect("failed to write data to socket");
        let _response: String = read_from_socket(&mut socket).await;

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
