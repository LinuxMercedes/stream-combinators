use core::mem;
use futures::stream::{once, Chain, Once, Stream, StreamFuture};
use futures::{Async, Future, Poll};

/// Drives a stream until just before it produces a value,
/// then performs a transformation on the stream and returns
/// the transformed stream. If the driven stream reaches its
/// end without producing a value, the transformation function
/// is not called and the returned stream also ends.
///
/// This combinator is conceptually equivalent to calling
/// `.into_future().and_then()` on a stream and properly passing
/// both the returned value and the stream through a stream
/// transformation function.

/// A stream that wraps and transforms another stream
/// once it has produced a value
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Sequence<S, F, U>
where
    S: Stream,
{
    state: SeqState<S, F, U>,
}

// The type of the stream passed to the stream transformation function
// (eventually we'll get rid of this...)
type OutputStream<S> = Chain<Once<<S as Stream>::Item, <S as Stream>::Error>, S>;

#[derive(Debug)]
enum SeqState<S, F, U>
where
    S: Stream,
{
    /// Wrapped stream is done
    Done,
    /// Waiting for the wrapped stream to produce a value
    Waiting((StreamFuture<S>, F)),
    /// Streaming transformed stream values
    Streaming(U),
}

impl<S, F, U> Sequence<S, F, U>
where
    S: Stream,
    F: FnOnce(OutputStream<S>) -> U,
    U: Stream,
{
    pub fn new(stream: S, f: F) -> Sequence<S, F, U> {
        Sequence {
            state: SeqState::Waiting((stream.into_future(), f)),
        }
    }

    fn poll_stream(&mut self, mut stream: U) -> Poll<Option<U::Item>, U::Error> {
        let result = stream.poll();
        self.state = SeqState::Streaming(stream);
        result
    }
}

pub trait SequenceStream: Stream + Sized {
    /// Create a `Sequence` stream from a stream.
    ///
    /// Takes a transformation function which will be applied to
    /// the stream immediately before it produces a value.
    fn sequence<F, U>(self, f: F) -> Sequence<Self, F, U>
    where
        F: FnOnce(OutputStream<Self>) -> U,
        U: Stream,
    {
        Sequence::new(self, f)
    }
}

/// Implement `sequence` for all `Stream`s
impl<S> SequenceStream for S where S: Stream {}

impl<S, F, U> Stream for Sequence<S, F, U>
where
    S: Stream,
    F: FnOnce(OutputStream<S>) -> U,
    U: Stream,
{
    type Item = U::Item;
    type Error = U::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match mem::replace(&mut self.state, SeqState::Done) {
            SeqState::Done => Ok(Async::Ready(None)),
            SeqState::Waiting((mut stream_future, f)) => match stream_future.poll() {
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
            },
            SeqState::Streaming(stream) => self.poll_stream(stream),
        }
    }
}
