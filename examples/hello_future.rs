use bytes::{Buf, Bytes};
use futures::{try_ready, Async, Future};
use std::io::{self, Cursor};
use tokio::io::AsyncWrite;
use tokio::net::tcp::ConnectFuture;
use tokio::net::TcpStream;

/// A future that is in the connecting state while connecting to a remote peer, and writes a buffer to the peer once
/// the connection is established
enum HelloWorld {
    Connecting(ConnectFuture),
    Connected(TcpStream, Cursor<Bytes>),
}

impl Future for HelloWorld {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        loop {
            match self {
                // When in Connecting state, check if we've connected yet and move to next state
                HelloWorld::Connecting(ref mut f) => {
                    let socket = try_ready!(f.poll());
                    let data = Cursor::new(Bytes::from_static(b"Hello, world"));
                    *self = HelloWorld::Connected(socket, data);
                }
                HelloWorld::Connected(s, c) => {
                    while c.has_remaining() {
                        try_ready!(s.write_buf(c));
                    }
                    return Ok(Async::Ready(()));
                }
            }
        }
    }
}

fn main() {
    let addr = "127.0.0.1:8887".parse().unwrap();
    let connection = TcpStream::connect(&addr);
    // Version 1, using a custom future
    //let hello_world = HelloWorld::Connecting(connection).map_err(|e| println!("Error: {:?}", e));
    // Version 2, using combinators
    let hello_world = connection
        .and_then(|s| tokio::io::write_all(s, b"Hello world, Mk2"))
        .map(|_| println!("Done"))
        .map_err(|e| println!("Error: {:?}", e));
    tokio::run(hello_world);
}
