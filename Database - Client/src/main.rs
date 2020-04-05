
use std::env;
use std::io;

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

        // println!("Welcome to the test client, please type: ");
        // println!("1: To type input for the server directly into the console.");
        // println!("2: To do a short test of the get, set and remove functions.");
        // println!("3: To do multiple iterations of random set commands. (Iteration number is defined in the code)");
        // println!("4: To send the png file found in the client folder to the server.");
        // println!("5: To run the Bob-Alice test where 2 clients set those values to the same key.");

        // let mut test_number = read_number_from_console();

        let mut test_number = 1;

        let client_one = tokio::spawn(client::main(addr.clone(), test_number));
        if test_number == 5{
            test_number += 1;
        }
        let client_two = tokio::spawn(client::main(addr.clone(), test_number));

        let client_one = client_one.await.expect("Client handle panicked.");
        let client_two = client_two.await.expect("Client handle panicked.");
        match client_one{
            Ok(()) => {
                println!("All tests completed successfully.");
            },
            Err(error) => {
                println!("Some error has occurred!");
                eprintln!("{:?}", error);
            }
        }
        match client_two{
            Ok(()) => {
                println!("All tests completed successfully.");
            },
            Err(error) => {
                println!("Some error has occurred!");
                eprintln!("{:?}", error);
            }
        }
        Ok(())
    })
}

fn read_number_from_console() -> u8{
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    trim_newline(&mut input);
    let my_int: u8 = input.parse().unwrap();
    my_int
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}
