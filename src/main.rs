extern crate ring;
extern crate antifa;
extern crate walkdir;
extern crate itertools;
extern crate sequence_trie;
extern crate futures;
extern crate futures_cpupool;

use sequence_trie::SequenceTrie;
use std::path::{Path, PathBuf};
use std::ffi::OsString;
use walkdir::WalkDir;
use std::env;
use ring::digest::Digest;
use itertools::Itertools;
use futures::{future, Future};
use futures_cpupool::CpuPool;

use antifa::hash::{hash_file, combine_digests};

fn main() {
    let args = env::args().collect::<Vec<_>>();

    let root_path = Path::new(&args[1]);

    let pool = CpuPool::new(32);

    let mut futures = vec![];

    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file()) {

        //println!("+");
        let path = entry.path().to_path_buf();
        let future = pool.spawn_fn(move || -> Result<(PathBuf, Digest), ()> {
            //println!(">");
            let file_hash = hash_file(&path).expect("couldn't hash file");
            Ok((path, file_hash))
        });
        futures.push(future);
    }


    let mut trie: SequenceTrie<OsString, Digest> = SequenceTrie::new();

    for (path, hash) in future::join_all(futures).wait().unwrap() {
        let key = path
            .strip_prefix(root_path)
            .unwrap()
            .components()
            .map(|c| c.as_os_str());

        trie.insert(key, hash);
    }

    trie.map(|node| {
        if node.is_leaf() {
            None
        } else {
            // FIXME: avoid allocations here :\
            let sorted_child_digests = node.children_with_keys()
                .into_iter()
                .sorted_by(|&(k1, _), &(k2, _)| k1.cmp(k2))
                .into_iter()
                .map(|(_, child_node)| child_node.value().unwrap())
                .collect();
            Some(combine_digests(sorted_child_digests))
        }
    });

    for (k, v) in trie.iter().sorted_by(|&(ref k1, _), &(ref k2, _)| k1.cmp(k2)) {
        println!("{:?}: {:?}", k, v);
    }
}
