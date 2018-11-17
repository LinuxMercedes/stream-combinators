extern crate futures;
extern crate stream_combinators;
extern crate tokio;
extern crate tokio_stdin;
extern crate tokio_timer;

use futures::stream::{once, Stream};
use std::time::Duration;
use stream_combinators::SequenceStream;
use tokio_stdin::spawn_stdin_stream_unbounded;
use tokio_timer::Interval;

enum Event {
    Byte,
    Second,
    Done,
}

fn main() {
    // Read bytes until we get a newline
    let stdin = spawn_stdin_stream_unbounded().skip_while(|byte| Ok(*byte != ('\n' as u8)));

    let mut bytes = 0;
    let mut seconds = 0;

    // Map our input into Byte events and indicate when we hit EOF
    let stdin = stdin
        .map(|_| Event::Byte)
        .map_err(|_| ())
        .chain(once(Ok(Event::Done)));
    // Map our timer into Second events
    let timer = Interval::new_interval(Duration::from_secs(1))
        .map(|_| Event::Second)
        .map_err(|_| ());

    // Afterwards, count the bytes received per second;
    // without `.sequence`, the timer would start counting before the first
    // newline arrived.
    let prog = stdin
        .sequence(move |stdin| {
            stdin.select(timer).and_then(move |event| match event {
                Event::Byte => {
                    bytes += 1;
                    Ok(())
                }
                Event::Second => {
                    seconds += 1;
                    println!("{} bytes in {} seconds", bytes, seconds);
                    Ok(())
                }
                Event::Done => {
                    println!("{} bytes in {}+ seconds", bytes, seconds);
                    Err(())
                }
            })
        }).for_each(|_| Ok(()));

    tokio::run(prog)
}
