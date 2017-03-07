extern crate futures;
extern crate tokio_core;
extern crate tokio_file_unix;

extern crate env_logger;

use std::fs;
use std::env;

use futures::future::{self, Future, Loop};
use tokio_file_unix::File;

#[no_mangle]
fn greatness() {
    env_logger::init().unwrap();
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();

    let filename = env::args().nth(1).unwrap();

    let raw_file = fs::File::open(filename).expect("raw file");
    let file = File::new_nb(raw_file).expect("non-blocking").into_reader(&handle).expect("into_reader");

    let init = (file, vec![0; 256]);

    // FIXME: empty files seem to break this... why?
    let f = future::loop_fn(init, |(f, buf)| {
        tokio_core::io::read(f, buf).map(|(f, buf, bytes_read)| {
            if bytes_read == 0 {
                println!("read 0 bytes, done");
                return Loop::Break(());
            }
            println!("got this: {:?}", &buf[..bytes_read]);
            Loop::Continue((f, buf))
        })
    });

    core.run(f).expect("core loop fail");
}

fn main() {
    greatness();
}
