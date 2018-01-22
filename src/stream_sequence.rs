use futures::{Poll, Async, Future};
use futures::stream::{Stream, Chain, StreamFuture, Once, once};
use core::mem;

type OutputStream<S> = Chain<Once<<S as Stream>::Item, <S as Stream>::Error>, S>;

enum SeqState<S, F, U>
    where S: Stream
{
    Done,
    Waiting((StreamFuture<S>, F)),
    Streaming(U),
}

pub struct Sequence<S, F, U>
    where S: Stream
{
    state: SeqState<S, F, U>,
}

impl<S, F, U> Sequence<S, F, U>
    where S: Stream,
          F: FnOnce(OutputStream<S>) -> U,
          U: Stream
{
    pub fn new(stream: S, f: F) -> Sequence<S, F, U> {
        Sequence { state: SeqState::Waiting((stream.into_future(), f)) }
    }

    fn poll_stream(&mut self, mut stream: U) -> Poll<Option<U::Item>, U::Error> {
        let result = stream.poll();
        self.state = SeqState::Streaming(stream);
        result
    }
}

pub trait SequenceStream: Stream + Sized {
    fn sequence<F, U>(self, f: F) -> Sequence<Self, F, U>
        where F: FnOnce(OutputStream<Self>) -> U,
              U: Stream
    {
        Sequence::new(self, f)
    }
}

impl<S> SequenceStream for S where S: Stream {}

impl<S, F, U> Stream for Sequence<S, F, U>
    where S: Stream,
          F: FnOnce(OutputStream<S>) -> U,
          U: Stream
{
    type Item = U::Item;
    type Error = U::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match mem::replace(&mut self.state, SeqState::Done) {
            SeqState::Done => Ok(Async::Ready(None)),
            SeqState::Waiting((mut stream_future, f)) => {
                match stream_future.poll() {
                    Ok(Async::Ready((Some(val), stream))) => {
                        let stream = once(Ok(val)).chain(stream);
                        let stream = f(stream);
                        self.poll_stream(stream)
                    }
                    Err((err, stream)) => {
                        let stream = once(Err(err)).chain(stream);
                        let stream = f(stream);
                        self.poll_stream(stream)
                    }
                    Ok(Async::NotReady) => {
                        self.state = SeqState::Waiting((stream_future, f));
                        Ok(Async::NotReady)
                    }
                    Ok(Async::Ready((None, _stream))) => {
                        self.state = SeqState::Done;
                        Ok(Async::Ready(None))
                    }
                }
            }
            SeqState::Streaming(stream) => self.poll_stream(stream),
        }
    }
}
