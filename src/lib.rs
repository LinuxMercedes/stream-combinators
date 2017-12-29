extern crate futures;
extern crate core;

pub mod filter_fold;
pub use filter_fold::FilterFoldStream;

use futures::{Poll, Async, Future};
use futures::stream::{Stream, Chain, StreamFuture, IterResult, iter_result};
use std::vec::IntoIter;
use std::result::Result;

type OutputStream<S> = Chain<IterResult<IntoIter<Result<<S as Stream>::Item, <S as Stream>::Error>>>, S>;

enum SeqState<S,U>
    where S: Stream,
          U: Stream
{
    Future(StreamFuture<S>),
    Stream(U),
}

pub struct Sequence<S, F, U>
    where S: Stream,
          F: FnOnce(OutputStream<S>) -> U,
          U: Stream
{
    state: SeqState<S, U>,
    f: Option<F>
}

impl<S, F, U> Sequence<S, F, U>
    where S: Stream,
          F: FnOnce(OutputStream<S>) -> U,
          U: Stream
{
    pub fn new(stream: S, f: F) -> Sequence<S, F, U>
    {
        Sequence {
            state: SeqState::Future(stream.into_future()),
            f: Some(f)
        }
    }
}

// TODO rewrite and use linear types instead of a sum type!
impl<S, F, U> Stream for Sequence<S, F, U>
    where S: Stream,
          F: FnOnce(OutputStream<S>) -> U,
          U: Stream
{
    type Item = U::Item;
    type Error = U::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let state = match self.state {
            SeqState::Future(ref mut stream_future) => match stream_future.poll() {
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Ok(Async::Ready((Some(val), stream))) => {
                    let val_n_stream = iter_result(vec![Ok(val)]).chain(stream);
                    let f = self.f.take().unwrap();
                    let the_stream = f(val_n_stream);
                    Some(SeqState::Stream(the_stream))
                },
                Ok(Async::Ready((None, _stream))) => {
                    return Ok(Async::Ready(None))
                },
                Err((err, stream)) => {
                    let err_n_stream = iter_result(vec![Err(err)]).chain(stream);
                    let f = self.f.take().unwrap();
                    let the_stream = f(err_n_stream);
                    Some(SeqState::Stream(the_stream))
                },
            },
            SeqState::Stream(_) => None,
        };

        if let Some(new_state) = state {
            self.state = new_state;
        }

        match self.state {
            SeqState::Future(_) => panic!("oops"),
            SeqState::Stream(ref mut stream) => stream.poll(),
        }
    }
}
