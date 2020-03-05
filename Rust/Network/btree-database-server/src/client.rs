#![warn(rust_2018_idioms)]

use std::str;

use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use std::time::{Duration, Instant};


pub async fn main() -> std::result::Result<(), std::boxed::Box<std::io::Error>> {

    // let addr = env::args()
    //     .nth(1)
    //     .unwrap_or_else(|| "127.0.0.1:6142".to_string());

    let addr = "127.0.0.1:6142".to_string();

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

    let mut sum_of_times = 0;

    for i in 0..10{

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

        println!("Loop {} time {:?}", i, end_time.duration_since(start_time).as_millis());

        sum_of_times += end_time.duration_since(start_time).as_millis();
    }

    println!("Client report: Time taken to complete tests is {}", (sum_of_times/10));

    Ok(())
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
