
use std::env;

use tokio::runtime::Builder;

mod client;

fn main() -> std::result::Result<(), std::boxed::Box<std::io::Error>> {

    // build runtime
    let mut rt = Builder::new()
    .threaded_scheduler()
    .enable_all()
    .build()
    .unwrap();

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6142".to_string());

    rt.block_on(async {
        let client_one = tokio::spawn(client::main(addr.clone(), 3));
        // let client_two = tokio::spawn(client::main(addr.clone(), 4));

        let client_one = client_one.await.expect("Client handle panicked.");
        // let client_two = client_two.await.expect("Client handle panicked.");
        match client_one{
            Ok(()) => {
                println!("All tests completed successfully.");
            },
            Err(error) => {
                println!("Some error has occurred!");
                eprintln!("{:?}", error);
            }
        }
        // match client_two{
        //     Ok(()) => {
        //         println!("All tests completed successfully.");
        //     },
        //     Err(error) => {
        //         println!("Some error has occurred!");
        //         eprintln!("{:?}", error);
        //     }
        // }
        Ok(())
    })
}
