use tokio::prelude::Future;
use tokio::prelude::future::{lazy, poll_fn};
use tokio_threadpool::{blocking, BlockingError};
use rand::Rng;
use std::time::{Instant, Duration};
use futures::Async;
use tokio_executor::enter;
use std::thread;

/// A really slow inefficient function for finding out if a value is prime
fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    let n_sqrt = f64::sqrt(n as f64);
    let n_sqrt = n_sqrt.trunc() as u64;
    (2..=n_sqrt).all(|v| n % v != 0)
}

/// An even more inefficient prime finding algorithm
fn find_nth_prime(n: u64) -> u64 {
    let mut found_primes = 0u64;
    let mut candidate = 1u64;
    while found_primes < n {
        candidate += 1;
        if is_prime(candidate) {
            found_primes += 1;
        }
    }
    candidate
}

/// The async version of [find_nth_prime] which wraps the function inside a future that will be run on a blocking
/// thread. The future resolves to the prime value and the time it took to find.
fn async_nth_prime(i: usize, n: u64) -> impl Future<Item=(usize, u64, u64, Duration), Error=BlockingError> {
    // The simplest way to wrap a future around a function is to call `poll_fn`. In fact this is exactly what poll_fn
    // does: it takes a fn or closure (which must return Poll<T, E>) and makes a future out of it. In this case
    // we're going to wrap the super-slow prime finding function and return the job #, n, the prime, and the time it
    // took to calculate.
    poll_fn(move || {
        println!("Starting job {}: {}th prime", i, n);
        // Now we can't really just make our poll function take a long time, because this would block up the worker
        // thread and prevent any other futures in the worker queue from making progress. Instead we delegate the
        // working to a blocking thread by calling `blocking`.
        // If you comment out the blocking, you'll see the difference
        blocking(move || {
            let start = Instant::now();
            let p = find_nth_prime(n);
            let time = start.elapsed();
            (i, n, p, time)
//            Ok(Async::Ready((i, n, p, time)))
        })
    })
}

fn launch_default_executor<F: Future<Item=(), Error=()> + Send + 'static>(future: F) {
    // Check enter before creating a new Runtime...
    let mut entered = enter().expect("A tokio runtine is already running");
    let n_threads = 5;
    println!("Using {} blocking threads", n_threads);
    let timer = Instant::now();
    let mut rt = tokio::runtime::Builder::new()
        .blocking_threads(n_threads)
        // Run the work scheduler on one thread so we can really see the effects of using `blocking` above
        .core_threads(1)
        .build()
        .expect("Could not start tokio runtime");
    rt.spawn(future);

    println!("Starting the Runtime");
    entered.on_exit(move || {
        println!("Shutting down. Bye {}s", timer.elapsed().as_millis() as f64 * 0.001);
    });
    entered
        .block_on(rt.shutdown_on_idle())
        .expect("shutdown cannot error");
}

fn main() {
    let mut rng = rand::thread_rng();
    let values: Vec<u64> = (0..100).map(|_| rng.gen_range(5_000u64, 100_000u64)).collect();
    let future = lazy(move || {
        for (i, v) in values.into_iter().enumerate() {
            // Tasks are futures that must have () as their item, so we need to handle the result of the
            // `async_nth_prime` future first.
            let task = async_nth_prime(i, v).then(|res| {
                match res {
                    Ok((i, n, v, t)) => {
                        println!("The {:6}th prime is {:12} (Job #{} found in {:6.3}s)", n, v, i, t.as_millis() as f64 * 0.001);
                    }
                    Err(e) => unreachable!("Error: {:?}", e),
                }
                Ok(())
            });
            tokio::spawn(task);
        }
        Ok(())
    });

    launch_default_executor(future);
}
