extern crate futures;
extern crate tokio_core;
extern crate tokio_stdin;
extern crate stream_combinators;

use futures::stream::Stream;
use tokio_core::reactor::Core;
use tokio_stdin::spawn_stdin_stream_unbounded;
use stream_combinators::FilterFoldStream;

fn main() {
    let mut core = Core::new().unwrap();

    // Print stdin line-by-line
    let prog = spawn_stdin_stream_unbounded()
        .filter_fold(|mut buf, elem| {
            match elem {
                // Got a newline
                Some(byte) if byte == '\n' as u8 => {
                    let s = String::from_utf8(buf).unwrap();
                    println!("debug stdin: {}", s);
                    (vec![], Some(s))
                },
                // Got a byte
                Some(byte) => {
                    buf.push(byte);
                    (buf, None)
                },
                // Got EOF
                None => {
                    if buf.len() > 0 {
                        let s = String::from_utf8(buf).unwrap();
                        println!("debug stdin is done: {}", s);
                        (vec![], Some(s))
                    }
                    else {
                        println!("debug stdin is done; no line");
                        (buf, None)
                    }
                }
            }
        }, vec![])
        .for_each(|line| {
            println!("{}", line);
            Ok(())
        });


    core.run(prog).unwrap()
}