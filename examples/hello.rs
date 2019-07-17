use tokio::io;
use tokio::net::TcpStream;
use tokio::prelude::*;

fn main() {
    let addr = "127.0.0.1:8777".parse().unwrap();
    let client = TcpStream::connect(&addr)
        .and_then(|s| {
            println!("Connected");
            io::write_all(s, "Hello, world\n")
        })
        .then(|r| {
            match r {
                Ok(_) => println!("Written"),
                Err(e) => println!("Error writing to socket: {:?}", e),
            };
            Ok(())
        });
    println!("Starting");
    tokio::run(client);
    println!("Bye");
}
