#![warn(rust_2018_idioms)]

use std::str;

use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use std::env;
use std::error::Error;


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
    let mut socket = TcpStream::connect(&addr).await?;

    socket.write_all("ç".as_bytes()).await.expect("failed to write data to socket");

    // socket.write_all("set Name Sephiroth;\n".as_bytes()).await.expect("failed to write data to socket");
    //
    // let response: String = read_from_socket(&mut socket).await;
    // assert_eq!(true, response.contains("Ok"));
    //
    // socket.write_all("get Name;\n".as_bytes()).await.expect("failed to read data from socket");
    // let response: String = read_from_socket(&mut socket).await;
    // assert_eq!(true, response.contains("Sephiroth"));
    //
    // socket.write_all("remove Name;\n".as_bytes()).await.expect("failed to write data to socket");
    // let response: String = read_from_socket(&mut socket).await;
    // assert_eq!(true, response.contains("Sephiroth"));
    //
    // socket.write_all("get Name;\n".as_bytes()).await.expect("failed to read data from socket");
    // let response: String = read_from_socket(&mut socket).await;
    // assert_eq!(true, (response.contains("not found") && response.contains("Name")));

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
