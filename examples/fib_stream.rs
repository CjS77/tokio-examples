use futures::{try_ready, Async, Future, Stream};
use std::env;
use std::time::Duration;
use tokio::timer::Interval;

struct FibonacciStream {
    cur: usize,
    next: usize,
}

impl FibonacciStream {
    fn new() -> FibonacciStream {
        FibonacciStream { cur: 1, next: 1 }
    }
}

impl Stream for FibonacciStream {
    type Item = usize;
    type Error = ();

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        let val = self.cur;
        let next_val = val + self.next;

        self.cur = self.next;
        self.next = next_val;

        Ok(Async::Ready(Some(val)))
    }
}

/// A future that takes n values from a stream
struct TakeFromStream<T: Stream> {
    stream: T,
    values: Vec<T::Item>,
    n: u64,
}

impl<T> TakeFromStream<T>
where
    T: Stream,
{
    fn new(s: T) -> TakeFromStream<T> {
        TakeFromStream {
            stream: s,
            values: Vec::new(),
            n: 1,
        }
    }

    fn take(mut self, n: u64) -> Self {
        self.n = n;
        self
    }
}

impl<T> Future for TakeFromStream<T>
where
    T: Stream,
    T::Item: Clone,
{
    type Item = Vec<T::Item>;
    type Error = T::Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        while (self.values.len() as u64) < self.n {
            let val = try_ready!(self.stream.poll());
            match val {
                Some(v) => self.values.push(v),
                None => return Ok(Async::Ready(self.values.clone())),
            }
        }
        Ok(Async::Ready(self.values.clone()))
    }
}

fn main() {
    let stream = FibonacciStream::new();
    let args: Vec<String> = env::args().collect();
    let n = if args.len() < 2 {
        5u64
    } else {
        args[1].parse().unwrap_or(5u64)
    };
    // This future waits until all the values are collected
    //let set = TakeFromStream::new(stream);
    //let future = set.take(n).map(move |res| println!("{} Fibonaccis = {:?}", n, res));

    // This future spits out each one as it arrives
    let interval = Interval::new_interval(Duration::from_secs(1)).map_err(|_| ());
    let future = interval.zip(stream).take(n).for_each(|(_, v)| {
        println!("{}", v);
        Ok(())
    });
    tokio::run(future);
}
