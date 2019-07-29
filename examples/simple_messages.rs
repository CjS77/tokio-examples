use futures::future::lazy;
use futures::stream;
use futures::sync::{mpsc, oneshot};
use futures::{Future, Sink, Stream};
use std::thread;
use std::time::Duration;

fn main() {
    // Create a future that spawns two sub-futures (tasks) onto the default tokio runtime - the first simply
    // sends a message on the channel, the other waits for it, and then prints it.
    // We use a oneshot message here. Oneshots are great for sending the result of a computation over to another task
    let future = lazy(|| {
        let (tx, rx) = oneshot::channel();

        tokio::spawn(lazy(|| {
            println!("In Task A");
            thread::sleep(Duration::from_millis(750));
            println!("Sending message");
            let _ = tx.send("Hello from spawned task");
            Ok(())
        }));

        rx.and_then(|msg| {
            println!("Message received: {}", msg);
            Ok(())
        })
        .map_err(|e| println!("error = {:?}", e))
    });

    tokio::run(future);

    // Now use an mpsc channel to send multiple messages this time
    let future = lazy(|| {
        let (tx, rx) = mpsc::channel(64);

        let stream = stream::iter_ok(1..10).map(|v| v * v);
        // You need to use fold here because the closure must be static, so we can't borrow tx; we have to pass it
        // around
        tokio::spawn(
            stream
                .fold(tx, |tx, v| {
                    thread::sleep(Duration::from_millis(100));
                    println!("Sending a message from producer: {}", v);
                    tx.send(format!("Found a square: {}", v))
                        .map_err(|e| println!("Error: {:?}", e))
                })
                .map(|_| ()), //Drop tx handle
        );

        rx.for_each(|msg| {
            println!("Received: {}", msg);
            Ok(())
        })
    });

    tokio::run(future);
}
