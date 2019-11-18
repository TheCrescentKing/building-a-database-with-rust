extern crate tokio;

use tokio::codec::Decoder;
use tokio::codec::LinesCodec;

use tokio::net::TcpListener;
use tokio::prelude::*;

fn main() {
    let addr = "127.0.0.1:6142".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    let server = listener
        .incoming()
        .map_err(|e| println!("failed to accept socket; error = {:?}", e))
        .for_each(move |socket| {
            let (lines_tx, lines_rx) = LinesCodec::new().framed(socket).split();

            let responses = lines_rx.map(move |incomming_message| {
                return incomming_message;
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
