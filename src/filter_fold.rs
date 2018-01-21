use futures::{Poll, Async, Future, IntoFuture};
use futures::stream::Stream;
use core::mem;

pub struct FilterFold<S, F, A, Fut>
    where S: Stream,
          F: FnMut(A, S::Item) -> Fut,
          Fut: IntoFuture,
{
    stream: S,
    f: F,
    state: State<A, Fut::Future>,
}

enum State<A, Fut>
    where Fut: Future
{
    Empty,
    Ready(A),
    Processing(Fut),
}

impl<S, F, A, Fut, U> FilterFold<S, F, A, Fut>
    where S: Stream,
          S::Error: From<Fut::Error>,
          F: FnMut(A, S::Item) -> Fut,
          Fut: IntoFuture<Item = (A, Option<U>)>,
{
    pub fn new(stream: S, acc: A, f: F) -> FilterFold<S, F, A, Fut> {
        FilterFold {
            stream: stream,
            f: f,
            state: State::Ready(acc),
        }
    }
}

pub trait FilterFoldStream: Stream + Sized {
    fn filter_fold<F, A, Fut, U>(self, acc: A, f: F) -> FilterFold<Self, F, A, Fut>
        where Self::Error: From<Fut::Error>,
              F: FnMut(A, Self::Item) -> Fut,
              Fut: IntoFuture<Item = (A, Option<U>)>,
    {
        FilterFold::new(self, acc, f)
    }
}

impl<S> FilterFoldStream for S
    where S: Stream
{ }

impl<S, F, A, Fut, U> Stream for FilterFold<S, F, A, Fut>
    where S: Stream,
          S::Error: From<Fut::Error>,
          F: FnMut(A, S::Item) -> Fut,
          Fut: IntoFuture<Item = (A, Option<U>)>
{
    type Item = U;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop { // I dream of explicit tail call optimization...
            match mem::replace(&mut self.state, State::Empty) {
                State::Empty => panic!("cannot poll FilterFold twice"),
                State::Ready(acc) => {
                    match self.stream.poll()? {
                        Async::Ready(None) => {
                            return Ok(Async::Ready(None));
                        },
                        Async::Ready(Some(elem)) => {
                            let future = (self.f)(acc, elem);
                            let future = future.into_future();
                            self.state = State::Processing(future);
                        },
                        Async::NotReady => {
                            self.state = State::Ready(acc);
                            return Ok(Async::NotReady);
                        }
                    }
                },
                State::Processing(mut future) => {
                    match future.poll()? {
                        Async::Ready((acc, res)) => {
                            self.state = State::Ready(acc);
                            if let Some(fold_elem) = res {
                                return Ok(Async::Ready(Some(fold_elem)))
                            }
                        }
                        Async::NotReady => {
                            self.state = State::Processing(future);
                            return Ok(Async::NotReady);
                        }
                    }
                }
            }
        }
    }
}
