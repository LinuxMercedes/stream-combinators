extern crate futures;
extern crate tokio_core;
extern crate tokio_stdin;
extern crate stream_combinators;

use futures::stream::{once, Stream};
use tokio_core::reactor::Core;
use tokio_stdin::spawn_stdin_stream_unbounded;
use stream_combinators::FilterFoldStream;

const NEWLINE: u8 = '\n' as u8;

fn main() {
    let mut core = Core::new().unwrap();

    // Print stdin line-by-line
    let prog = spawn_stdin_stream_unbounded()
        // Wrap stream values so we can give a sentinel for EOF
        .map(Some)
        .chain(once(Ok(None)))
        // Accumulate bytes into lines
        .filter_fold(vec![], |mut buf, val| {
            match val {
                Some(NEWLINE) => {
                    let s = String::from_utf8(buf).unwrap();
                    Ok((vec![], Some(s)))
                },
                Some(byte) => {
                    buf.push(byte);
                    Ok((buf, None))
                },
                None => {
                    if buf.len() > 0 {
                        let s = String::from_utf8(buf).unwrap();
                        Ok((vec![], Some(s)))
                    } else {
                        Ok((buf, None))
                    }
                }
            }
        })
        .for_each(|line| {
            println!("{}", line);
            Ok(())
        });


    core.run(prog).unwrap()
}