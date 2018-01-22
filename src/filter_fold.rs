use futures::{Poll, Async, Future, IntoFuture};
use futures::stream::Stream;
use core::mem;

/// Execute an accumulating computation over a stream,
/// collecting values into a stream of computation results.
///
/// This combinator will collect all successful results of this stream
/// according to the closure provided. The closure takes two parameters:
/// the accumulator and the next stream result. It returns a tuple
/// containing the new accumulator value and an `Option`. If the returned option
/// is `Some`, the associated value is yielded from the returned stream.
///
/// Note: it is possible for the accumulator to contain intermediate values
/// when the stream is closed; if you need to access the accumulator's value
/// once the stream closes, we suggest chaining a sentinel value onto the end
/// of your stream.
///
/// # Examples
///
/// ```
/// use futures::stream::{iter_ok, Stream};
/// use futures::Future;
/// use stream_combinators::FilterFoldStream;
///
/// let number_stream = iter_ok::<_,()>((0..6));
/// let pairs = number_stream.filter_fold(None, |acc, val| {
///     match acc {
///         Some(prev) => Ok((None, Some((prev, val)))),
///         None => Ok((Some(val), None)),
///     }
/// }).collect();
/// assert_eq!(pairs.wait(), Ok(vec![(0,1), (2,3), (4,5)]))
/// ```

/// A stream used to collect results from one stream into a new stream
///
/// This stream is returned by the `FilterFoldStream::filter_fold` method.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct FilterFold<S, F, A, Fut>
    where S: Stream,
          F: FnMut(A, S::Item) -> Fut,
          Fut: IntoFuture,
{
    stream: S,
    f: F,
    state: State<A, Fut::Future>,
}

#[derive(Debug)]
enum State<A, Fut>
    where Fut: Future
{
    /// Placeholder when doing work
    Empty,
    /// Ready to process the next stream item; current accumulator is the `A`
    Ready(A),
    /// Working on a future that processes the current stream item
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
    /// Create a `FilterFold` stream from a stream.
    ///
    /// Takes an inital accumulator value and an `FnMut` which takes
    /// two parameters, the current accumulator value and the current
    /// value from the stream, and returns a `Future` which resolves to
    /// a tuple where the first item is the new accumulator value
    /// and the second item is an `Option`. The `FilterFold` stream contains
    /// the values that are `Some`.
    fn filter_fold<F, A, Fut, U>(self, acc: A, f: F) -> FilterFold<Self, F, A, Fut>
        where Self::Error: From<Fut::Error>,
              F: FnMut(A, Self::Item) -> Fut,
              Fut: IntoFuture<Item = (A, Option<U>)>,
    {
        FilterFold::new(self, acc, f)
    }
}

/// Implement `filter_fold` for all `Stream`s
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

    // Inspiration/code borrowed from:
    // https://github.com/alexcrichton/futures-rs/blob/master/src/stream/fold.rs
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop { // I dream of explicit tail call optimization...
            match mem::replace(&mut self.state, State::Empty) {
                State::Empty => panic!("cannot poll FilterFold twice"),
                State::Ready(acc) => { // Ready; get next stream item
                    match self.stream.poll()? {
                        Async::Ready(None) => {
                            return Ok(Async::Ready(None));
                        },
                        Async::Ready(Some(elem)) => {
                            let future = (self.f)(acc, elem);
                            let future = future.into_future();
                            self.state = State::Processing(future);
                            // loop around and start processing the future
                        },
                        Async::NotReady => {
                            self.state = State::Ready(acc);
                            return Ok(Async::NotReady);
                        }
                    }
                },
                State::Processing(mut future) => { // Busy; get a result from the future
                    match future.poll()? {
                        Async::Ready((acc, res)) => {
                            self.state = State::Ready(acc);
                            if let Some(fold_elem) = res {
                                return Ok(Async::Ready(Some(fold_elem)))
                            }
                            // Otherwise loop around and start processing the next item
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
