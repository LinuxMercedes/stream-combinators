use futures::{Poll, Async, Future, IntoFuture};
use futures::stream::Stream;
use std::fmt::Debug;
use core::mem;

/* FilterFold :: (F: (Acc, S_elem) -> (Acc, Option<FF_elem>)) -> Acc -> Stream<S_elem> ->
 * Stream<FF_elem>
 * FilterFold fn, acc, stream = do
 *   s_elem <- stream
 *   (acc, res) = fn(acc, s_elem)
 *   if let Some(ff_elem) = res
 *      yield ff_elem
 */

pub struct FilterFold<S, F, A, U>
    where S: Stream,
          F: FnMut(&mut A, Option<S::Item>) -> (A, Option<U>)
{
    stream: S,
    f: F,
    acc: A,
    done: bool,
}

impl<S, F, A, U> FilterFold<S, F, A, U>
    where S: Stream,
          F: FnMut(&mut A, Option<S::Item>) -> (A, Option<U>)
{
    pub fn new(stream: S, f: F, acc: A) -> FilterFold<S, F, A, U> {
        FilterFold {
            stream: stream,
            f: f,
            acc: acc,
            done: false,
        }
    }
}

pub trait FilterFoldStream: Stream + Sized {
    fn filter_fold<F, A, U>(self, f: F, acc: A) -> FilterFold<Self, F, A, U>
          where F: FnMut(&mut A, Option<Self::Item>) -> (A, Option<U>)
    {
        FilterFold::new(self, f, acc)
    }
}

impl<S> FilterFoldStream for S
    where S: Stream
{ }

impl<S, F, A, U> Stream for FilterFold<S, F, A, U>
    where S: Stream,
          F: FnMut(&mut A, Option<S::Item>) -> (A, Option<U>),
          S::Item: Debug, A: Debug, U: Debug
{
    type Item = U;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        println!("Polling filterfold");

        loop {
            if self.done {
                println!("Done");
                return Ok(Async::Ready(None))
            } else {
                match self.stream.poll()? {
                    Async::Ready(elem) => {
                        println!("Got something: {:?}", elem);
                        self.done = elem.is_none();

                        let (acc, res) = (self.f)(&mut self.acc, elem);
                        self.acc = acc;
                        println!("acc: {:?}, res: {:?}", self.acc, res);

                        if let Some(fold_elem) = res {
                            return Ok(Async::Ready(Some(fold_elem)))
                        }
                    },
                    Async::NotReady => return Ok(Async::NotReady),
                }
            }
        }
    }
}
