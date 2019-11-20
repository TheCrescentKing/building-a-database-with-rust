extern crate tokio;

use std::sync::Arc;
use std::sync::Mutex;

use tokio::codec::Decoder;
use tokio::codec::LinesCodec;

//use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::*;

fn main() {
    let addr = "127.0.0.1:6142".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    let counter = Arc::new(Mutex::new(BTreeMap::new(BTreeMap<&str, &str>)));

    let server = listener
        .incoming()
        .map_err(|e| println!("failed to accept socket; error = {:?}", e))
        .for_each(move |socket| {
            let counter = Arc::clone(&counter);
            let (lines_tx, lines_rx) = LinesCodec::new().framed(socket).split();

            let responses = lines_rx.map(move |incomming_message| {
                match incomming_message.as_ref() {
                    "insert" => {
                        let value = counter.lock().unwrap();
                        return format!("The counter reads: {}\n", *value);
                    }
                    _ => {
                        return format!("The commands are: set, get, remove, listkeys.\n");
                    }
                }
                //return incomming_message;
            });

            let writes = responses.fold(lines_tx, |writer, response| {
                //Return the future that handles to send the response to the socket
                writer.send(response)
            });

            tokio::spawn(writes.then(move |_| Ok(())));

            /*
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
            */
            Ok(())
        });

    println!("Server running on {}", addr);

    tokio::run(server);
}
