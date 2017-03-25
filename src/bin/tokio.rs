extern crate futures;
extern crate tokio_core;
extern crate tokio_file_unix;
extern crate ring;
extern crate walkdir;
extern crate sequence_trie;

use std::fs;
use std::rc::Rc;
use std::cell::RefCell;
use std::env;
use std::path::Path;
use std::ffi::OsString;

use ring::digest::{Digest, Context, SHA256};
use futures::{Async, Poll};
use futures::future::{self, Future, Loop};
use futures::stream::Stream;
use tokio_file_unix::File;
use walkdir::{WalkDir, DirEntry};
use sequence_trie::SequenceTrie;

const BUF_SIZE: usize = 4096;
const MAX_OPEN_FILES: usize = 5;

struct DirectoryStream {
    inner: walkdir::Iter,
    open_file_count: Rc<RefCell<usize>>,
}

impl Stream for DirectoryStream {
    type Item = DirEntry;
    type Error = std::io::Error;

    fn poll(&mut self) -> Poll<Option<DirEntry>, Self::Error> {
        if *self.open_file_count.borrow() > MAX_OPEN_FILES {
            println!("throttled");
            return Ok(Async::NotReady);
        }

        match self.inner.next() {
            Some(maybe_dir) => {
                let dir = maybe_dir.expect("what the fuckitty");
                //println!("Yielding: {:?}", dir.path());
                Ok(Async::Ready(Some(dir)))
            }
            None => {
                println!("nothing left");
                Ok(Async::Ready(None))
            }
        }
    }
}

fn main() {
    let args = env::args().collect::<Vec<_>>();

    let root_path = Path::new(&args[1]);

    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();

    let open_file_count = Rc::new(RefCell::new(0));

    let dir_stream = DirectoryStream {
        inner: WalkDir::new(root_path).into_iter(),
        open_file_count: open_file_count.clone()
    };

    //let trie: SequenceTrie<OsString, Digest> = SequenceTrie::new();
    //let trie = Rc::new(RefCell::new(trie));

    core.run(dir_stream
    .filter(|dir_entry| dir_entry.file_type().is_file())
    .filter(|dir_entry| dir_entry.metadata().map(|m| m.len() > 0).unwrap_or(false))
    .for_each(|dir_entry| {
        //println!("Doing stuff!");
        *open_file_count.borrow_mut() += 1;

        let raw_file = fs::File::open(dir_entry.path()).expect("raw file");
        let file = File::new_nb(raw_file).expect("non-blocking").into_reader(&handle).expect("into_reader");

        // Hash whole file using async IO
        let init = (file, vec![0; BUF_SIZE], Context::new(&SHA256));

        let key: Vec<OsString> = dir_entry.path()
            .strip_prefix(root_path)
            .unwrap()
            .components()
            .map(|c| c.as_os_str().into()).collect();

        future::loop_fn(init, |(f, buf, mut ctxt)| {
            tokio_core::io::read(f, buf).map(|(f, buf, bytes_read)| {
                if bytes_read == 0 {
                    *open_file_count.borrow_mut() -= 1;

                    let digest = ctxt.finish();

                    //new_trie.borrow_mut().insert_owned(key, digest);
                    println!("{:?}", digest);
                    return Loop::Break(());
                }
                ctxt.update(&buf[..bytes_read]);
                Loop::Continue((f, buf, ctxt))
            })
        })
    })).expect("core loop fail");
}
