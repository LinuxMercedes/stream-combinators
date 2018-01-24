extern crate futures;
extern crate stream_combinators;
extern crate tokio_core;
extern crate tokio_stdin;
extern crate tokio_timer;

use futures::stream::{Stream, once};
use stream_combinators::SequenceStream;
use tokio_core::reactor::Core;
use tokio_stdin::spawn_stdin_stream_unbounded;
use tokio_timer::Timer;
use std::time::Duration;

enum Event {
    Byte,
    Second,
    Done,
}

fn main() {
    let mut core = Core::new().unwrap();

    // Read bytes until we get a newline
    let stdin = spawn_stdin_stream_unbounded().skip_while(|byte| Ok(*byte != ('\n' as u8)));

    let mut bytes = 0;
    let mut seconds = 0;

    // Afterwards, count the bytes received per second;
    // without `.sequence`, the timer would start counting before the first
    // newline arrived.
    let prog = stdin.sequence(|input| {
            // Map our input into Byte events and indicate when we hit EOF
            let input = input.map(|_| Event::Byte)
                .map_err(|_| ())
                .chain(once(Ok(Event::Done)));
            // Map our timer into Second events
            let timer = Timer::default()
                .interval(Duration::from_secs(1))
                .map(|_| Event::Second)
                .map_err(|_| ());

            input.select(timer).and_then(|event| {
                match event {
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
                }
            })
        })
        .for_each(|_| Ok(()));

    core.run(prog).unwrap_or(());
}
