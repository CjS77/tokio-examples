use futures::{Async, Future, Poll};
use tokio::net::tcp::ConnectFuture;
use tokio::net::TcpStream;

/// A future that connects to and returns the address of a remote peer
struct GetPeerAddr {
    connect: ConnectFuture,
}

impl Future for GetPeerAddr {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        match self.connect.poll() {
            Ok(Async::Ready(s)) => {
                println!("Connection established: {}", s.peer_addr().unwrap());
                Ok(Async::Ready(()))
            }
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => {
                println!("Error making connection: {:?}", e);
                Ok(Async::Ready(()))
            }
        }
    }
}

fn main() {
    let addr = "127.0.0.1:8887".parse().unwrap();
    let connect_future = TcpStream::connect(&addr);
    let get_peer_addr = GetPeerAddr {
        connect: connect_future,
    };
    tokio::run(get_peer_addr);
}
