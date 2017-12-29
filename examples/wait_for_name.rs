extern crate futures;
extern crate tokio_core;
extern crate tokio_stdin;
extern crate stream_sequence;

use futures::stream::Stream;
use tokio_core::reactor::Core;
use tokio_stdin::spawn_stdin_stream_unbounded;
use stream_sequence::Sequence;

fn main() {
    let mut core = Core::new().unwrap();

    // Read bytes until we get an 's'
    let stdin = spawn_stdin_stream_unbounded()
        .skip_while(|byte| {
            Ok(*byte != ('s' as u8))
        });

    // Afterwards, map the resulting stream to print the bytes we read
    let prog = Sequence::new(stdin, |input| {
        input.map(|byte| println!("{:?}", byte))
    }).for_each(|_| Ok(()));

    core.run(prog).unwrap()
}