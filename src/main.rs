use futures::{Async, Future};

fn main() {
    println!("run cargo --example [name]")
}

trait AsyncMessage<R, T, E> {
    type Request = R;
    type Response = T;
    type Error = E;
}

impl Future for AsyncMessage<R, T, E> {
    type Item = R;
    type Error = E;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        unimplemented!()
    }
}
