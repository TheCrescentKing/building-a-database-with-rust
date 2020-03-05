// #![warn(rust_2018_idioms)]

use tokio::runtime::Builder;

mod server;
mod client;

fn main() -> std::result::Result<(), std::boxed::Box<std::io::Error>> {

    // build runtime
    let mut rt = Builder::new()
    .threaded_scheduler()
    .enable_all()
    .build()
    .unwrap();

    rt.block_on(async {
        let _server_handle = tokio::spawn(server::main());
        let client_handle = tokio::spawn(client::main());

        // let server_result = server_handle.await.expect("Server handle panicked.");
        let client_result = client_handle.await.expect("Client handle panicked.");
        match client_result{
            Ok(()) => {
                println!("All tests completed successfully.");
                Ok(())
            },
            Err(error) => {
                println!("Some error has occurred!");
                Err(error)
            }
        }
    })
}
