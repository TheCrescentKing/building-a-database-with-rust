extern crate tokio;

use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::*;

fn main() {
    let addr = "127.0.0.1:6142".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    let server = listener
        .incoming()
        .map_err(|e| println!("failed to accept socket; error = {:?}", e))
        .for_each(move |socket| {
            let (reader, writer) = socket.split();
            let amount = io::copy(reader, writer);
            let msg = amount.then(|result| {
                match result {
                    Ok((amount, _, _)) => println!("Wrote {} bytes", amount),
                    Err(e) => println!("Error: {}", e),
                }
                Ok(())
            });

            tokio::spawn(msg);
            Ok(())
        });

    println!("Server running on {}", addr);

    tokio::run(server);
}
