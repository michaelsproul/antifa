extern crate ring;
extern crate antifa;
extern crate walkdir;
extern crate sequence_trie;
extern crate data_encoding;

use sequence_trie::SequenceTrie;
use std::path::{Path, Component};
use std::ffi::OsString;
use walkdir::WalkDir;
use std::env;
use data_encoding::hex;
use ring::digest::Digest;

use antifa::hash::{hash_file, combine_digests, dummy_digest};

fn main() {
    let args = env::args().collect::<Vec<_>>();

    let mut trie: SequenceTrie<OsString, Digest> = SequenceTrie::new();
    let root_path = Path::new(&args[1]);

    for raw_entry in WalkDir::new(root_path) {
        let entry = match raw_entry {
            Ok(e) => e,
            Err(err) => { println!("{:?}", err); continue }
        };

        let key = entry.path().components().map(|c| c.as_os_str());

        if entry.file_type().is_dir() {
            // Post-order visit, calculate the child count!
            if entry.is_second_visit() {
                let subtrie = trie.get_node_mut(key).unwrap();

                let dir_hash = {
                    // FIXME: maybe use a BTreeMap in the trie to get this for free
                    let mut children = subtrie.children_with_keys();
                    children.sort_by_key(|&(filename, _)| filename);

                    let digests = children.iter()
                        .map(|&(_, node)| node.value().unwrap())
                        .collect();

                    combine_digests(digests)
                };

                *subtrie.value_mut().unwrap() = dir_hash;
            }
            // Pre-order visit, insert the directory with a dummy value.
            else {
                // FIXME: investigate the most efficient dummy value/disable pre-order visists
                trie.insert(key, dummy_digest());
            }
        } else if entry.file_type().is_file() {
            let file_hash = hash_file(entry.path()).expect("couldn't hash file");
            trie.insert(key, file_hash);
        }
    }

    for (k, v) in trie.iter() {
        println!("{:?}: {}", k, hex::encode(v.as_ref()).to_lowercase());
    }

    let root_key = root_path.components().map(Component::as_os_str);
    let root_hash = trie.get(root_key).unwrap();
    println!("Root hash: {}", hex::encode(root_hash.as_ref()).to_lowercase());
}
