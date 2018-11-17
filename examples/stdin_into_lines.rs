extern crate futures;
extern crate stream_combinators;
extern crate tokio;
extern crate tokio_stdin;

use futures::stream::{once, Stream};
use stream_combinators::FilterFoldStream;
use tokio_stdin::spawn_stdin_stream_unbounded;

fn main() {
    // Print stdin line-by-line
    let prog = spawn_stdin_stream_unbounded()
        // Include an extra newline in case the input is missing a trailing newline
        // Note: this will print a spurious blank line in the case the input has
        // a trailing newline; see `examples/fancier_stdin_into_lines.rs`
        // for a way to prevent that situation, if it matters.
        .chain(once(Ok('\n' as u8)))
        // Accumulate bytes into lines
        .filter_fold(vec![], |mut buf, byte| {
            if byte == ('\n' as u8) {
                let s = String::from_utf8(buf).unwrap();
                Ok((vec![], Some(s)))
            } else {
                buf.push(byte);
                Ok((buf, None))
            }
        }).for_each(|line| {
            println!("{}", line);
            Ok(())
        });

    tokio::run(prog)
}
