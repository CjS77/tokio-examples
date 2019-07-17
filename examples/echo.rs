use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::*;

fn main() {
    let addr = "127.0.0.1:8777".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();
    let server = listener
        .incoming()
        .for_each(|socket| {
            // split the socket stream into readable and writable parts
            let (reader, writer) = socket.split();
            // copy bytes from the reader into the writer
            let amount = io::copy(reader, writer);
            let msg = amount.then(|res| {
                match res {
                    Ok((b, _, _)) => println!("{} bytes written", b),
                    Err(e) => println!("Error copying data: {:?}", e),
                }
                Ok(())
            });
            // Spawn the future onto the runtime. Withotu this, each incoming connection would run sequentially.
            tokio::spawn(msg);
            Ok(())
        })
        .map_err(|e| {
            println!("Error binding server: {:?}", e);
        });

    println!("server running...");
    tokio::run(server);
}
