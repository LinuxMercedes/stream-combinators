extern crate futures;
extern crate stream_combinators;

use futures::stream::{iter_ok, Stream};
use futures::Future;
use stream_combinators::FilterFoldStream;

fn main() {
    let number_stream = iter_ok::<_,()>((0..6));
    let pairs = number_stream.filter_fold(None, |acc, val| {
        match acc {
            Some(prev) => Ok((None, Some((prev, val)))),
            None => Ok((Some(val), None)),
        }
    }).collect();
    assert_eq!(pairs.wait(), Ok(vec![(0,1), (2,3), (4,5)]))
}
